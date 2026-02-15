use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use super::project::Project;

/// A portable project manifest used for export/import.
///
/// Contains relative paths and optional git remote URLs so a workspace
/// can be reconstructed on another machine.
#[derive(Debug, Serialize, Deserialize)]
pub struct Manifest {
    pub version: u32,
    pub base_dir: String,
    pub projects: Vec<ManifestEntry>,
}

/// A single project entry within a [`Manifest`].
#[derive(Debug, Serialize, Deserialize)]
pub struct ManifestEntry {
    pub name: String,
    pub relative_path: String,
    pub remote_url: Option<String>,
    pub tags: Vec<String>,
}

/// Compute the longest common prefix of all project paths.
fn common_prefix(paths: &[&Path]) -> PathBuf {
    if paths.is_empty() {
        return PathBuf::from(".");
    }
    let first = paths[0];
    let components: Vec<_> = first.components().collect();
    let mut prefix_len = components.len();

    for path in &paths[1..] {
        let other: Vec<_> = path.components().collect();
        let mut shared = 0;
        for (a, b) in components.iter().zip(other.iter()) {
            if a == b {
                shared += 1;
            } else {
                break;
            }
        }
        prefix_len = prefix_len.min(shared);
    }

    let mut result = PathBuf::new();
    for c in &components[..prefix_len] {
        result.push(c);
    }
    result
}

/// Read the git origin remote URL for a path.
fn read_git_remote(path: &Path) -> Option<String> {
    let repo = git2::Repository::open(path).ok()?;
    let remote = repo.find_remote("origin").ok()?;
    remote.url().map(|s| s.to_string())
}

/// Export projects to a manifest.
pub fn export(projects: &[Project], base_dir: Option<&Path>) -> Manifest {
    let paths: Vec<&Path> = projects.iter().map(|p| p.path.as_path()).collect();
    let base = base_dir
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| common_prefix(&paths));

    let entries: Vec<ManifestEntry> = projects
        .iter()
        .map(|p| {
            let relative_path = p
                .path
                .strip_prefix(&base)
                .map(|r| r.to_string_lossy().to_string())
                .unwrap_or_else(|_| p.path.to_string_lossy().to_string());
            let remote_url = read_git_remote(&p.path);
            ManifestEntry {
                name: p.name.clone(),
                relative_path,
                remote_url,
                tags: p.tags.clone(),
            }
        })
        .collect();

    Manifest {
        version: 1,
        base_dir: base.to_string_lossy().to_string(),
        projects: entries,
    }
}

/// Parse a manifest from TOML string.
pub fn parse(content: &str) -> Result<Manifest, toml::de::Error> {
    toml::from_str(content)
}

/// Serialize a manifest to TOML string.
pub fn serialize(manifest: &Manifest) -> Result<String, toml::ser::Error> {
    toml::to_string_pretty(manifest)
}

/// Determine target paths for import.
pub fn import_targets(
    manifest: &Manifest,
    base_dir: Option<&Path>,
) -> Vec<(ManifestEntry, PathBuf)> {
    let base = base_dir
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| PathBuf::from(&manifest.base_dir));

    manifest
        .projects
        .iter()
        .map(|entry| {
            let target = base.join(&entry.relative_path);
            (
                ManifestEntry {
                    name: entry.name.clone(),
                    relative_path: entry.relative_path.clone(),
                    remote_url: entry.remote_url.clone(),
                    tags: entry.tags.clone(),
                },
                target,
            )
        })
        .collect()
}
