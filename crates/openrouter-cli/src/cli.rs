use std::path::PathBuf;

use clap::{Args, Parser, Subcommand, ValueEnum};

#[derive(Debug, Clone, Copy, Eq, PartialEq, ValueEnum)]
pub enum OutputFormat {
    Text,
    Json,
}

#[derive(Debug, Clone, Args)]
pub struct GlobalOptions {
    /// Path to CLI profile config TOML.
    #[arg(long, global = true)]
    pub config: Option<PathBuf>,

    /// Profile name to use.
    #[arg(long, global = true)]
    pub profile: Option<String>,

    /// Override API key (highest priority).
    #[arg(long, global = true)]
    pub api_key: Option<String>,

    /// Override management key (highest priority).
    #[arg(long, global = true)]
    pub management_key: Option<String>,

    /// Override OpenRouter API base URL.
    #[arg(long, global = true)]
    pub base_url: Option<String>,

    /// Output format.
    #[arg(long, global = true, value_enum, default_value_t = OutputFormat::Text)]
    pub output: OutputFormat,
}

#[derive(Debug, Clone, Subcommand)]
pub enum ProfileCommands {
    /// Show resolved profile and auth sources.
    Show,
}

#[derive(Debug, Clone, Subcommand)]
pub enum ConfigCommands {
    /// Show resolved configuration snapshot.
    Show,
    /// Print the resolved config file path.
    Path,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, ValueEnum)]
pub enum ModelCategoryArg {
    Roleplay,
    Programming,
    Marketing,
    #[value(name = "marketing/seo", alias = "marketing-seo")]
    MarketingSeo,
    Technology,
    Science,
    Translation,
    Legal,
    Finance,
    Health,
    Trivia,
    Academia,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, ValueEnum)]
pub enum SupportedParameterArg {
    #[value(name = "tools")]
    Tools,
    #[value(name = "temperature")]
    Temperature,
    #[value(name = "top_p")]
    TopP,
    #[value(name = "top_k")]
    TopK,
    #[value(name = "min_p")]
    MinP,
    #[value(name = "top_a")]
    TopA,
    #[value(name = "frequency_penalty")]
    FrequencyPenalty,
    #[value(name = "presence_penalty")]
    PresencePenalty,
    #[value(name = "repetition_penalty")]
    RepetitionPenalty,
    #[value(name = "max_tokens")]
    MaxTokens,
    #[value(name = "logit_bias")]
    LogitBias,
    #[value(name = "logprobs")]
    Logprobs,
    #[value(name = "top_logprobs")]
    TopLogprobs,
    #[value(name = "seed")]
    Seed,
    #[value(name = "response_format")]
    ResponseFormat,
    #[value(name = "structured_outputs")]
    StructuredOutputs,
    #[value(name = "stop")]
    Stop,
    #[value(name = "include_reasoning")]
    IncludeReasoning,
    #[value(name = "reasoning")]
    Reasoning,
    #[value(name = "web_search_options")]
    WebSearchOptions,
}

#[derive(Debug, Clone, Args)]
pub struct ModelsListArgs {
    /// Filter models by category.
    #[arg(long, value_enum, conflicts_with = "supported_parameter")]
    pub category: Option<ModelCategoryArg>,

    /// Filter models by supported parameter.
    #[arg(long, value_enum, conflicts_with = "category")]
    pub supported_parameter: Option<SupportedParameterArg>,
}

#[derive(Debug, Clone, Args)]
pub struct ModelsShowArgs {
    /// Model ID (for example: openai/gpt-4.1).
    pub model_id: String,
}

#[derive(Debug, Clone, Args)]
pub struct ModelsEndpointsArgs {
    /// Model ID (for example: openai/gpt-4.1).
    pub model_id: String,
}

#[derive(Debug, Clone, Subcommand)]
pub enum ModelsCommands {
    /// List models.
    List(ModelsListArgs),
    /// Show a single model by model ID.
    Show(ModelsShowArgs),
    /// List endpoints for a specific model.
    Endpoints(ModelsEndpointsArgs),
}

#[derive(Debug, Clone, Subcommand)]
pub enum ProvidersCommands {
    /// List providers.
    List,
}

#[derive(Debug, Clone, Subcommand)]
pub enum Commands {
    /// Profile-related commands.
    Profile {
        #[command(subcommand)]
        command: ProfileCommands,
    },
    /// Configuration commands.
    Config {
        #[command(subcommand)]
        command: ConfigCommands,
    },
    /// Model discovery commands.
    Models {
        #[command(subcommand)]
        command: ModelsCommands,
    },
    /// Provider discovery commands.
    Providers {
        #[command(subcommand)]
        command: ProvidersCommands,
    },
    /// Management command group placeholder (planned in OR-21).
    Keys,
    /// Guardrail command group placeholder (planned in OR-21).
    Guardrails,
    /// Usage/billing command group placeholder (planned in OR-22).
    Usage,
}

#[derive(Debug, Clone, Parser)]
#[command(name = "openrouter-cli", version, about = "OpenRouter CLI", long_about = None)]
pub struct Cli {
    #[command(flatten)]
    pub global: GlobalOptions,

    #[command(subcommand)]
    pub command: Commands,
}
