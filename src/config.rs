use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// The root directory of projects, where new projjects will be created.
    project_root: PathBuf,
}
