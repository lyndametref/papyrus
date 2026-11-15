use crate::document::{DocumentArgs, DocumentInvocation, DocumentVerb};
use crate::error::AppError;
use crate::registry::{ArgAction, ArgSpec, ArtifactCategory, TypeDescriptor, VerbSpec};
use clap::{Arg, ArgMatches, Command, Subcommand};

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
            .action(clap::ArgAction::SetTrue)
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
            .action(clap::ArgAction::SetTrue)
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
    // clap's `string` feature lets the command own its name, so the slug does
    // not need a `'static` leak (ADR-0016 decision 1, ADR-0017 decision 10).
    let mut cmd = Command::new(descriptor.slug.clone())
        .about(format!("Manage {} documents.", descriptor.slug));

    for spec in descriptor.ordered_verb_specs() {
        cmd = cmd.subcommand(verb_command(spec));
    }
    cmd
}

fn verb_command(spec: &VerbSpec) -> Command {
    let mut cmd = Command::new(spec.verb.as_str());
    if !spec.about.is_empty() {
        cmd = cmd.about(spec.about);
    }
    for arg in spec.args {
        cmd = cmd.arg(build_arg(*arg));
    }
    cmd
}

fn build_arg(spec: ArgSpec) -> Arg {
    let mut arg = Arg::new(spec.id);
    if let Some(long) = spec.long {
        arg = arg.long(long);
    }
    if let Some(short) = spec.short {
        arg = arg.short(short);
    }
    if !spec.value_name.is_empty() {
        arg = arg.value_name(spec.value_name);
    }
    if !spec.help.is_empty() {
        arg = arg.help(spec.help);
    }
    arg = match spec.action {
        ArgAction::Set => arg,
        ArgAction::Append => arg.action(clap::ArgAction::Append),
        ArgAction::SetTrue => arg.action(clap::ArgAction::SetTrue),
        ArgAction::OptionalValue => arg.num_args(0..=1),
    };
    if spec.required {
        arg = arg.required(true);
    }
    if let Some(other) = spec.required_unless_present {
        arg = arg.required_unless_present(other);
    }
    if let Some(other) = spec.conflicts_with {
        arg = arg.conflicts_with(other);
    }
    arg
}

/// Parse one document invocation from the slug-level matches.
///
/// `generic` selects the historical no-verb behaviour: a config-defined type
/// lists, while a built-in typed type requires a verb. An unknown verb is
/// always a validation error, never a silent `list`.
pub fn parse_document_invocation(
    matches: &ArgMatches,
    slug: &str,
    generic: bool,
) -> Result<DocumentInvocation, AppError> {
    let (verb, sub) = match matches.subcommand() {
        Some((name, sub)) => {
            let verb = DocumentVerb::from_name(name)
                .ok_or_else(|| AppError::Validation(format!("unknown verb: {}", name)))?;
            (verb, sub)
        }
        None if generic => (DocumentVerb::List, matches),
        None => return Err(AppError::Validation("missing verb".to_string())),
    };

    let args = collect_args(sub);
    let title = args.get("title").or_else(|| args.get("title_flag"));
    let mut invocation = DocumentInvocation::new(slug, verb);
    invocation.id = args.get("id").map(str::to_string);
    invocation.title = title.map(str::to_string);
    invocation.description = args.get("description").map(str::to_string);
    invocation.labels = args.get_all("label").to_vec();
    invocation.references = args.get_all("reference").to_vec();
    invocation.editor = args.flag("editor");
    invocation.status_value = args.get("value").map(str::to_string);
    invocation.force = args.flag("force");
    invocation.args = args;
    Ok(invocation)
}

fn collect_args(matches: &ArgMatches) -> DocumentArgs {
    let mut args = DocumentArgs::default();
    for id in matches.ids() {
        let id = id.as_str();
        if matches!(id, "json" | "dir" | "config" | "no_color") {
            continue;
        }
        // Only explicitly supplied arguments belong to the invocation map;
        // defaults must not look like present switches.
        if matches.value_source(id) != Some(clap::parser::ValueSource::CommandLine) {
            continue;
        }
        let mut values: Vec<String> = Vec::new();
        if let Ok(Some(flags)) = matches.try_get_many::<bool>(id) {
            // Presence is the value; keep the key with an empty list.
            let _ = flags.count();
        } else if let Ok(Some(items)) = matches.try_get_many::<String>(id) {
            values.extend(items.cloned());
        }
        args.insert(id, values);
    }
    args
}
