use crate::document::DocumentInvocation;
use crate::document::DocumentVerb;
use crate::registry::{ArtifactCategory, ContentModel, TypeDescriptor, Verb};
use clap::{Arg, ArgAction, ArgMatches, Command, Subcommand};

pub fn tree(
    registry: Option<&crate::registry::ArtifactRegistry>,
) -> Result<Command, crate::error::AppError> {
    let mut cmd = Command::new("papyrus")
        .version(env!("CARGO_PKG_VERSION"))
        .about("Project-level backlog and documentation manager.")
        .arg_required_else_help(true);

    // global args
    cmd = cmd.arg(
        Arg::new("json")
            .long("json")
            .action(ArgAction::SetTrue)
            .global(true)
            .help("Emit stable, machine-readable JSON."),
    );
    cmd = cmd.arg(
        Arg::new("dir")
            .long("dir")
            .value_name("path")
            .global(true)
            .help("Project directory. Default: the current working directory."),
    );
    cmd = cmd.arg(
        Arg::new("config")
            .long("config")
            .value_name("file")
            .global(true)
            .help("Configuration file. Default: papyrus.config.yml in the project directory."),
    );
    cmd = cmd.arg(
        Arg::new("no_color")
            .long("no-color")
            .action(ArgAction::SetTrue)
            .global(true)
            .help("Disable ANSI color output."),
    );

    // builtins
    cmd = crate::cli::builtin::BuiltinCmd::augment_subcommands(cmd);

    // document slugs
    if let Some(reg) = registry {
        let builtins: Vec<&str> = crate::cli::builtin::BUILTIN_COMMANDS
            .iter()
            .map(|e| e.name)
            .collect();
        // Pre-build guard: reject any document type whose slug collides with a
        // built-in command name (case-insensitive).
        for desc in reg.descriptors() {
            if desc.category != crate::registry::ArtifactCategory::Document {
                continue;
            }
            if let Some(builtin) = builtins.iter().find(|b| b.eq_ignore_ascii_case(&desc.slug)) {
                return Err(crate::error::AppError::Validation(format!(
                    "document_types.{}: slug conflicts with built-in type {}",
                    desc.slug, builtin
                )));
            }
        }
        for desc in reg.enabled(ArtifactCategory::Document) {
            cmd = cmd.subcommand(document_command(desc));
        }
    }

    Ok(cmd)
}

pub fn document_command(descriptor: &TypeDescriptor) -> Command {
    let slug = descriptor.slug.clone();
    let slug_static: &'static str = Box::leak(slug.clone().into_boxed_str());
    let mut cmd = Command::new(slug_static).about(format!("Manage {} documents.", slug));

    match &descriptor.content_model {
        ContentModel::Generic => {
            // baseline
            cmd = cmd.subcommand(create_generic_cmd());
            cmd = cmd.subcommand(Command::new("list").about("List documents."));
            cmd = cmd.subcommand(
                Command::new("view")
                    .about("View a document.")
                    .arg(Arg::new("id").required(true)),
            );
            cmd = cmd.subcommand(edit_generic_cmd());
            cmd = cmd.subcommand(
                Command::new("archive")
                    .about("Archive a document.")
                    .arg(Arg::new("id").required(true)),
            );
            cmd = cmd.subcommand(
                Command::new("delete")
                    .about("Delete a document.")
                    .arg(Arg::new("id").required(true))
                    .arg(Arg::new("force").long("force").action(ArgAction::SetTrue)),
            );
            if descriptor.has_verb(Verb::Status) {
                cmd = cmd.subcommand(
                    Command::new("status")
                        .about("Set document status.")
                        .arg(Arg::new("id").required(true))
                        .arg(Arg::new("value").required(true)),
                );
            }
        }
        ContentModel::Typed(schema) | ContentModel::HtmlMockup(schema) => {
            // augment with module enum
            match schema.handler {
                crate::registry::TypedHandler::Requirement(
                    crate::registry::RequirementKind::Business,
                ) => {
                    cmd = crate::req::ReqCmd::augment_subcommands(cmd);
                }
                crate::registry::TypedHandler::Requirement(
                    crate::registry::RequirementKind::NonFunctional,
                ) => {
                    cmd = crate::req::ReqCmd::augment_subcommands(cmd);
                }
                crate::registry::TypedHandler::Glossary => {
                    cmd = crate::glossary::GlossaryCmd::augment_subcommands(cmd);
                }
                crate::registry::TypedHandler::Rule => {
                    cmd = crate::rule::RuleCmd::augment_subcommands(cmd);
                }
                crate::registry::TypedHandler::UiSpec => {
                    cmd = crate::ui_spec::UiSpecCmd::augment_subcommands(cmd);
                }
                crate::registry::TypedHandler::Audit => {
                    cmd = crate::audit::AuditCmd::augment_subcommands(cmd);
                }
                crate::registry::TypedHandler::TechSpec => {
                    cmd = crate::tech_spec::TechSpecCmd::augment_subcommands(cmd);
                }
                crate::registry::TypedHandler::UseCase => {
                    cmd = crate::use_case::UseCaseCmd::augment_subcommands(cmd);
                }
                crate::registry::TypedHandler::TestCase => {
                    cmd = crate::test_case::TestCaseCmd::augment_subcommands(cmd);
                }
                crate::registry::TypedHandler::TestReport => {
                    cmd = crate::test_report::TestReportCmd::augment_subcommands(cmd);
                }
                crate::registry::TypedHandler::UiMockup => {
                    cmd = crate::ui_mockup::UiMockupCmd::augment_subcommands(cmd);
                }
                _ => {}
            }
            // Typed modules declare their own status subcommand when applicable.
        }
    }

    cmd
}

fn create_generic_cmd() -> Command {
    Command::new("create")
        .about("Create a document.")
        .arg(Arg::new("title").value_name("title").required(false))
        .arg(Arg::new("title_flag").long("title").value_name("title"))
        .arg(Arg::new("description").short('d').long("description"))
        .arg(Arg::new("label").long("label").action(ArgAction::Append))
        .arg(
            Arg::new("reference")
                .long("reference")
                .action(ArgAction::Append),
        )
}

fn edit_generic_cmd() -> Command {
    Command::new("edit")
        .about("Edit a document.")
        .arg(Arg::new("id").required(true))
        .arg(Arg::new("title").long("title"))
        .arg(Arg::new("description").short('d').long("description"))
        .arg(Arg::new("editor").long("editor").action(ArgAction::SetTrue))
        .arg(Arg::new("label").long("label").action(ArgAction::Append))
        .arg(
            Arg::new("reference")
                .long("reference")
                .action(ArgAction::Append),
        )
}

pub fn parse_document_invocation(matches: &ArgMatches, slug: &str) -> DocumentInvocation {
    // matches is the slug-level ArgMatches
    let (verb_name, sub) = matches.subcommand().unwrap_or(("list", matches));
    let verb = match verb_name {
        "create" => DocumentVerb::Create,
        "list" => DocumentVerb::List,
        "view" => DocumentVerb::View,
        "edit" => DocumentVerb::Edit,
        "archive" => DocumentVerb::Archive,
        "delete" => DocumentVerb::Delete,
        "status" => DocumentVerb::Status,
        _ => DocumentVerb::List,
    };
    let id = sub.try_get_one::<String>("id").ok().flatten().cloned();
    // title: positional or flag
    let title = sub
        .try_get_one::<String>("title")
        .ok()
        .flatten()
        .cloned()
        .or_else(|| {
            sub.try_get_one::<String>("title_flag")
                .ok()
                .flatten()
                .cloned()
        });
    let description = sub
        .try_get_one::<String>("description")
        .ok()
        .flatten()
        .cloned();
    let labels: Vec<String> = sub
        .try_get_many::<String>("label")
        .ok()
        .flatten()
        .map(|v| v.cloned().collect())
        .unwrap_or_default();
    let references: Vec<String> = sub
        .try_get_many::<String>("reference")
        .ok()
        .flatten()
        .map(|v| v.cloned().collect())
        .unwrap_or_default();
    let editor = sub
        .try_get_one::<bool>("editor")
        .ok()
        .flatten()
        .copied()
        .unwrap_or(false);
    let status_value = sub.try_get_one::<String>("value").ok().flatten().cloned();
    let force = sub
        .try_get_one::<bool>("force")
        .ok()
        .flatten()
        .copied()
        .unwrap_or(false);
    DocumentInvocation {
        slug: slug.to_string(),
        verb,
        id,
        title,
        description,
        labels,
        references,
        editor,
        status_value,
        force,
    }
}
