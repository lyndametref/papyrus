use clap::Parser;
use std::path::PathBuf;

/// Papyrus command-line interface.
#[derive(Parser, Debug)]
#[command(
    name = "papyrus",
    version,
    about = "Project-level backlog and documentation manager.",
    arg_required_else_help = true
)]
pub struct Cli {
    /// Emit stable, machine-readable JSON.
    #[arg(long, action = clap::ArgAction::SetTrue)]
    pub json: bool,

    /// Project directory. Default: the current working directory.
    #[arg(long, value_name = "path")]
    pub dir: Option<PathBuf>,

    /// Configuration file. Default: papyrus.config.yml in the project directory.
    #[arg(long, value_name = "file")]
    pub config: Option<PathBuf>,

    /// Disable ANSI color output.
    #[arg(long, action = clap::ArgAction::SetTrue)]
    pub no_color: bool,
}

pub fn parse() -> Cli {
    Cli::parse()
}
