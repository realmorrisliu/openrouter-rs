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
    /// Discovery command group placeholder (planned in OR-20).
    Models,
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
