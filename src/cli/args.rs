use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "prj", about = "Local project manager", version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Subcommand)]
pub enum Command {
    /// Register a project (defaults to current directory)
    Add {
        /// Path to the project directory
        path: Option<PathBuf>,
        /// Display name for the project
        #[arg(long)]
        name: Option<String>,
    },

    /// Recursively discover and register projects
    Scan {
        /// Directory to scan
        dir: PathBuf,
        /// Maximum directory depth to scan
        #[arg(long, default_value = "3")]
        depth: usize,
    },

    /// Git clone and auto-register
    New {
        /// Git clone arguments (e.g. `https://github.com/user/repo`)
        #[arg(long = "git")]
        git: String,
    },

    /// Unregister a project (no file deletion)
    Remove {
        /// Project name to remove
        project: String,
    },

    /// List registered projects
    List {
        /// Plain text output (no TUI)
        #[arg(long)]
        plain: bool,
        /// Filter by tag
        #[arg(long)]
        tag: Option<String>,
    },

    /// Show project statistics
    Stats {
        /// Specific project name (omit for overview)
        project: Option<String>,
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// Output shell init script
    Init {
        /// Shell type (zsh, bash, powershell)
        shell: String,
        /// Name of the shell function to create
        #[arg(long, default_value = "prjp")]
        cmd: String,
    },

    /// Add tags to a project
    Tag {
        /// Project name
        project: String,
        /// Tags to add
        tags: Vec<String>,
    },

    /// Remove tags from a project
    Untag {
        /// Project name
        project: String,
        /// Tags to remove
        tags: Vec<String>,
    },

    /// Quick git status dashboard across all projects
    Status {
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// Remove projects whose paths no longer exist
    Gc {
        /// Only show what would be removed
        #[arg(long)]
        dry_run: bool,
    },

    /// Delete artifact directories (target, node_modules, etc.)
    Clean {
        /// Project name (omit with --all for all projects)
        project: Option<String>,
        /// Clean all projects
        #[arg(long)]
        all: bool,
        /// Only show what would be deleted
        #[arg(long)]
        dry_run: bool,
    },

    /// Run a command in project directory(s)
    Run {
        /// Command to execute
        cmd: String,
        /// Target specific project
        #[arg(long)]
        project: Option<String>,
        /// Filter by tag
        #[arg(long)]
        tag: Option<String>,
        /// Run in all projects
        #[arg(long)]
        all: bool,
    },

    /// Export project manifest
    Export {
        /// Output file (defaults to stdout)
        #[arg(long)]
        output: Option<PathBuf>,
        /// Base directory for relative paths
        #[arg(long)]
        base_dir: Option<PathBuf>,
    },

    /// Import and clone projects from a manifest
    Import {
        /// Manifest file to import
        file: PathBuf,
        /// Base directory for cloning
        #[arg(long)]
        base_dir: Option<PathBuf>,
    },
}
