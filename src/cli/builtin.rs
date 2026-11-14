use crate::SearchType;
use crate::decision;
use crate::import;
use crate::milestone;
use crate::release;
use crate::task;
use clap::{Args, Subcommand};

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

#[derive(Args, Debug)]
pub struct InitArgs {
    /// Project name.
    pub name: String,
    /// Do not initialize a Git repository.
    #[arg(long)]
    pub no_git: bool,
    /// Allow initialization in a non-empty directory.
    #[arg(long)]
    pub force: bool,
}

#[derive(Subcommand, Debug)]
pub enum BuiltinCmd {
    /// Initialize a project.
    Init {
        #[command(flatten)]
        args: InitArgs,
    },
    /// Read and edit configuration.
    Config {
        #[command(subcommand)]
        action: ConfigCmd,
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
    /// Manage decision records.
    Decision {
        #[command(subcommand)]
        action: decision::DecisionCmd,
    },
    /// Import records from a Backlog.md store.
    Import {
        #[command(subcommand)]
        action: import::ImportCmd,
    },
    /// Serve the local web interface.
    #[command(
        override_usage = "papyrus browser [--host <host>] [--port <port>] [--no-open] [--debug]"
    )]
    Browser {
        /// Bind address. Default: 127.0.0.1.
        #[arg(long, value_name = "host")]
        host: Option<String>,
        /// TCP port. Default: 6420.
        #[arg(
            long,
            value_name = "port",
            value_parser = crate::parse_browser_port,
            allow_hyphen_values = true
        )]
        port: Option<u16>,
        /// Do not open the default browser automatically.
        #[arg(long = "no-open")]
        no_open: bool,
        /// Log one line per received request to stderr.
        #[arg(long)]
        debug: bool,
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
        #[arg(long, value_name = "n", value_parser = crate::parse_limit, allow_hyphen_values = true)]
        limit: Option<usize>,
    },
}

pub struct BuiltinEntry {
    pub name: &'static str,
}

pub const BUILTIN_COMMANDS: &[BuiltinEntry] = &[
    BuiltinEntry { name: "init" },
    BuiltinEntry { name: "config" },
    BuiltinEntry { name: "task" },
    BuiltinEntry { name: "milestone" },
    BuiltinEntry { name: "release" },
    BuiltinEntry { name: "decision" },
    BuiltinEntry { name: "search" },
    BuiltinEntry { name: "browser" },
    BuiltinEntry { name: "import" },
    BuiltinEntry { name: "help" },
    BuiltinEntry { name: "version" },
];

// help/version are clap-provided; ensure names reserved.
