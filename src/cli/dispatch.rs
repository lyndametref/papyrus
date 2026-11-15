use crate::CommandOutput;
use crate::cli::builtin::BuiltinCmd;
use crate::context::Context;
use crate::error::AppError;
use crate::registry::ArtifactRegistry;
use clap::ArgMatches;
use clap::FromArgMatches;
use std::path::PathBuf;

pub fn dispatch(
    matches: &ArgMatches,
    project_raw: PathBuf,
    config_path: PathBuf,
    json: bool,
    no_color: bool,
    registry: Option<&ArtifactRegistry>,
) -> Result<CommandOutput, AppError> {
    let (name, sub) = matches
        .subcommand()
        .ok_or_else(|| AppError::Validation("missing subcommand".to_string()))?;

    // builtins
    if let Ok(cmd) = BuiltinCmd::from_arg_matches(matches) {
        return dispatch_builtin(cmd, project_raw, config_path, json, no_color, registry);
    }

    // document slug?
    if let Some(reg) = registry
        && let Some(desc) = reg.descriptor(name)
        && desc.category == crate::registry::ArtifactCategory::Document
    {
        return dispatch_document(
            name,
            sub,
            desc,
            project_raw,
            config_path,
            json,
            no_color,
            reg,
        );
    }

    // unknown
    Err(AppError::Validation(format!("unknown command: {}", name)))
}

fn build_context(
    project_raw: PathBuf,
    config_path: PathBuf,
    json: bool,
    no_color: bool,
) -> Result<Context, AppError> {
    let project_dir = Context::canonical_project_dir(&project_raw)?;
    let config_path = Context::ensure_within(&project_dir, &config_path).map_err(|_| {
        AppError::Runtime(format!(
            "configuration path resolves outside project directory: {}",
            config_path.display()
        ))
    })?;
    if !config_path.exists() {
        return Err(AppError::Runtime(format!(
            "configuration file not found: {}; run `papyrus init` to create it",
            config_path.display()
        )));
    }
    Ok(Context {
        project_dir,
        config_path,
        json,
        no_color,
    })
}

fn build_check_context(
    project_raw: PathBuf,
    config_path: PathBuf,
    json: bool,
    no_color: bool,
) -> Result<Context, AppError> {
    let project_dir = Context::canonical_project_dir(&project_raw)?;
    let config_path = Context::ensure_within(&project_dir, &config_path).map_err(|_| {
        AppError::Runtime(format!(
            "configuration path resolves outside project directory: {}",
            config_path.display()
        ))
    })?;
    // The diagnostic command deliberately does not require the file to exist:
    // a missing configuration is a finding, not a startup failure.
    Ok(Context {
        project_dir,
        config_path,
        json,
        no_color,
    })
}

fn dispatch_builtin(
    cmd: BuiltinCmd,
    project_raw: PathBuf,
    config_path: PathBuf,
    json: bool,
    no_color: bool,
    registry: Option<&ArtifactRegistry>,
) -> Result<CommandOutput, AppError> {
    use crate::cli::builtin::ConfigCmd;
    match cmd {
        BuiltinCmd::Init { args } => {
            crate::init::run_init(
                project_raw,
                crate::init::InitOptions {
                    name: args.name,
                    no_git: args.no_git,
                    force: args.force,
                },
            )?;
            Ok(CommandOutput {
                text: String::new(),
                json: if json {
                    Some(serde_json::json!({"status":"ok"}))
                } else {
                    None
                },
                ..Default::default()
            })
        }
        BuiltinCmd::Check => {
            let ctx = build_check_context(project_raw, config_path, json, no_color)?;
            crate::check::run(&ctx)
        }
        BuiltinCmd::Config { action } => {
            let ctx = build_context(project_raw, config_path, json, no_color)?;
            match action {
                ConfigCmd::List => crate::commands::config_list(&ctx),
                ConfigCmd::Get { key } => crate::commands::config_get(&ctx, &key),
                ConfigCmd::Set { key, value } => crate::commands::config_set(&ctx, &key, &value),
                ConfigCmd::Unset { key } => crate::commands::config_unset(&ctx, &key),
            }
        }
        BuiltinCmd::Milestone { action } => {
            let ctx = build_context(project_raw, config_path, json, no_color)?;
            crate::milestone::run(&ctx, action)
        }
        BuiltinCmd::Release { action } => {
            let ctx = build_context(project_raw, config_path, json, no_color)?;
            crate::release::run(&ctx, action)
        }
        BuiltinCmd::Task { action, id } => {
            let ctx = build_context(project_raw, config_path, json, no_color)?;
            if let Some(a) = action {
                crate::task::run(&ctx, a)
            } else if let Some(i) = id {
                crate::task::view(&ctx, &i)
            } else {
                Err(AppError::Validation(
                    "missing task subcommand or id".to_string(),
                ))
            }
        }
        BuiltinCmd::Decision { action } => {
            let ctx = build_context(project_raw, config_path, json, no_color)?;
            crate::decision::run(&ctx, action)
        }
        BuiltinCmd::Import { action } => {
            let ctx = build_context(project_raw, config_path, json, no_color)?;
            crate::import::run(&ctx, action)
        }
        BuiltinCmd::Browser {
            host,
            port,
            no_open,
            debug,
        } => {
            let ctx = build_context(project_raw, config_path, json, no_color)?;
            let registry = registry
                .ok_or_else(|| AppError::Runtime("artifact registry is unavailable".to_string()))?;
            crate::browser::run(&ctx, registry, host.as_deref(), port, no_open, debug)
        }
        BuiltinCmd::Search {
            query,
            types,
            limit,
        } => {
            let ctx = build_context(project_raw, config_path, json, no_color)?;
            let registry = registry
                .ok_or_else(|| AppError::Runtime("artifact registry is unavailable".to_string()))?;
            crate::search::run(&ctx, registry, &query, &types, limit)
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn dispatch_document(
    slug: &str,
    sub: &ArgMatches,
    desc: &crate::registry::TypeDescriptor,
    project_raw: PathBuf,
    config_path: PathBuf,
    json: bool,
    no_color: bool,
    registry: &ArtifactRegistry,
) -> Result<CommandOutput, AppError> {
    let ctx = build_context(project_raw, config_path, json, no_color)?;
    let generic = matches!(desc.content_model, crate::registry::ContentModel::Generic);
    let invocation = crate::cli::tree::parse_document_invocation(sub, slug, generic)?;
    crate::document::run(&ctx, registry, invocation)
}
