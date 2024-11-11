use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Project {
    pub name: String,
    pub project_type: ProjectType,
    pub path: PathBuf,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ProjectType {
    CMake,
    Cargo,
    Other, // Extend as needed
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectStorage {
    pub projects: Vec<Project>,
}

impl ProjectStorage {
    /// Load projects from file or initialize new storage
    pub fn load_or_initialize() -> Self {
        let file_path = project_file_path();
        if file_path.exists() {
            let contents = fs::read_to_string(file_path).expect("Failed to read projects file");
            toml::from_str(&contents).unwrap_or_else(|_| Self {
                projects: Vec::new(),
            })
        } else {
            Self {
                projects: Vec::new(),
            }
        }
    }

    /// Save all projects to `projects.toml`
    pub fn save(&self) {
        let file_path = project_file_path();
        let toml_data = toml::to_string(self).expect("Failed to serialize to TOML");
        fs::write(file_path, toml_data).expect("Failed to write projects file");
    }

    /// Add a new project
    pub fn add_project(&mut self, project: Project) {
        self.projects.push(project);
        self.save();
    }

    /// Remove a project by name or path
    pub fn remove_project(&mut self, name: Option<String>, path: Option<PathBuf>) {
        self.projects.retain(|proj| {
            !name.as_ref().map_or(false, |n| &proj.name == n)
                && !path.as_ref().map_or(false, |p| &proj.path == p)
        });
        self.save();
    }

    /// Get a project path by name for navigation
    pub fn get_project_path(&self, name: &str) -> Option<PathBuf> {
        self.projects
            .iter()
            .find(|proj| proj.name == name)
            .map(|proj| proj.path.clone())
    }
}

pub fn project_state_dir() -> PathBuf {
    let base_dir = match env::var("XDG_STATE_HOME") {
        Ok(dir) => PathBuf::from(dir),
        Err(_) => {
            let mut home_dir = dirs::home_dir().expect("Unable to determine home directory");
            home_dir.push(".local/state");
            home_dir
        }
    };

    let prj_dir = base_dir.join("prj");
    if !prj_dir.exists() {
        fs::create_dir_all(&prj_dir).expect("Failed to create prj directory");
    }

    prj_dir
}

fn project_file_path() -> PathBuf {
    let prj_dir = project_state_dir();
    prj_dir.join("projects.toml")
}
