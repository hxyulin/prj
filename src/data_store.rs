use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataStore {
    version: String,
    projects: Vec<Project>,
    templates: Vec<Template>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    name: String,
    path: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize, clap::ValueEnum)]
pub enum ProjectType {
    Cargo,
    CMake,
    Makefile,
}

impl ProjectType {
    pub fn autodetect(path: &PathBuf) -> Vec<ProjectType> {
        let mut types = Vec::new();
        if path.join("Cargo.toml").exists() {
            types.push(ProjectType::Cargo);
        }
        if path.join("CMakeLists.txt").exists() {
            types.push(ProjectType::CMake);
        }
        if path.join("Makefile").exists() {
            types.push(ProjectType::Makefile);
        }
        types
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Template {
    name: String,
}

impl DataStore {
    pub fn get_path() -> PathBuf {
        dirs::data_dir().unwrap().join("prj")
    }

    pub fn load() -> Result<Self, String> {
        let path = Self::get_path();
        if !path.exists() {
            std::fs::create_dir_all(&path).map_err(|e| e.to_string())?;
        }
        let data_path = path.join("data.toml");
        if !data_path.exists() {
            let data = DataStore {
                version: "0.1.0".to_string(),
                projects: vec![],
                templates: vec![],
            };
            data.save().map_err(|e| e.to_string())?;
        }
        let data = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
        Ok(toml::from_str(&data).map_err(|e| e.to_string())?)
    }

    pub fn save(&self) -> Result<(), String> {
        let path = Self::get_path();
        let data_path = path.join("data.toml");
        let data = toml::to_string(self).map_err(|e| e.to_string())?;
        std::fs::write(data_path, data).map_err(|e| e.to_string())
    }
}

impl Drop for DataStore {
    fn drop(&mut self) {
        if let Err(err) = self.save() {
            eprintln!("Failed to save data store: {}", err);
            std::process::exit(1);
        }
    }
}
