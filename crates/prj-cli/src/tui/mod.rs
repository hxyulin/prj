pub mod actions;
mod app;
mod fuzzy;
mod view;

use std::path::PathBuf;

use prj_core::config::Config;
use prj_core::project::Project;

pub fn run_picker(projects: &[Project]) -> color_eyre::Result<Option<PathBuf>> {
    app::run_picker(projects)
}

/// Run the interactive list TUI. Returns a path if the user chose "cd to project".
pub fn run_list(
    projects: &mut Vec<Project>,
    config: &Config,
) -> color_eyre::Result<Option<PathBuf>> {
    app::run_list(projects, config)
}
