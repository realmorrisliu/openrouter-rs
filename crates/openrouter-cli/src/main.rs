mod cli;
mod config;

use anyhow::Result;
use clap::Parser;
use serde::Serialize;

use crate::{
    cli::{Cli, Commands, ConfigCommands, OutputFormat, ProfileCommands},
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

fn run() -> Result<()> {
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
        Commands::Models => {
            println!("models command group is not implemented yet (planned in OR-20)");
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

fn main() {
    if let Err(error) = run() {
        eprintln!("error: {error:#}");
        std::process::exit(1);
    }
}
