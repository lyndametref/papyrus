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
            })
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
    use crate::registry::ContentModel;
    match &desc.content_model {
        ContentModel::Generic => {
            let ctx = build_context(project_raw, config_path, json, no_color)?;
            let inv = crate::cli::tree::parse_document_invocation(sub, slug);
            crate::document::run(&ctx, registry, inv)
        }
        _ => {
            // typed module dispatch based on subcommand name
            let (verb, verb_matches) = sub
                .subcommand()
                .ok_or_else(|| AppError::Validation("missing verb".to_string()))?;
            // status handled generically
            if verb == "status" {
                let ctx = build_context(project_raw, config_path, json, no_color)?;
                let id = verb_matches
                    .get_one::<String>("id")
                    .cloned()
                    .unwrap_or_default();
                let value = verb_matches
                    .get_one::<String>("value")
                    .cloned()
                    .unwrap_or_default();
                let inv = crate::document::DocumentInvocation {
                    slug: slug.to_string(),
                    verb: crate::document::DocumentVerb::Status,
                    id: Some(id),
                    title: None,
                    description: None,
                    labels: vec![],
                    references: vec![],
                    editor: false,
                    status_value: Some(value),
                    force: false,
                };
                return crate::document::run(&ctx, registry, inv);
            }
            // module dispatch: parse the slug-level matches
            let ctx = build_context(project_raw, config_path, json, no_color)?;
            dispatch_typed_module(sub, desc, &ctx)
        }
    }
}

fn dispatch_typed_module(
    matches: &ArgMatches,
    desc: &crate::registry::TypeDescriptor,
    ctx: &Context,
) -> Result<CommandOutput, AppError> {
    use crate::registry::TypedHandler;
    let handler = match &desc.content_model {
        crate::registry::ContentModel::Typed(s) | crate::registry::ContentModel::HtmlMockup(s) => {
            s.handler
        }
        _ => return Err(AppError::Validation("not typed".to_string())),
    };
    match handler {
        TypedHandler::Requirement(crate::registry::RequirementKind::Business) => {
            let cmd = crate::req::ReqCmd::from_arg_matches(matches)
                .map_err(|e| AppError::Validation(e.to_string()))?;
            crate::req::run(ctx, cmd)
        }
        TypedHandler::Requirement(crate::registry::RequirementKind::NonFunctional) => {
            let cmd = crate::req::ReqCmd::from_arg_matches(matches)
                .map_err(|e| AppError::Validation(e.to_string()))?;
            crate::req::run_nfr(ctx, cmd)
        }
        TypedHandler::Glossary => {
            let cmd = crate::glossary::GlossaryCmd::from_arg_matches(matches)
                .map_err(|e| AppError::Validation(e.to_string()))?;
            crate::glossary::run(ctx, cmd)
        }
        TypedHandler::Rule => {
            let cmd = crate::rule::RuleCmd::from_arg_matches(matches)
                .map_err(|e| AppError::Validation(e.to_string()))?;
            crate::rule::run(ctx, cmd)
        }
        TypedHandler::UiSpec => {
            let cmd = crate::ui_spec::UiSpecCmd::from_arg_matches(matches)
                .map_err(|e| AppError::Validation(e.to_string()))?;
            crate::ui_spec::run(ctx, cmd)
        }
        TypedHandler::Audit => {
            let cmd = crate::audit::AuditCmd::from_arg_matches(matches)
                .map_err(|e| AppError::Validation(e.to_string()))?;
            crate::audit::run(ctx, cmd)
        }
        TypedHandler::TechSpec => {
            let cmd = crate::tech_spec::TechSpecCmd::from_arg_matches(matches)
                .map_err(|e| AppError::Validation(e.to_string()))?;
            crate::tech_spec::run(ctx, cmd)
        }
        TypedHandler::UseCase => {
            let cmd = crate::use_case::UseCaseCmd::from_arg_matches(matches)
                .map_err(|e| AppError::Validation(e.to_string()))?;
            crate::use_case::run(ctx, cmd)
        }
        TypedHandler::TestCase => {
            let cmd = crate::test_case::TestCaseCmd::from_arg_matches(matches)
                .map_err(|e| AppError::Validation(e.to_string()))?;
            crate::test_case::run(ctx, cmd)
        }
        TypedHandler::TestReport => {
            let cmd = crate::test_report::TestReportCmd::from_arg_matches(matches)
                .map_err(|e| AppError::Validation(e.to_string()))?;
            crate::test_report::run(ctx, cmd)
        }
        TypedHandler::UiMockup => {
            let cmd = crate::ui_mockup::UiMockupCmd::from_arg_matches(matches)
                .map_err(|e| AppError::Validation(e.to_string()))?;
            crate::ui_mockup::run(ctx, cmd)
        }
        _ => Err(AppError::Validation("unsupported type".to_string())),
    }
}
