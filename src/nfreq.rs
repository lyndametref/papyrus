use crate::context::Context;
use crate::error::AppError;
use crate::requirement;
use clap::Subcommand;

#[derive(Subcommand, Debug)]
pub enum NfreqCmd {
    /// Create a non-functional requirement.
    Create {
        /// Requirement title.
        #[arg(value_name = "title")]
        title: String,
        /// Description.
        #[arg(short = 'd', long = "description")]
        description: Option<String>,
        /// Owning non-functional-requirement group.
        #[arg(long = "group")]
        group: String,
    },
    /// Create a non-functional-requirement group.
    CreateGroup {
        /// Group title.
        #[arg(long = "title")]
        title: String,
        /// Description.
        #[arg(short = 'd', long = "description")]
        description: Option<String>,
    },
    /// List non-functional requirements.
    List,
    /// View a non-functional requirement.
    View { id: String },
    /// Edit a non-functional requirement.
    Edit {
        id: String,
        /// Update the title.
        #[arg(long = "title")]
        title: Option<String>,
        /// Update the description.
        #[arg(short = 'd', long = "description")]
        description: Option<String>,
        /// Open the document body in $VISUAL or $EDITOR.
        #[arg(long = "editor")]
        editor: bool,
    },
    /// Deprecate a non-functional requirement.
    Deprecate { id: String },
    /// Restore a deprecated non-functional requirement.
    Restore { id: String },
}

pub fn run(ctx: &Context, cmd: NfreqCmd) -> Result<crate::CommandOutput, AppError> {
    let kind = requirement::Kind::Nfr;
    match cmd {
        NfreqCmd::Create {
            title,
            description,
            group,
        } => {
            let id = requirement::create(
                ctx,
                kind,
                Some(&title),
                None,
                None,
                description.as_deref(),
                &group,
            )?;
            let text = id.clone();
            let json = if ctx.json {
                Some(serde_json::json!({"id": id}))
            } else {
                None
            };
            Ok(crate::CommandOutput { text, json })
        }
        NfreqCmd::CreateGroup { title, description } => {
            let id = requirement::create_group(ctx, kind, &title, None, description.as_deref())?;
            let text = id.clone();
            let json = if ctx.json {
                Some(serde_json::json!({"id": id}))
            } else {
                None
            };
            Ok(crate::CommandOutput { text, json })
        }
        NfreqCmd::List => requirement::list(ctx, kind),
        NfreqCmd::View { id } => requirement::view(ctx, kind, &id),
        NfreqCmd::Edit {
            id,
            title,
            description,
            editor,
        } => requirement::edit(
            ctx,
            kind,
            &id,
            title.as_deref(),
            None,
            description.as_deref(),
            editor,
        ),
        NfreqCmd::Deprecate { id } => requirement::deprecate(ctx, kind, &id),
        NfreqCmd::Restore { id } => requirement::restore(ctx, kind, &id),
    }
}
