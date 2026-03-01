mod cli;
mod config;

use anyhow::{Result, anyhow, bail};
use clap::Parser;
use openrouter_rs::{
    OpenRouterClient,
    api::{discovery, models},
    types::{ModelCategory, SupportedParameters},
};
use serde::Serialize;

use crate::{
    cli::{
        Cli, Commands, ConfigCommands, ModelCategoryArg, ModelsCommands, OutputFormat,
        ProfileCommands, ProvidersCommands, SupportedParameterArg,
    },
    config::{Environment, ResolvedProfile, resolve_profile},
};

#[derive(Debug, Serialize)]
struct ConfigSnapshot<'a> {
    profile: &'a str,
    profile_source: &'a config::ValueSource,
    config_path: String,
    config_path_source: &'a config::ValueSource,
    base_url: &'a str,
    base_url_source: &'a config::ValueSource,
    api_key_present: bool,
    api_key_source: &'a config::ValueSource,
    management_key_present: bool,
    management_key_source: &'a config::ValueSource,
}

fn snapshot_from_profile(profile: &ResolvedProfile) -> ConfigSnapshot<'_> {
    ConfigSnapshot {
        profile: &profile.profile,
        profile_source: &profile.profile_source,
        config_path: profile.config_path.display().to_string(),
        config_path_source: &profile.config_path_source,
        base_url: &profile.base_url,
        base_url_source: &profile.base_url_source,
        api_key_present: profile.api_key.is_some(),
        api_key_source: &profile.api_key_source,
        management_key_present: profile.management_key.is_some(),
        management_key_source: &profile.management_key_source,
    }
}

fn print_snapshot(snapshot: &ConfigSnapshot<'_>, output: OutputFormat) -> Result<()> {
    match output {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(snapshot)?);
        }
        OutputFormat::Text => {
            println!("profile: {}", snapshot.profile);
            println!("profile_source: {:?}", snapshot.profile_source);
            println!("config_path: {}", snapshot.config_path);
            println!("config_path_source: {:?}", snapshot.config_path_source);
            println!("base_url: {}", snapshot.base_url);
            println!("base_url_source: {:?}", snapshot.base_url_source);
            println!("api_key_present: {}", snapshot.api_key_present);
            println!("api_key_source: {:?}", snapshot.api_key_source);
            println!(
                "management_key_present: {}",
                snapshot.management_key_present
            );
            println!(
                "management_key_source: {:?}",
                snapshot.management_key_source
            );
        }
    }
    Ok(())
}

fn print_json<T: Serialize + ?Sized>(value: &T) -> Result<()> {
    println!("{}", serde_json::to_string_pretty(value)?);
    Ok(())
}

fn print_table(headers: &[&str], rows: &[Vec<String>]) {
    if rows.is_empty() {
        println!("(empty)");
        return;
    }

    let mut widths: Vec<usize> = headers.iter().map(|header| header.len()).collect();
    for row in rows {
        for (index, value) in row.iter().enumerate() {
            widths[index] = widths[index].max(value.len());
        }
    }

    let header_line = headers
        .iter()
        .enumerate()
        .map(|(index, value)| format!("{value:<width$}", width = widths[index]))
        .collect::<Vec<_>>()
        .join("  ");
    println!("{header_line}");
    println!(
        "{}",
        widths
            .iter()
            .map(|width| "-".repeat(*width))
            .collect::<Vec<_>>()
            .join("  ")
    );

    for row in rows {
        let line = row
            .iter()
            .enumerate()
            .map(|(index, value)| format!("{value:<width$}", width = widths[index]))
            .collect::<Vec<_>>()
            .join("  ");
        println!("{line}");
    }
}

fn model_category(value: ModelCategoryArg) -> ModelCategory {
    match value {
        ModelCategoryArg::Roleplay => ModelCategory::Roleplay,
        ModelCategoryArg::Programming => ModelCategory::Programming,
        ModelCategoryArg::Marketing => ModelCategory::Marketing,
        ModelCategoryArg::MarketingSeo => ModelCategory::MarketingSeo,
        ModelCategoryArg::Technology => ModelCategory::Technology,
        ModelCategoryArg::Science => ModelCategory::Science,
        ModelCategoryArg::Translation => ModelCategory::Translation,
        ModelCategoryArg::Legal => ModelCategory::Legal,
        ModelCategoryArg::Finance => ModelCategory::Finance,
        ModelCategoryArg::Health => ModelCategory::Health,
        ModelCategoryArg::Trivia => ModelCategory::Trivia,
        ModelCategoryArg::Academia => ModelCategory::Academia,
    }
}

fn supported_parameter(value: SupportedParameterArg) -> SupportedParameters {
    match value {
        SupportedParameterArg::Tools => SupportedParameters::Tools,
        SupportedParameterArg::Temperature => SupportedParameters::Temperature,
        SupportedParameterArg::TopP => SupportedParameters::TopP,
        SupportedParameterArg::TopK => SupportedParameters::TopK,
        SupportedParameterArg::MinP => SupportedParameters::MinP,
        SupportedParameterArg::TopA => SupportedParameters::TopA,
        SupportedParameterArg::FrequencyPenalty => SupportedParameters::FrequencyPenalty,
        SupportedParameterArg::PresencePenalty => SupportedParameters::PresencePenalty,
        SupportedParameterArg::RepetitionPenalty => SupportedParameters::RepetitionPenalty,
        SupportedParameterArg::MaxTokens => SupportedParameters::MaxTokens,
        SupportedParameterArg::LogitBias => SupportedParameters::LogitBias,
        SupportedParameterArg::Logprobs => SupportedParameters::Logprobs,
        SupportedParameterArg::TopLogprobs => SupportedParameters::TopLogprobs,
        SupportedParameterArg::Seed => SupportedParameters::Seed,
        SupportedParameterArg::ResponseFormat => SupportedParameters::ResponseFormat,
        SupportedParameterArg::StructuredOutputs => SupportedParameters::StructuredOutputs,
        SupportedParameterArg::Stop => SupportedParameters::Stop,
        SupportedParameterArg::IncludeReasoning => SupportedParameters::IncludeReasoning,
        SupportedParameterArg::Reasoning => SupportedParameters::Reasoning,
        SupportedParameterArg::WebSearchOptions => SupportedParameters::WebSearchOptions,
    }
}

fn parse_model_id(model_id: &str) -> Result<(&str, &str)> {
    let (author, slug) = model_id
        .split_once('/')
        .ok_or_else(|| anyhow!("model id must be in '<author>/<slug>' format"))?;
    Ok((author, slug))
}

fn build_api_client(resolved: &ResolvedProfile) -> Result<OpenRouterClient> {
    let Some(api_key) = resolved.api_key.as_deref() else {
        bail!("api key is required; set --api-key, OPENROUTER_API_KEY, or profile.api_key");
    };

    let mut builder = OpenRouterClient::builder();
    builder.base_url(resolved.base_url.clone());
    builder.api_key(api_key);
    if let Some(management_key) = resolved.management_key.as_deref() {
        builder.management_key(management_key);
    }
    Ok(builder.build()?)
}

fn print_models(models: &[models::Model], output: OutputFormat) -> Result<()> {
    match output {
        OutputFormat::Json => print_json(models)?,
        OutputFormat::Text => {
            let rows = models
                .iter()
                .map(|model| {
                    vec![
                        model.id.clone(),
                        model.name.clone(),
                        model.context_length.to_string(),
                        model.pricing.prompt.clone(),
                        model.pricing.completion.clone(),
                    ]
                })
                .collect::<Vec<_>>();
            print_table(
                &[
                    "id",
                    "name",
                    "context_length",
                    "prompt_price",
                    "completion_price",
                ],
                &rows,
            );
        }
    }
    Ok(())
}

fn print_model(model: &models::Model, output: OutputFormat) -> Result<()> {
    match output {
        OutputFormat::Json => print_json(model)?,
        OutputFormat::Text => {
            println!("id: {}", model.id);
            println!("name: {}", model.name);
            println!("created: {}", model.created);
            println!("context_length: {}", model.context_length);
            println!("description: {}", model.description);
            println!("modality: {}", model.architecture.modality);
            println!("tokenizer: {}", model.architecture.tokenizer);
            println!("prompt_price: {}", model.pricing.prompt);
            println!("completion_price: {}", model.pricing.completion);
        }
    }
    Ok(())
}

fn print_endpoints(response: &models::EndpointData, output: OutputFormat) -> Result<()> {
    match output {
        OutputFormat::Json => print_json(response)?,
        OutputFormat::Text => {
            let rows = response
                .endpoints
                .iter()
                .map(|endpoint| {
                    vec![
                        endpoint.name.clone(),
                        endpoint.provider_name.clone(),
                        endpoint.context_length.to_string(),
                        endpoint.pricing.prompt.clone(),
                        endpoint.pricing.completion.clone(),
                        endpoint
                            .status
                            .as_ref()
                            .map(ToString::to_string)
                            .unwrap_or_else(|| "-".to_string()),
                    ]
                })
                .collect::<Vec<_>>();
            print_table(
                &[
                    "name",
                    "provider",
                    "context_length",
                    "prompt_price",
                    "completion_price",
                    "status",
                ],
                &rows,
            );
        }
    }
    Ok(())
}

fn print_providers(providers: &[discovery::Provider], output: OutputFormat) -> Result<()> {
    match output {
        OutputFormat::Json => print_json(providers)?,
        OutputFormat::Text => {
            let rows = providers
                .iter()
                .map(|provider| {
                    vec![
                        provider.name.clone(),
                        provider.slug.clone(),
                        provider
                            .status_page_url
                            .clone()
                            .unwrap_or_else(|| "-".to_string()),
                    ]
                })
                .collect::<Vec<_>>();
            print_table(&["name", "slug", "status_page_url"], &rows);
        }
    }
    Ok(())
}

async fn run() -> Result<()> {
    let cli = Cli::parse();
    let env = Environment::from_process();
    let resolved = resolve_profile(&cli.global, &env)?;

    match cli.command {
        Commands::Profile { command } => match command {
            ProfileCommands::Show => {
                let snapshot = snapshot_from_profile(&resolved);
                print_snapshot(&snapshot, cli.global.output)?;
            }
        },
        Commands::Config { command } => match command {
            ConfigCommands::Show => {
                let snapshot = snapshot_from_profile(&resolved);
                print_snapshot(&snapshot, cli.global.output)?;
            }
            ConfigCommands::Path => match cli.global.output {
                OutputFormat::Json => {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&serde_json::json!({
                            "config_path": resolved.config_path.display().to_string()
                        }))?
                    );
                }
                OutputFormat::Text => {
                    println!("{}", resolved.config_path.display());
                }
            },
        },
        Commands::Models { command } => {
            let client = build_api_client(&resolved)?;
            let models_client = client.models();
            match command {
                ModelsCommands::List(args) => {
                    let response = if let Some(category) = args.category {
                        models_client
                            .list_by_category(model_category(category))
                            .await?
                    } else if let Some(parameter) = args.supported_parameter {
                        models_client
                            .list_by_parameters(supported_parameter(parameter))
                            .await?
                    } else {
                        models_client.list().await?
                    };
                    print_models(&response, cli.global.output)?;
                }
                ModelsCommands::Show(args) => {
                    let models = models_client.list().await?;
                    let model = models
                        .iter()
                        .find(|model| model.id == args.model_id)
                        .ok_or_else(|| anyhow!("model not found: {}", args.model_id))?;
                    print_model(model, cli.global.output)?;
                }
                ModelsCommands::Endpoints(args) => {
                    let (author, slug) = parse_model_id(&args.model_id)?;
                    let response = models_client.list_endpoints(author, slug).await?;
                    print_endpoints(&response, cli.global.output)?;
                }
            }
        }
        Commands::Providers { command } => {
            let client = build_api_client(&resolved)?;
            let models_client = client.models();
            match command {
                ProvidersCommands::List => {
                    let providers = models_client.list_providers().await?;
                    print_providers(&providers, cli.global.output)?;
                }
            }
        }
        Commands::Keys => {
            println!("keys command group is not implemented yet (planned in OR-21)");
        }
        Commands::Guardrails => {
            println!("guardrails command group is not implemented yet (planned in OR-21)");
        }
        Commands::Usage => {
            println!("usage command group is not implemented yet (planned in OR-22)");
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() {
    if let Err(error) = run().await {
        eprintln!("error: {error:#}");
        std::process::exit(1);
    }
}
