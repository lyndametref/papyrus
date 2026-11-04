pub mod browser;
pub mod commands;
pub mod config;
pub mod context;
pub mod error;
pub mod init;
pub mod milestone;
pub mod nfreq;
pub mod release;
pub mod req;
pub mod requirement;
pub mod sanitize;
pub mod search;
pub mod task;

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

/// Documented record types accepted by `papyrus search --type`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, clap::ValueEnum)]
pub enum SearchType {
    Task,
    Milestone,
    Release,
    Decision,
    Doc,
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
    /// Manage business requirements.
    Req {
        #[command(subcommand)]
        action: req::ReqCmd,
    },
    /// Manage non-functional requirements.
    Nfreq {
        #[command(subcommand)]
        action: nfreq::NfreqCmd,
    },
    /// Manage milestones.
    Milestone {
        #[command(subcommand)]
        action: milestone::MilestoneCmd,
    },
    /// Manage releases.
    Release {
        #[command(subcommand)]
        action: release::ReleaseCmd,
    },
    /// Manage tasks.
    Task {
        #[command(subcommand)]
        action: Option<task::TaskCmd>,
        /// Task ID (shorthand for view).
        #[arg(value_name = "id")]
        id: Option<String>,
    },
    /// Serve the local web interface.
    #[command(override_usage = "papyrus browser [--host <host>] [--port <port>] [--no-open]")]
    Browser {
        /// Bind address. Default: 127.0.0.1.
        #[arg(long, value_name = "host")]
        host: Option<String>,
        /// TCP port. Default: 6420.
        #[arg(
            long,
            value_name = "port",
            value_parser = parse_browser_port,
            allow_hyphen_values = true
        )]
        port: Option<u16>,
        /// Do not open the default browser automatically.
        #[arg(long = "no-open")]
        no_open: bool,
    },
    /// Search records by free text.
    Search {
        /// Free-text search term or phrase. Quote phrases that contain spaces.
        #[arg(value_name = "query")]
        query: String,
        /// Restrict to record types. Repeatable. Default: all types.
        #[arg(long = "type", value_name = "type", value_enum)]
        types: Vec<SearchType>,
        /// Maximum number of results. Must be a positive integer.
        #[arg(long, value_name = "n", value_parser = parse_limit, allow_hyphen_values = true)]
        limit: Option<usize>,
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

/// Parse `--limit`: a positive integer. Zero, negative, and non-integer values
/// are usage errors (clap exits with code 2).
/// Parse `--port`: an integer in `1..=65535`. Zero, negative, out-of-range,
/// and non-numeric values are usage errors (clap exits with code 2).
fn parse_browser_port(raw: &str) -> Result<u16, String> {
    let value: i64 = raw
        .parse()
        .map_err(|_| "port must be an integer between 1 and 65535".to_string())?;
    if !(1..=65535).contains(&value) {
        return Err("port must be an integer between 1 and 65535".to_string());
    }
    u16::try_from(value).map_err(|_| "port must be an integer between 1 and 65535".to_string())
}

pub(crate) fn parse_limit(raw: &str) -> Result<usize, String> {
    let value: i64 = raw
        .parse()
        .map_err(|_| "limit must be a positive integer".to_string())?;
    if value < 1 {
        return Err("limit must be a positive integer".to_string());
    }
    usize::try_from(value).map_err(|_| "limit must be a positive integer".to_string())
}

pub fn parse() -> Cli {
    Cli::parse()
}

/// Build a command context for non-init commands, enforcing the
/// project-directory boundary for the configuration path.
fn build_context(
    project_raw: PathBuf,
    config_path: PathBuf,
    json: bool,
    no_color: bool,
) -> Result<context::Context, error::AppError> {
    let project_dir = context::Context::canonical_project_dir(&project_raw)?;
    let config_path =
        context::Context::ensure_within(&project_dir, &config_path).map_err(|_| {
            error::AppError::Runtime(format!(
                "configuration path resolves outside project directory: {}",
                config_path.display()
            ))
        })?;
    Ok(context::Context {
        project_dir,
        config_path,
        json,
        no_color,
    })
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
            let ctx = build_context(project_raw, config_path, cli.json, no_color)?;
            match action {
                ConfigCmd::List => commands::config_list(&ctx),
                ConfigCmd::Get { key } => commands::config_get(&ctx, &key),
                ConfigCmd::Set { key, value } => commands::config_set(&ctx, &key, &value),
                ConfigCmd::Unset { key } => commands::config_unset(&ctx, &key),
            }
        }
        Commands::Req { action } => {
            let ctx = build_context(project_raw, config_path, cli.json, no_color)?;
            req::run(&ctx, action)
        }
        Commands::Nfreq { action } => {
            let ctx = build_context(project_raw, config_path, cli.json, no_color)?;
            nfreq::run(&ctx, action)
        }
        Commands::Milestone { action } => {
            let ctx = build_context(project_raw, config_path, cli.json, no_color)?;
            milestone::run(&ctx, action)
        }
        Commands::Release { action } => {
            let ctx = build_context(project_raw, config_path, cli.json, no_color)?;
            release::run(&ctx, action)
        }
        Commands::Task { action, id } => {
            let ctx = build_context(project_raw, config_path, cli.json, no_color)?;
            if let Some(a) = action {
                task::run(&ctx, a)
            } else if let Some(i) = id {
                task::view(&ctx, &i)
            } else {
                Err(error::AppError::Validation(
                    "missing task subcommand or id".to_string(),
                ))
            }
        }
        Commands::Browser {
            host,
            port,
            no_open,
        } => {
            let ctx = build_context(project_raw, config_path, cli.json, no_color)?;
            browser::run(&ctx, host.as_deref(), port, no_open)
        }
        Commands::Search {
            query,
            types,
            limit,
        } => {
            let ctx = build_context(project_raw, config_path, cli.json, no_color)?;
            search::run(&ctx, &query, &types, limit)
        }
    }
}
