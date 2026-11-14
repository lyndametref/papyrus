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
        /// Label.
        #[arg(long = "label")]
        label: Vec<String>,
        /// Reference.
        #[arg(long = "reference")]
        reference: Vec<String>,
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
        /// Label.
        #[arg(long = "label")]
        label: Vec<String>,
        /// Reference.
        #[arg(long = "reference")]
        reference: Vec<String>,
    },
    /// Archive a business requirement.
    Archive { id: String },
    /// Delete a business requirement.
    Delete {
        id: String,
        #[arg(long)]
        force: bool,
    },
    /// Set business requirement status.
    Status { id: String, value: String },
}

pub fn run(ctx: &Context, cmd: ReqCmd) -> Result<crate::CommandOutput, AppError> {
    run_with_kind(ctx, requirement::Kind::Req, cmd)
}

pub fn run_nfr(ctx: &Context, cmd: ReqCmd) -> Result<crate::CommandOutput, AppError> {
    run_with_kind(ctx, requirement::Kind::Nfr, cmd)
}

fn run_with_kind(
    ctx: &Context,
    kind: requirement::Kind,
    cmd: ReqCmd,
) -> Result<crate::CommandOutput, AppError> {
    match cmd {
        ReqCmd::Create {
            title,
            title_flag,
            story,
            description,
            group,
            label,
            reference,
        } => {
            let id = requirement::create(
                ctx,
                kind,
                title.as_deref(),
                title_flag.as_deref(),
                story.as_deref(),
                description.as_deref(),
                &group,
                &label,
                &reference,
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
            label,
            reference,
        } => requirement::edit(
            ctx,
            kind,
            &id,
            title.as_deref(),
            story.as_deref(),
            description.as_deref(),
            &label,
            &reference,
            editor,
        ),
        ReqCmd::Archive { id } => {
            let slug = if matches!(kind, requirement::Kind::Req) {
                "req"
            } else {
                "nfr"
            };
            crate::document::archive(ctx, slug, &id)
        }
        ReqCmd::Delete { id, force } => {
            let slug = if matches!(kind, requirement::Kind::Req) {
                "req"
            } else {
                "nfr"
            };
            crate::document::delete(ctx, slug, &id, force)
        }
        ReqCmd::Status { id, value } => {
            let slug = if matches!(kind, requirement::Kind::Req) {
                "req"
            } else {
                "nfr"
            };
            crate::document::status(ctx, slug, &id, &value)
        }
    }
}
