pub mod commands;
pub mod config;
pub mod context;
pub mod error;
pub mod init;
pub mod sanitize;

use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    name = "papyrus",
    version,
    about = "Project-level backlog and documentation manager.",
    arg_required_else_help = true
)]
pub struct Cli {
    /// Emit stable, machine-readable JSON.
    #[arg(long, action = clap::ArgAction::SetTrue, global = true)]
    pub json: bool,

    /// Project directory. Default: the current working directory.
    #[arg(long, value_name = "path", global = true)]
    pub dir: Option<PathBuf>,

    /// Configuration file. Default: papyrus.config.yml in the project directory.
    #[arg(long, value_name = "file", global = true)]
    pub config: Option<PathBuf>,

    /// Disable ANSI color output.
    #[arg(long, action = clap::ArgAction::SetTrue, global = true)]
    pub no_color: bool,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Initialize a project.
    Init {
        /// Project name.
        name: String,
        /// Do not initialize a Git repository.
        #[arg(long)]
        no_git: bool,
        /// Allow initialization in a non-empty directory.
        #[arg(long)]
        force: bool,
    },
    /// Read and edit configuration.
    Config {
        #[command(subcommand)]
        action: ConfigCmd,
    },
}

#[derive(Subcommand, Debug)]
pub enum ConfigCmd {
    /// Print all effective keys.
    List,
    /// Print one effective value.
    Get { key: String },
    /// Persist one value.
    Set {
        key: String,
        #[arg(allow_hyphen_values = true)]
        value: String,
    },
    /// Remove one key.
    Unset { key: String },
}

pub struct CommandOutput {
    pub text: String,
    pub json: Option<serde_json::Value>,
}

pub fn parse() -> Cli {
    Cli::parse()
}

pub fn run(cli: Cli) -> Result<CommandOutput, error::AppError> {
    let project_raw = context::Context::select_project_dir(cli.dir.clone())?;
    let config_path_raw = context::Context::select_config_path(cli.config.clone(), &project_raw);
    // Sanitize the selected config path components (exit 4 if unnormalizable).
    let config_path = crate::sanitize::sanitize_path(&config_path_raw).ok_or_else(|| {
        crate::error::AppError::Validation("configuration path cannot be normalized".to_string())
    })?;

    // no_color / NO_COLOR
    let no_color = cli.no_color
        || std::env::var("NO_COLOR")
            .map(|v| !v.is_empty())
            .unwrap_or(false);

    match cli.command {
        Commands::Init {
            name,
            no_git,
            force,
        } => {
            init::run_init(
                project_raw,
                init::InitOptions {
                    name,
                    no_git,
                    force,
                },
            )?;
            Ok(CommandOutput {
                text: String::new(),
                json: if cli.json {
                    Some(serde_json::json!({"status":"ok"}))
                } else {
                    None
                },
            })
        }
        Commands::Config { action } => {
            // For non-init commands, canonicalize project dir (must exist).
            let project_dir = context::Context::canonical_project_dir(&project_raw)?;
            // Enforce project-directory boundary for config path (incl. explicit --config).
            let config_path =
                context::Context::ensure_within(&project_dir, &config_path).map_err(|_| {
                    crate::error::AppError::Runtime(format!(
                        "configuration path resolves outside project directory: {}",
                        config_path.display()
                    ))
                })?;
            let ctx = context::Context {
                project_dir: project_dir.clone(),
                config_path,
                json: cli.json,
                no_color,
            };
            match action {
                ConfigCmd::List => commands::config_list(&ctx),
                ConfigCmd::Get { key } => commands::config_get(&ctx, &key),
                ConfigCmd::Set { key, value } => commands::config_set(&ctx, &key, &value),
                ConfigCmd::Unset { key } => commands::config_unset(&ctx, &key),
            }
        }
    }
}
