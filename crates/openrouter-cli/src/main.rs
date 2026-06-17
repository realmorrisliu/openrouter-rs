mod cli;
mod config;

use anyhow::{Result, anyhow, bail};
use clap::{Parser, error::ErrorKind};
use openrouter_rs::{
    OpenRouterClient,
    api::{credits, discovery, guardrails, models, workspaces},
    types::{ModelCategory, PaginationOptions, SupportedParameters},
};
use serde::Serialize;

use crate::{
    cli::{
        Cli, Commands, ConfigCommands, CreditsCommands, GuardrailAssignmentCommands,
        GuardrailKeyAssignmentCommands, GuardrailMemberAssignmentCommands, GuardrailsCommands,
        KeysCommands, ModelCategoryArg, ModelsCommands, OrganizationCommands,
        OrganizationMemberCommands, OutputFormat, PaginationArgs, ProfileCommands,
        ProvidersCommands, SupportedParameterArg, UsageCommands, WorkspaceMemberCommands,
        WorkspacesCommands,
    },
    config::{Environment, ResolvedProfile, resolve_profile},
};

const OUTPUT_SCHEMA_VERSION: &str = "0.1";

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

#[derive(Debug, Serialize)]
struct JsonEnvelope<'a, T: Serialize + ?Sized> {
    schema_version: &'static str,
    data: &'a T,
}

#[derive(Debug, Serialize)]
struct JsonErrorBody<'a> {
    code: &'static str,
    message: &'a str,
}

#[derive(Debug, Serialize)]
struct JsonErrorEnvelope<'a> {
    schema_version: &'static str,
    error: JsonErrorBody<'a>,
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
        OutputFormat::Json => print_json(snapshot)?,
        OutputFormat::Table => {
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
    let envelope = JsonEnvelope {
        schema_version: OUTPUT_SCHEMA_VERSION,
        data: value,
    };
    println!("{}", serde_json::to_string_pretty(&envelope)?);
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

fn print_value<T: Serialize>(value: &T, output: OutputFormat) -> Result<()> {
    match output {
        OutputFormat::Json => {
            print_json(value)?;
        }
        OutputFormat::Table => {
            println!("{}", serde_json::to_string_pretty(value)?);
        }
    }
    Ok(())
}

fn require_yes(yes: bool, action: &str) -> Result<()> {
    if yes {
        return Ok(());
    }
    bail!("refusing to {action} without --yes");
}

fn pagination_from_args(args: &PaginationArgs) -> Option<PaginationOptions> {
    match (args.offset, args.limit) {
        (None, None) => None,
        (offset, limit) => Some(PaginationOptions::new(offset, limit)),
    }
}

fn build_management_client(resolved: &ResolvedProfile) -> Result<OpenRouterClient> {
    let Some(management_key) = resolved.management_key.as_deref() else {
        bail!(
            "management key is required for this command; set --management-key, OPENROUTER_MANAGEMENT_KEY, or profile.management_key"
        );
    };

    let mut builder = OpenRouterClient::builder();
    builder.base_url(resolved.base_url.clone());
    builder.management_key(management_key);
    if let Some(api_key) = resolved.api_key.as_deref() {
        builder.api_key(api_key);
    }
    Ok(builder.build()?)
}

fn resolve_disabled_flag(disable: bool, enable: bool) -> Option<bool> {
    if disable {
        Some(true)
    } else if enable {
        Some(false)
    } else {
        None
    }
}

fn resolve_enabled_flag(enable: bool, disable: bool) -> Option<bool> {
    if enable {
        Some(true)
    } else if disable {
        Some(false)
    } else {
        None
    }
}

fn resolve_enforce_zdr_flag(enforce_zdr: bool, no_enforce_zdr: bool) -> Option<bool> {
    if enforce_zdr {
        Some(true)
    } else if no_enforce_zdr {
        Some(false)
    } else {
        None
    }
}

fn display_optional<T: ToString>(value: Option<T>) -> String {
    value
        .map(|value| value.to_string())
        .unwrap_or_else(|| "-".to_string())
}

fn print_models(models: &[models::Model], output: OutputFormat) -> Result<()> {
    match output {
        OutputFormat::Json => print_json(models)?,
        OutputFormat::Table => {
            let rows = models
                .iter()
                .map(|model| {
                    vec![
                        model.id.clone(),
                        model.name.clone(),
                        display_optional(model.context_length),
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
        OutputFormat::Table => {
            println!("id: {}", model.id);
            println!("name: {}", model.name);
            println!("created: {}", model.created);
            println!("context_length: {}", display_optional(model.context_length));
            println!("description: {}", model.description);
            println!(
                "modality: {}",
                display_optional(model.architecture.modality.as_deref())
            );
            println!(
                "tokenizer: {}",
                display_optional(model.architecture.tokenizer.as_deref())
            );
            println!("prompt_price: {}", model.pricing.prompt);
            println!("completion_price: {}", model.pricing.completion);
        }
    }
    Ok(())
}

fn print_endpoints(response: &models::EndpointData, output: OutputFormat) -> Result<()> {
    match output {
        OutputFormat::Json => print_json(response)?,
        OutputFormat::Table => {
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
        OutputFormat::Table => {
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

fn print_credits_table(credits: &credits::CreditsData) {
    print_table(
        &["total_credits", "total_usage"],
        &[vec![
            credits.total_credits.to_string(),
            credits.total_usage.to_string(),
        ]],
    );
}

fn print_charge_table(charge: &credits::CoinbaseChargeData) {
    print_table(
        &[
            "id",
            "chain_id",
            "sender",
            "address_count",
            "calldata_fields",
        ],
        &[vec![
            charge.id.clone().unwrap_or_else(|| "-".to_string()),
            charge.chain_id.to_string(),
            charge.sender.clone(),
            charge.addresses.len().to_string(),
            charge.calldata.len().to_string(),
        ]],
    );
}

fn print_activity_table(activity: &[openrouter_rs::api::discovery::ActivityItem]) {
    let rows = activity
        .iter()
        .map(|item| {
            vec![
                item.date.clone(),
                item.model.clone(),
                item.provider_name.clone(),
                item.usage.to_string(),
                item.requests.to_string(),
                item.prompt_tokens.to_string(),
                item.completion_tokens.to_string(),
                item.reasoning_tokens.to_string(),
            ]
        })
        .collect::<Vec<_>>();

    print_table(
        &[
            "date",
            "model",
            "provider",
            "usage",
            "requests",
            "prompt_tokens",
            "completion_tokens",
            "reasoning_tokens",
        ],
        &rows,
    );
}

async fn run(cli: Cli) -> Result<()> {
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
                    print_json(&serde_json::json!({
                        "config_path": resolved.config_path.display().to_string()
                    }))?;
                }
                OutputFormat::Table => {
                    println!("{}", resolved.config_path.display());
                }
            },
        },
        Commands::Credits { command } => {
            let client = build_api_client(&resolved)?;
            let management = client.management();

            match command {
                CreditsCommands::Show => {
                    let credits = management.get_credits().await?;
                    match cli.global.output {
                        OutputFormat::Json => print_json(&credits)?,
                        OutputFormat::Table => print_credits_table(&credits),
                    }
                }
                CreditsCommands::Charge(args) => {
                    let request = credits::CoinbaseChargeRequest::builder()
                        .amount(args.amount)
                        .sender(args.sender)
                        .chain_id(args.chain_id)
                        .build()?;
                    let charge = management.create_coinbase_charge(&request).await?;
                    match cli.global.output {
                        OutputFormat::Json => print_json(&charge)?,
                        OutputFormat::Table => print_charge_table(&charge),
                    }
                }
            }
        }
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
        Commands::Keys { command } => {
            let client = build_management_client(&resolved)?;
            let management = client.management();

            match command {
                KeysCommands::List(args) => {
                    let pagination = args.offset.map(PaginationOptions::with_offset);
                    let include_disabled = if args.include_disabled {
                        Some(true)
                    } else {
                        None
                    };
                    let response = management
                        .list_api_keys_in_workspace(
                            pagination,
                            include_disabled,
                            args.workspace_id.as_deref(),
                        )
                        .await?;
                    print_value(&response, cli.global.output)?;
                }
                KeysCommands::Create(args) => {
                    let response = management
                        .create_api_key_in_workspace(
                            &args.name,
                            args.limit,
                            args.workspace_id.as_deref(),
                        )
                        .await?;
                    print_value(&response, cli.global.output)?;
                }
                KeysCommands::Get(args) => {
                    let response = management.get_api_key(&args.hash).await?;
                    print_value(&response, cli.global.output)?;
                }
                KeysCommands::Update(args) => {
                    let disabled = resolve_disabled_flag(args.disable, args.enable);
                    if args.name.is_none() && args.limit.is_none() && disabled.is_none() {
                        bail!(
                            "no update fields provided; use --name, --limit, --enable, or --disable"
                        );
                    }

                    let response = management
                        .update_api_key(&args.hash, args.name, disabled, args.limit)
                        .await?;
                    print_value(&response, cli.global.output)?;
                }
                KeysCommands::Delete(args) => {
                    require_yes(args.yes, "delete key")?;
                    let deleted = management.delete_api_key(&args.hash).await?;
                    print_value(
                        &serde_json::json!({
                            "hash": args.hash,
                            "deleted": deleted,
                        }),
                        cli.global.output,
                    )?;
                }
            }
        }
        Commands::Guardrails { command } => {
            let client = build_management_client(&resolved)?;
            let management = client.management();

            match command {
                GuardrailsCommands::List(args) => {
                    let pagination = pagination_from_args(&args.pagination);
                    let response = management
                        .list_guardrails_in_workspace(pagination, args.workspace_id.as_deref())
                        .await?;
                    print_value(&response, cli.global.output)?;
                }
                GuardrailsCommands::Create(args) => {
                    let mut builder = guardrails::CreateGuardrailRequest::builder();
                    builder.name(args.name);

                    if let Some(description) = args.description {
                        builder.description(description);
                    }
                    if let Some(limit_usd) = args.limit_usd {
                        builder.limit_usd(limit_usd);
                    }
                    if let Some(reset_interval) = args.reset_interval {
                        builder.reset_interval(reset_interval);
                    }
                    if !args.allowed_providers.is_empty() {
                        builder.allowed_providers(args.allowed_providers);
                    }
                    if !args.allowed_models.is_empty() {
                        builder.allowed_models(args.allowed_models);
                    }
                    if args.enforce_zdr {
                        builder.enforce_zdr(true);
                    }
                    if let Some(workspace_id) = args.workspace_id {
                        builder.workspace_id(workspace_id);
                    }

                    let request = builder.build()?;
                    let response = management.create_guardrail(&request).await?;
                    print_value(&response, cli.global.output)?;
                }
                GuardrailsCommands::Get(args) => {
                    let response = management.get_guardrail(&args.id).await?;
                    print_value(&response, cli.global.output)?;
                }
                GuardrailsCommands::Update(args) => {
                    let enforce_zdr =
                        resolve_enforce_zdr_flag(args.enforce_zdr, args.no_enforce_zdr);

                    if args.name.is_none()
                        && args.description.is_none()
                        && args.limit_usd.is_none()
                        && args.reset_interval.is_none()
                        && args.allowed_providers.is_empty()
                        && args.allowed_models.is_empty()
                        && !args.clear_allowed_providers
                        && !args.clear_allowed_models
                        && enforce_zdr.is_none()
                    {
                        bail!("no update fields provided; pass at least one update argument");
                    }

                    let mut builder = guardrails::UpdateGuardrailRequest::builder();
                    if let Some(name) = args.name {
                        builder.name(name);
                    }
                    if let Some(description) = args.description {
                        builder.description(description);
                    }
                    if let Some(limit_usd) = args.limit_usd {
                        builder.limit_usd(limit_usd);
                    }
                    if let Some(reset_interval) = args.reset_interval {
                        builder.reset_interval(reset_interval);
                    }
                    if args.clear_allowed_providers {
                        builder.clear_allowed_providers();
                    } else if !args.allowed_providers.is_empty() {
                        builder.allowed_providers(args.allowed_providers);
                    }
                    if args.clear_allowed_models {
                        builder.clear_allowed_models();
                    } else if !args.allowed_models.is_empty() {
                        builder.allowed_models(args.allowed_models);
                    }
                    if let Some(enforce_zdr) = enforce_zdr {
                        builder.enforce_zdr(enforce_zdr);
                    }

                    let request = builder.build()?;
                    let response = management.update_guardrail(&args.id, &request).await?;
                    print_value(&response, cli.global.output)?;
                }
                GuardrailsCommands::Delete(args) => {
                    require_yes(args.yes, "delete guardrail")?;
                    let deleted = management.delete_guardrail(&args.id).await?;
                    print_value(
                        &serde_json::json!({
                            "id": args.id,
                            "deleted": deleted,
                        }),
                        cli.global.output,
                    )?;
                }
                GuardrailsCommands::Assignments { command } => match command {
                    GuardrailAssignmentCommands::Keys { command } => match command {
                        GuardrailKeyAssignmentCommands::List(args) => {
                            let pagination = pagination_from_args(&args.pagination);
                            if let Some(guardrail_id) = args.guardrail_id {
                                let response = management
                                    .list_guardrail_key_assignments(&guardrail_id, pagination)
                                    .await?;
                                print_value(&response, cli.global.output)?;
                            } else {
                                let response = management.list_key_assignments(pagination).await?;
                                print_value(&response, cli.global.output)?;
                            }
                        }
                        GuardrailKeyAssignmentCommands::Assign(args) => {
                            let request = guardrails::BulkKeyAssignmentRequest::builder()
                                .key_hashes(args.key_hashes)
                                .build()?;
                            let response = management
                                .create_guardrail_key_assignments(&args.guardrail_id, &request)
                                .await?;
                            print_value(&response, cli.global.output)?;
                        }
                        GuardrailKeyAssignmentCommands::Unassign(args) => {
                            require_yes(args.yes, "unassign keys from guardrail")?;
                            let request = guardrails::BulkKeyAssignmentRequest::builder()
                                .key_hashes(args.request.key_hashes)
                                .build()?;
                            let response = management
                                .delete_guardrail_key_assignments(
                                    &args.request.guardrail_id,
                                    &request,
                                )
                                .await?;
                            print_value(&response, cli.global.output)?;
                        }
                    },
                    GuardrailAssignmentCommands::Members { command } => match command {
                        GuardrailMemberAssignmentCommands::List(args) => {
                            let pagination = pagination_from_args(&args.pagination);
                            if let Some(guardrail_id) = args.guardrail_id {
                                let response = management
                                    .list_guardrail_member_assignments(&guardrail_id, pagination)
                                    .await?;
                                print_value(&response, cli.global.output)?;
                            } else {
                                let response =
                                    management.list_member_assignments(pagination).await?;
                                print_value(&response, cli.global.output)?;
                            }
                        }
                        GuardrailMemberAssignmentCommands::Assign(args) => {
                            let request = guardrails::BulkMemberAssignmentRequest::builder()
                                .member_user_ids(args.member_user_ids)
                                .build()?;
                            let response = management
                                .create_guardrail_member_assignments(&args.guardrail_id, &request)
                                .await?;
                            print_value(&response, cli.global.output)?;
                        }
                        GuardrailMemberAssignmentCommands::Unassign(args) => {
                            require_yes(args.yes, "unassign members from guardrail")?;
                            let request = guardrails::BulkMemberAssignmentRequest::builder()
                                .member_user_ids(args.request.member_user_ids)
                                .build()?;
                            let response = management
                                .delete_guardrail_member_assignments(
                                    &args.request.guardrail_id,
                                    &request,
                                )
                                .await?;
                            print_value(&response, cli.global.output)?;
                        }
                    },
                },
            }
        }
        Commands::Organization { command } => {
            let client = build_management_client(&resolved)?;
            let management = client.management();

            match command {
                OrganizationCommands::Members { command } => match command {
                    OrganizationMemberCommands::List(args) => {
                        let pagination = pagination_from_args(&args);
                        let response = management.list_organization_members(pagination).await?;
                        print_value(&response, cli.global.output)?;
                    }
                },
            }
        }
        Commands::Workspaces { command } => {
            let client = build_management_client(&resolved)?;
            let management = client.management();

            match command {
                WorkspacesCommands::List(args) => {
                    let pagination = pagination_from_args(&args);
                    let response = management.list_workspaces(pagination).await?;
                    print_value(&response, cli.global.output)?;
                }
                WorkspacesCommands::Create(args) => {
                    let data_discount_logging = resolve_enabled_flag(
                        args.enable_data_discount_logging,
                        args.disable_data_discount_logging,
                    );
                    let observability_broadcast = resolve_enabled_flag(
                        args.enable_observability_broadcast,
                        args.disable_observability_broadcast,
                    );
                    let observability_io_logging = resolve_enabled_flag(
                        args.enable_observability_io_logging,
                        args.disable_observability_io_logging,
                    );

                    let mut builder = workspaces::CreateWorkspaceRequest::builder();
                    builder.name(args.name);

                    if let Some(slug) = args.slug {
                        builder.slug(slug);
                    }
                    if let Some(description) = args.description {
                        builder.description(description);
                    }
                    if let Some(default_text_model) = args.default_text_model {
                        builder.default_text_model(default_text_model);
                    }
                    if let Some(default_image_model) = args.default_image_model {
                        builder.default_image_model(default_image_model);
                    }
                    if let Some(default_provider_sort) = args.default_provider_sort {
                        builder.default_provider_sort(default_provider_sort);
                    }
                    if !args.io_logging_api_key_ids.is_empty() {
                        builder.io_logging_api_key_ids(args.io_logging_api_key_ids);
                    }
                    if let Some(io_logging_sampling_rate) = args.io_logging_sampling_rate {
                        builder.io_logging_sampling_rate(io_logging_sampling_rate);
                    }
                    if let Some(enabled) = data_discount_logging {
                        builder.is_data_discount_logging_enabled(enabled);
                    }
                    if let Some(enabled) = observability_broadcast {
                        builder.is_observability_broadcast_enabled(enabled);
                    }
                    if let Some(enabled) = observability_io_logging {
                        builder.is_observability_io_logging_enabled(enabled);
                    }

                    let request = builder.build()?;
                    let response = management.create_workspace(&request).await?;
                    print_value(&response, cli.global.output)?;
                }
                WorkspacesCommands::Get(args) => {
                    let response = management.get_workspace(&args.id).await?;
                    print_value(&response, cli.global.output)?;
                }
                WorkspacesCommands::Update(args) => {
                    let data_discount_logging = resolve_enabled_flag(
                        args.enable_data_discount_logging,
                        args.disable_data_discount_logging,
                    );
                    let observability_broadcast = resolve_enabled_flag(
                        args.enable_observability_broadcast,
                        args.disable_observability_broadcast,
                    );
                    let observability_io_logging = resolve_enabled_flag(
                        args.enable_observability_io_logging,
                        args.disable_observability_io_logging,
                    );

                    if args.name.is_none()
                        && args.slug.is_none()
                        && args.description.is_none()
                        && args.default_text_model.is_none()
                        && args.default_image_model.is_none()
                        && args.default_provider_sort.is_none()
                        && args.io_logging_api_key_ids.is_empty()
                        && !args.clear_io_logging_api_key_ids
                        && args.io_logging_sampling_rate.is_none()
                        && data_discount_logging.is_none()
                        && observability_broadcast.is_none()
                        && observability_io_logging.is_none()
                    {
                        bail!("no update fields provided; pass at least one update argument");
                    }

                    let mut builder = workspaces::UpdateWorkspaceRequest::builder();
                    if let Some(name) = args.name {
                        builder.name(name);
                    }
                    if let Some(slug) = args.slug {
                        builder.slug(slug);
                    }
                    if let Some(description) = args.description {
                        builder.description(description);
                    }
                    if let Some(default_text_model) = args.default_text_model {
                        builder.default_text_model(default_text_model);
                    }
                    if let Some(default_image_model) = args.default_image_model {
                        builder.default_image_model(default_image_model);
                    }
                    if let Some(default_provider_sort) = args.default_provider_sort {
                        builder.default_provider_sort(default_provider_sort);
                    }
                    if !args.io_logging_api_key_ids.is_empty() {
                        builder.io_logging_api_key_ids(args.io_logging_api_key_ids);
                    }
                    if let Some(io_logging_sampling_rate) = args.io_logging_sampling_rate {
                        builder.io_logging_sampling_rate(io_logging_sampling_rate);
                    }
                    if let Some(enabled) = data_discount_logging {
                        builder.is_data_discount_logging_enabled(enabled);
                    }
                    if let Some(enabled) = observability_broadcast {
                        builder.is_observability_broadcast_enabled(enabled);
                    }
                    if let Some(enabled) = observability_io_logging {
                        builder.is_observability_io_logging_enabled(enabled);
                    }

                    let request = builder.build()?;
                    let response = if args.clear_io_logging_api_key_ids {
                        management
                            .update_workspace_with_cleared_io_logging_api_key_ids(
                                &args.id, &request,
                            )
                            .await?
                    } else {
                        management.update_workspace(&args.id, &request).await?
                    };
                    print_value(&response, cli.global.output)?;
                }
                WorkspacesCommands::Delete(args) => {
                    require_yes(args.yes, "delete workspace")?;
                    let deleted = management.delete_workspace(&args.id).await?;
                    print_value(
                        &serde_json::json!({
                            "id": args.id,
                            "deleted": deleted,
                        }),
                        cli.global.output,
                    )?;
                }
                WorkspacesCommands::Members { command } => match command {
                    WorkspaceMemberCommands::Add(args) => {
                        let workspace_id = args.workspace_id;
                        let user_ids = args.user_ids;
                        let request = workspaces::WorkspaceMembersRequest::builder()
                            .user_ids(user_ids)
                            .build()?;
                        let response = management
                            .add_workspace_members(&workspace_id, &request)
                            .await?;
                        print_value(&response, cli.global.output)?;
                    }
                    WorkspaceMemberCommands::Remove(args) => {
                        require_yes(args.yes, "remove members from workspace")?;
                        let workspace_id = args.request.workspace_id;
                        let user_ids = args.request.user_ids;
                        let request = workspaces::WorkspaceMembersRequest::builder()
                            .user_ids(user_ids)
                            .build()?;
                        let response = management
                            .remove_workspace_members(&workspace_id, &request)
                            .await?;
                        print_value(&response, cli.global.output)?;
                    }
                },
            }
        }
        Commands::Usage { command } => {
            let client = build_management_client(&resolved)?;
            let management = client.management();

            match command {
                UsageCommands::Activity(args) => {
                    let activity = management.get_activity(args.date.as_deref()).await?;
                    match cli.global.output {
                        OutputFormat::Json => print_json(&activity)?,
                        OutputFormat::Table => print_activity_table(&activity),
                    }
                }
            }
        }
    }

    Ok(())
}

fn detect_output_from_args() -> OutputFormat {
    let mut args = std::env::args().skip(1);

    while let Some(arg) = args.next() {
        if arg == "--output" {
            if let Some(value) = args.next() {
                return match value.as_str() {
                    "json" => OutputFormat::Json,
                    "table" | "text" => OutputFormat::Table,
                    _ => OutputFormat::Table,
                };
            }
        }

        if let Some(value) = arg.strip_prefix("--output=") {
            return match value {
                "json" => OutputFormat::Json,
                "table" | "text" => OutputFormat::Table,
                _ => OutputFormat::Table,
            };
        }
    }

    OutputFormat::Table
}

fn print_error(error: &anyhow::Error, output: OutputFormat) {
    match output {
        OutputFormat::Table => {
            eprintln!("error: {error:#}");
        }
        OutputFormat::Json => {
            let body = JsonErrorEnvelope {
                schema_version: OUTPUT_SCHEMA_VERSION,
                error: JsonErrorBody {
                    code: "cli_error",
                    message: &format!("{error:#}"),
                },
            };
            if let Ok(payload) = serde_json::to_string_pretty(&body) {
                eprintln!("{payload}");
            } else {
                eprintln!(
                    "{{\"schema_version\":\"0.1\",\"error\":{{\"code\":\"cli_error\",\"message\":\"serialization failure\"}}}}"
                );
            }
        }
    }
}

#[tokio::main]
async fn main() {
    let requested_output = detect_output_from_args();

    let cli = match Cli::try_parse() {
        Ok(cli) => cli,
        Err(error) => {
            let exit_code = error.exit_code();
            if matches!(
                error.kind(),
                ErrorKind::DisplayHelp | ErrorKind::DisplayVersion
            ) {
                if let Err(print_error) = error.print() {
                    eprintln!("error: failed to print clap output: {print_error:#}");
                }
            } else {
                print_error(&anyhow!(error.to_string()), requested_output);
            }
            std::process::exit(exit_code);
        }
    };

    let output = cli.global.output;

    if let Err(error) = run(cli).await {
        print_error(&error, output);
        std::process::exit(1);
    }
}
