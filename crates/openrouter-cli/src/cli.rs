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

#[derive(Debug, Clone, Args)]
pub struct KeysListArgs {
    /// Optional offset for key listing.
    #[arg(long)]
    pub offset: Option<u32>,

    /// Include disabled keys.
    #[arg(long)]
    pub include_disabled: bool,
}

#[derive(Debug, Clone, Args)]
pub struct KeysCreateArgs {
    /// Display name for the key.
    #[arg(long)]
    pub name: String,

    /// Optional spending limit in USD.
    #[arg(long)]
    pub limit: Option<f64>,
}

#[derive(Debug, Clone, Args)]
pub struct KeysGetArgs {
    /// Key hash.
    pub hash: String,
}

#[derive(Debug, Clone, Args)]
pub struct KeysUpdateArgs {
    /// Key hash.
    pub hash: String,

    /// Optional new display name.
    #[arg(long)]
    pub name: Option<String>,

    /// Optional new spending limit in USD.
    #[arg(long)]
    pub limit: Option<f64>,

    /// Disable this key.
    #[arg(long, conflicts_with = "enable")]
    pub disable: bool,

    /// Enable this key.
    #[arg(long, conflicts_with = "disable")]
    pub enable: bool,
}

#[derive(Debug, Clone, Args)]
pub struct KeysDeleteArgs {
    /// Key hash.
    pub hash: String,

    /// Confirm destructive action.
    #[arg(long)]
    pub yes: bool,
}

#[derive(Debug, Clone, Subcommand)]
pub enum KeysCommands {
    /// List API keys.
    List(KeysListArgs),
    /// Create an API key.
    Create(KeysCreateArgs),
    /// Get a single API key.
    Get(KeysGetArgs),
    /// Update an API key.
    Update(KeysUpdateArgs),
    /// Delete an API key.
    Delete(KeysDeleteArgs),
}

#[derive(Debug, Clone, Args)]
pub struct PaginationArgs {
    /// Offset for pagination.
    #[arg(long)]
    pub offset: Option<u32>,

    /// Limit for pagination.
    #[arg(long)]
    pub limit: Option<u32>,
}

#[derive(Debug, Clone, Args)]
pub struct GuardrailsGetArgs {
    /// Guardrail ID.
    pub id: String,
}

#[derive(Debug, Clone, Args)]
pub struct GuardrailsDeleteArgs {
    /// Guardrail ID.
    pub id: String,

    /// Confirm destructive action.
    #[arg(long)]
    pub yes: bool,
}

#[derive(Debug, Clone, Args)]
pub struct GuardrailsCreateArgs {
    /// Guardrail name.
    #[arg(long)]
    pub name: String,

    /// Optional guardrail description.
    #[arg(long)]
    pub description: Option<String>,

    /// Optional spending cap in USD.
    #[arg(long)]
    pub limit_usd: Option<f64>,

    /// Optional reset interval (for example: daily, monthly).
    #[arg(long)]
    pub reset_interval: Option<String>,

    /// Allowed provider IDs.
    #[arg(long = "allowed-provider")]
    pub allowed_providers: Vec<String>,

    /// Allowed model IDs.
    #[arg(long = "allowed-model")]
    pub allowed_models: Vec<String>,

    /// Enforce ZDR.
    #[arg(long)]
    pub enforce_zdr: bool,
}

#[derive(Debug, Clone, Args)]
pub struct GuardrailsUpdateArgs {
    /// Guardrail ID.
    pub id: String,

    /// Optional new name.
    #[arg(long)]
    pub name: Option<String>,

    /// Optional new description.
    #[arg(long)]
    pub description: Option<String>,

    /// Optional new spending cap in USD.
    #[arg(long)]
    pub limit_usd: Option<f64>,

    /// Optional new reset interval.
    #[arg(long)]
    pub reset_interval: Option<String>,

    /// Replace allowed provider IDs.
    #[arg(long = "allowed-provider", conflicts_with = "clear_allowed_providers")]
    pub allowed_providers: Vec<String>,

    /// Replace allowed model IDs.
    #[arg(long = "allowed-model", conflicts_with = "clear_allowed_models")]
    pub allowed_models: Vec<String>,

    /// Clear all allowed providers (send empty list).
    #[arg(long)]
    pub clear_allowed_providers: bool,

    /// Clear all allowed models (send empty list).
    #[arg(long)]
    pub clear_allowed_models: bool,

    /// Set `enforce_zdr=true`.
    #[arg(long, conflicts_with = "no_enforce_zdr")]
    pub enforce_zdr: bool,

    /// Set `enforce_zdr=false`.
    #[arg(long = "no-enforce-zdr", conflicts_with = "enforce_zdr")]
    pub no_enforce_zdr: bool,
}

#[derive(Debug, Clone, Args)]
pub struct AssignmentListArgs {
    /// Optional guardrail ID. If omitted, lists global assignments.
    #[arg(long)]
    pub guardrail_id: Option<String>,

    #[command(flatten)]
    pub pagination: PaginationArgs,
}

#[derive(Debug, Clone, Args)]
pub struct KeyAssignmentApplyArgs {
    /// Guardrail ID.
    pub guardrail_id: String,

    /// One or more key hashes.
    #[arg(value_name = "KEY_HASH", required = true, num_args = 1..)]
    pub key_hashes: Vec<String>,
}

#[derive(Debug, Clone, Args)]
pub struct KeyAssignmentUnassignArgs {
    #[command(flatten)]
    pub request: KeyAssignmentApplyArgs,

    /// Confirm destructive action.
    #[arg(long)]
    pub yes: bool,
}

#[derive(Debug, Clone, Subcommand)]
pub enum GuardrailKeyAssignmentCommands {
    /// List key assignments.
    List(AssignmentListArgs),
    /// Assign keys to a guardrail.
    Assign(KeyAssignmentApplyArgs),
    /// Unassign keys from a guardrail.
    Unassign(KeyAssignmentUnassignArgs),
}

#[derive(Debug, Clone, Args)]
pub struct MemberAssignmentApplyArgs {
    /// Guardrail ID.
    pub guardrail_id: String,

    /// One or more member user IDs.
    #[arg(value_name = "MEMBER_USER_ID", required = true, num_args = 1..)]
    pub member_user_ids: Vec<String>,
}

#[derive(Debug, Clone, Args)]
pub struct MemberAssignmentUnassignArgs {
    #[command(flatten)]
    pub request: MemberAssignmentApplyArgs,

    /// Confirm destructive action.
    #[arg(long)]
    pub yes: bool,
}

#[derive(Debug, Clone, Subcommand)]
pub enum GuardrailMemberAssignmentCommands {
    /// List member assignments.
    List(AssignmentListArgs),
    /// Assign members to a guardrail.
    Assign(MemberAssignmentApplyArgs),
    /// Unassign members from a guardrail.
    Unassign(MemberAssignmentUnassignArgs),
}

#[derive(Debug, Clone, Subcommand)]
pub enum GuardrailAssignmentCommands {
    /// Key assignment operations.
    Keys {
        #[command(subcommand)]
        command: GuardrailKeyAssignmentCommands,
    },
    /// Member assignment operations.
    Members {
        #[command(subcommand)]
        command: GuardrailMemberAssignmentCommands,
    },
}

#[derive(Debug, Clone, Subcommand)]
pub enum GuardrailsCommands {
    /// List guardrails.
    List(PaginationArgs),
    /// Create a guardrail.
    Create(GuardrailsCreateArgs),
    /// Get a guardrail.
    Get(GuardrailsGetArgs),
    /// Update a guardrail.
    Update(GuardrailsUpdateArgs),
    /// Delete a guardrail.
    Delete(GuardrailsDeleteArgs),
    /// Manage guardrail assignments.
    Assignments {
        #[command(subcommand)]
        command: GuardrailAssignmentCommands,
    },
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
    /// API key management commands.
    Keys {
        #[command(subcommand)]
        command: KeysCommands,
    },
    /// Guardrail management commands.
    Guardrails {
        #[command(subcommand)]
        command: GuardrailsCommands,
    },
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
