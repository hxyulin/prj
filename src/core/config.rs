use std::path::PathBuf;

use directories::ProjectDirs;
use serde::{Deserialize, Serialize};

/// User configuration loaded from `~/.config/prj/config.toml`.
///
/// All fields have sensible defaults so the config file is optional.
#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    #[serde(default = "default_shell_cmd")]
    pub shell_cmd: String,
    #[serde(default = "default_scan_depth")]
    pub scan_depth: usize,
    pub database_path: Option<PathBuf>,
}

fn default_shell_cmd() -> String {
    "prjp".to_string()
}

fn default_scan_depth() -> usize {
    3
}

impl Default for Config {
    fn default() -> Self {
        Self {
            shell_cmd: default_shell_cmd(),
            scan_depth: default_scan_depth(),
            database_path: None,
        }
    }
}

impl Config {
    pub fn load() -> color_eyre::Result<Self> {
        let config_path = Self::config_path();
        if config_path.exists() {
            let content = std::fs::read_to_string(&config_path)?;
            Ok(toml::from_str(&content)?)
        } else {
            Ok(Self::default())
        }
    }

    pub fn config_path() -> PathBuf {
        Self::project_dirs().config_dir().join("config.toml")
    }

    pub fn database_path(&self) -> PathBuf {
        self.database_path
            .clone()
            .unwrap_or_else(|| Self::project_dirs().data_dir().join("projects.toml"))
    }

    fn project_dirs() -> ProjectDirs {
        ProjectDirs::from("", "", "prj").expect("could not determine project directories")
    }
}
