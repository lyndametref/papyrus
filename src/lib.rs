pub mod audit;
pub mod browser;
pub mod check;
pub mod cli;
pub mod commands;
pub mod config;
pub mod context;
pub mod decision;
pub mod document;
pub mod error;
pub mod glossary;
pub mod import;
pub mod init;
pub mod milestone;
pub mod record;
pub mod registry;
pub mod release;
pub mod requirement;
pub mod rule;
pub mod sanitize;
pub mod search;
pub mod task;
pub mod tech_spec;
pub mod test_case;
pub mod test_report;
pub mod ui_mockup;
pub mod ui_spec;
pub mod use_case;

/// Documented record types accepted by `papyrus search --type`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, clap::ValueEnum)]
pub enum SearchType {
    Task,
    Milestone,
    Release,
    Decision,
    Doc,
}

#[derive(Default)]
pub struct CommandOutput {
    pub text: String,
    pub json: Option<serde_json::Value>,
    /// Process exit status; findings use 4, every other command keeps 0.
    pub exit_code: i32,
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

pub fn run() -> Result<CommandOutput, error::AppError> {
    use crate::cli;
    use crate::registry::ArtifactRegistry;

    let full_args: Vec<String> = std::env::args().collect();
    let args: Vec<String> = full_args.iter().skip(1).cloned().collect();
    let prescan = cli::scan(&args);

    let project_raw = context::Context::select_project_dir(prescan.dir.clone())?;
    let config_path_raw =
        context::Context::select_config_path(prescan.config.clone(), &project_raw);
    let config_path = crate::sanitize::sanitize_path(&config_path_raw).ok_or_else(|| {
        crate::error::AppError::Validation("configuration path cannot be normalized".to_string())
    })?;

    let json = args.iter().any(|a| a == "--json");
    let no_color = args.iter().any(|a| a == "--no-color")
        || std::env::var("NO_COLOR")
            .map(|v| !v.is_empty())
            .unwrap_or(false);

    let first = prescan.first_positional.clone();
    let is_init = first.as_deref() == Some("init");
    let is_config = first.as_deref() == Some("config");
    let is_check = first.as_deref() == Some("check");
    let is_help = prescan.help || first.as_deref() == Some("help");
    let is_version = prescan.version;

    // Phase 2: config
    let (cfg_opt, registry_opt): (Option<crate::config::Config>, Option<ArtifactRegistry>) =
        if is_init {
            (None, None)
        } else if is_config {
            let cfg = if config_path.exists() {
                crate::commands::load_effective_config(&config_path, &project_raw).ok()
            } else {
                None
            };
            let reg = cfg.as_ref().and_then(|c| ArtifactRegistry::build(c).ok());
            (cfg, reg)
        } else if is_check {
            // The diagnostic command loads its own configuration, so the tree
            // must exist before that load (ADR-0029 decision 2).
            (None, None)
        } else if (is_help || is_version) && !config_path.exists() {
            let cfg = crate::config::Config::default();
            let reg = ArtifactRegistry::build(&cfg).ok();
            (Some(cfg), reg)
        } else {
            // load + validate
            let cfg = crate::commands::load_effective_config(&config_path, &project_raw)?;
            let reg = ArtifactRegistry::build(&cfg).map_err(crate::error::AppError::from)?;
            (Some(cfg), Some(reg))
        };

    let tree = cli::tree(registry_opt.as_ref())?;

    // Phase 3: parse once
    let matches = match tree.try_get_matches_from(&full_args) {
        Ok(m) => m,
        Err(e) => {
            use clap::error::ErrorKind;
            match e.kind() {
                ErrorKind::DisplayHelp | ErrorKind::DisplayVersion => {
                    e.print().ok();
                    std::process::exit(0);
                }
                _ => {
                    e.print().ok();
                    std::process::exit(2);
                }
            }
        }
    };

    // Phase 4: dispatch
    cli::dispatch(
        &matches,
        project_raw,
        config_path,
        json,
        no_color,
        cfg_opt.as_ref(),
        registry_opt.as_ref(),
    )
}
