use crate::context::Context;
use crate::error::AppError;
use crate::requirement;
use clap::Subcommand;

#[derive(Subcommand, Debug)]
pub enum ReqCmd {
    /// Create a business requirement.
    Create {
        /// Requirement title (positional).
        #[arg(value_name = "title", required_unless_present = "title_flag")]
        title: Option<String>,
        /// Requirement title (flag).
        #[arg(long = "title", required_unless_present = "title")]
        title_flag: Option<String>,
        /// User story.
        #[arg(long = "story")]
        story: Option<String>,
        /// Description.
        #[arg(short = 'd', long = "description")]
        description: Option<String>,
        /// Owning business-requirement group.
        #[arg(long = "group")]
        group: String,
    },
    /// Create a business-requirement group.
    CreateGroup {
        /// Group title.
        #[arg(long = "title")]
        title: String,
        /// User story.
        #[arg(long = "story")]
        story: Option<String>,
        /// Description.
        #[arg(short = 'd', long = "description")]
        description: Option<String>,
    },
    /// List business requirements.
    List,
    /// View a business requirement.
    View { id: String },
    /// Edit a business requirement.
    Edit {
        id: String,
        /// Update the title.
        #[arg(long = "title")]
        title: Option<String>,
        /// Update the user story.
        #[arg(long = "story")]
        story: Option<String>,
        /// Update the description.
        #[arg(short = 'd', long = "description")]
        description: Option<String>,
        /// Open the document body in $VISUAL or $EDITOR.
        #[arg(long = "editor")]
        editor: bool,
    },
    /// Deprecate a business requirement.
    Deprecate { id: String },
    /// Restore a deprecated business requirement.
    Restore { id: String },
}

pub fn run(ctx: &Context, cmd: ReqCmd) -> Result<crate::CommandOutput, AppError> {
    let kind = requirement::Kind::Req;
    match cmd {
        ReqCmd::Create {
            title,
            title_flag,
            story,
            description,
            group,
        } => {
            let id = requirement::create(
                ctx,
                kind,
                title.as_deref(),
                title_flag.as_deref(),
                story.as_deref(),
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
        ReqCmd::CreateGroup {
            title,
            story,
            description,
        } => {
            let id = requirement::create_group(
                ctx,
                kind,
                &title,
                story.as_deref(),
                description.as_deref(),
            )?;
            let text = id.clone();
            let json = if ctx.json {
                Some(serde_json::json!({"id": id}))
            } else {
                None
            };
            Ok(crate::CommandOutput { text, json })
        }
        ReqCmd::List => requirement::list(ctx, kind),
        ReqCmd::View { id } => requirement::view(ctx, kind, &id),
        ReqCmd::Edit {
            id,
            title,
            story,
            description,
            editor,
        } => requirement::edit(
            ctx,
            kind,
            &id,
            title.as_deref(),
            story.as_deref(),
            description.as_deref(),
            editor,
        ),
        ReqCmd::Deprecate { id } => requirement::deprecate(ctx, kind, &id),
        ReqCmd::Restore { id } => requirement::restore(ctx, kind, &id),
    }
}
