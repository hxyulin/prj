use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::config::Config;
use crate::detect::{BuildSystem, VcsType};
use crate::error::PrjError;

/// A registered project with its detected metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub name: String,
    pub path: PathBuf,
    pub vcs: Vec<VcsType>,
    pub build_systems: Vec<BuildSystem>,
    pub artifact_dirs: Vec<String>,
    pub added_at: DateTime<Utc>,
    #[serde(default)]
    pub tags: Vec<String>,
}

/// Persistent store of all registered projects, serialized as TOML.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ProjectDatabase {
    #[serde(default)]
    pub projects: Vec<Project>,
}

impl ProjectDatabase {
    /// Load the database from disk, or return an empty one if it doesn't exist.
    pub fn load(config: &Config) -> Result<Self, PrjError> {
        let path = config.database_path();
        if path.exists() {
            let content =
                std::fs::read_to_string(&path).map_err(|e| PrjError::DatabaseRead(Box::new(e)))?;
            toml::from_str(&content).map_err(|e| PrjError::DatabaseRead(Box::new(e)))
        } else {
            Ok(Self::default())
        }
    }

    /// Save the database to disk.
    pub fn save(&self, config: &Config) -> Result<(), PrjError> {
        let path = config.database_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| PrjError::DatabaseWrite(Box::new(e)))?;
        }
        let content =
            toml::to_string_pretty(self).map_err(|e| PrjError::DatabaseWrite(Box::new(e)))?;
        std::fs::write(&path, content).map_err(|e| PrjError::DatabaseWrite(Box::new(e)))?;
        Ok(())
    }

    /// Add a project. Returns error if a project with the same path already exists.
    pub fn add(&mut self, project: Project) -> Result<(), PrjError> {
        if self.projects.iter().any(|p| p.path == project.path) {
            return Err(PrjError::ProjectAlreadyExists(
                project.path.display().to_string(),
            ));
        }
        self.projects.push(project);
        Ok(())
    }

    /// Remove a project by name. Returns error if not found.
    pub fn remove(&mut self, name: &str) -> Result<Project, PrjError> {
        let idx = self
            .projects
            .iter()
            .position(|p| p.name == name)
            .ok_or_else(|| PrjError::ProjectNotFound(name.to_string()))?;
        Ok(self.projects.remove(idx))
    }

    /// Find a project by name.
    pub fn find(&self, name: &str) -> Option<&Project> {
        self.projects.iter().find(|p| p.name == name)
    }

    /// Find a project by name (mutable).
    pub fn find_mut(&mut self, name: &str) -> Option<&mut Project> {
        self.projects.iter_mut().find(|p| p.name == name)
    }

    /// Add tags to a project.
    pub fn add_tags(&mut self, name: &str, tags: &[String]) -> Result<(), PrjError> {
        let project = self
            .find_mut(name)
            .ok_or_else(|| PrjError::ProjectNotFound(name.to_string()))?;
        for tag in tags {
            if !project.tags.contains(tag) {
                project.tags.push(tag.clone());
            }
        }
        project.tags.sort();
        Ok(())
    }

    /// Remove tags from a project.
    pub fn remove_tags(&mut self, name: &str, tags: &[String]) -> Result<(), PrjError> {
        let project = self
            .find_mut(name)
            .ok_or_else(|| PrjError::ProjectNotFound(name.to_string()))?;
        project.tags.retain(|t| !tags.contains(t));
        Ok(())
    }

    /// Find projects whose paths no longer exist.
    pub fn find_orphaned(&self) -> Vec<&Project> {
        self.projects.iter().filter(|p| !p.path.exists()).collect()
    }

    /// Remove projects whose paths no longer exist, returning the removed ones.
    pub fn remove_orphaned(&mut self) -> Vec<Project> {
        let (orphaned, alive): (Vec<_>, Vec<_>) = std::mem::take(&mut self.projects)
            .into_iter()
            .partition(|p| !p.path.exists());
        self.projects = alive;
        orphaned
    }

    /// Register a project at the given path with detection.
    pub fn register(&mut self, path: &Path, name: Option<&str>) -> Result<&Project, PrjError> {
        let path = path
            .canonicalize()
            .map_err(|_| PrjError::PathNotFound(path.to_path_buf()))?;

        if !path.is_dir() {
            return Err(PrjError::NotADirectory(path));
        }

        let detection = crate::detect::detect_project(&path);

        let name = name.map(|s| s.to_string()).unwrap_or_else(|| {
            path.file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| "unknown".to_string())
        });

        let project = Project {
            name,
            path,
            vcs: detection.vcs,
            build_systems: detection.build_systems,
            artifact_dirs: detection.artifact_dirs,
            added_at: Utc::now(),
            tags: Vec::new(),
        };

        self.add(project)?;
        Ok(self.projects.last().expect("just pushed"))
    }
}
