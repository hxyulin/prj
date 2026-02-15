use std::path::Path;

use serde::{Deserialize, Serialize};

/// Version control systems that `prj` can detect.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum VcsType {
    Git,
}

impl std::fmt::Display for VcsType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VcsType::Git => write!(f, "Git"),
        }
    }
}

/// Build systems detected by the presence of their marker files.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BuildSystem {
    Cargo,
    Npm,
    CMake,
    Go,
    Python,
    Zig,
    Make,
    Gradle,
    Maven,
    Meson,
}

impl std::fmt::Display for BuildSystem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            BuildSystem::Cargo => "Cargo",
            BuildSystem::Npm => "Npm",
            BuildSystem::CMake => "CMake",
            BuildSystem::Go => "Go",
            BuildSystem::Python => "Python",
            BuildSystem::Zig => "Zig",
            BuildSystem::Make => "Make",
            BuildSystem::Gradle => "Gradle",
            BuildSystem::Maven => "Maven",
            BuildSystem::Meson => "Meson",
        };
        write!(f, "{s}")
    }
}

struct BuildSystemInfo {
    marker: &'static str,
    system: BuildSystem,
    artifact_dirs: &'static [&'static str],
}

const BUILD_SYSTEMS: &[BuildSystemInfo] = &[
    BuildSystemInfo {
        marker: "Cargo.toml",
        system: BuildSystem::Cargo,
        artifact_dirs: &["target"],
    },
    BuildSystemInfo {
        marker: "package.json",
        system: BuildSystem::Npm,
        artifact_dirs: &["node_modules", "dist", "build"],
    },
    BuildSystemInfo {
        marker: "CMakeLists.txt",
        system: BuildSystem::CMake,
        artifact_dirs: &["build"],
    },
    BuildSystemInfo {
        marker: "go.mod",
        system: BuildSystem::Go,
        artifact_dirs: &[],
    },
    BuildSystemInfo {
        marker: "pyproject.toml",
        system: BuildSystem::Python,
        artifact_dirs: &["__pycache__", ".venv", "dist"],
    },
    BuildSystemInfo {
        marker: "build.zig",
        system: BuildSystem::Zig,
        artifact_dirs: &["zig-out", "zig-cache"],
    },
    BuildSystemInfo {
        marker: "Makefile",
        system: BuildSystem::Make,
        artifact_dirs: &[],
    },
    BuildSystemInfo {
        marker: "build.gradle",
        system: BuildSystem::Gradle,
        artifact_dirs: &["build", ".gradle"],
    },
    BuildSystemInfo {
        marker: "build.gradle.kts",
        system: BuildSystem::Gradle,
        artifact_dirs: &["build", ".gradle"],
    },
    BuildSystemInfo {
        marker: "pom.xml",
        system: BuildSystem::Maven,
        artifact_dirs: &["target"],
    },
    BuildSystemInfo {
        marker: "meson.build",
        system: BuildSystem::Meson,
        artifact_dirs: &["builddir"],
    },
];

/// Known artifact directory names (used during scan to skip).
pub const ARTIFACT_DIR_NAMES: &[&str] = &[
    "target",
    "node_modules",
    "dist",
    "build",
    "__pycache__",
    ".venv",
    "zig-out",
    "zig-cache",
    ".gradle",
    "builddir",
    ".git",
];

/// Result of scanning a project directory for VCS and build system markers.
pub struct DetectionResult {
    pub vcs: Vec<VcsType>,
    pub build_systems: Vec<BuildSystem>,
    pub artifact_dirs: Vec<String>,
}

/// Detect VCS, build systems, and artifact directories for a given path.
pub fn detect_project(path: &Path) -> DetectionResult {
    let mut vcs = Vec::new();
    let mut build_systems = Vec::new();
    let mut artifact_dirs = Vec::new();

    // VCS detection
    if path.join(".git").exists() {
        vcs.push(VcsType::Git);
    }

    // Build system detection
    for info in BUILD_SYSTEMS {
        if path.join(info.marker).exists() {
            // Avoid duplicate build systems (e.g. build.gradle and build.gradle.kts)
            if !build_systems.contains(&info.system) {
                build_systems.push(info.system.clone());
            }
            for dir in info.artifact_dirs {
                let s = dir.to_string();
                if !artifact_dirs.contains(&s) {
                    artifact_dirs.push(s);
                }
            }
        }
    }

    DetectionResult {
        vcs,
        build_systems,
        artifact_dirs,
    }
}

/// Returns true if the given path looks like a project root.
pub fn is_project(path: &Path) -> bool {
    if path.join(".git").exists() {
        return true;
    }
    for info in BUILD_SYSTEMS {
        if path.join(info.marker).exists() {
            return true;
        }
    }
    false
}

/// Scan a directory tree for projects up to `max_depth`.
/// Skips children of already-detected projects and artifact directories.
pub fn scan_projects(root: &Path, max_depth: usize) -> Vec<std::path::PathBuf> {
    let mut found = Vec::new();

    let walker = walkdir::WalkDir::new(root)
        .max_depth(max_depth)
        .follow_links(false)
        .into_iter();

    // Track project roots so we skip their children
    let mut project_roots: Vec<std::path::PathBuf> = Vec::new();

    for entry in walker.filter_entry(|e| {
        // Always allow the root itself
        if e.depth() == 0 {
            return true;
        }
        // Skip non-directories
        if !e.file_type().is_dir() {
            return false;
        }
        // Skip artifact directories
        if let Some(name) = e.file_name().to_str() {
            if ARTIFACT_DIR_NAMES.contains(&name) {
                return false;
            }
            // Skip hidden directories (except .git which we handle)
            if name.starts_with('.') {
                return false;
            }
        }
        true
    }) {
        let Ok(entry) = entry else { continue };
        if !entry.file_type().is_dir() {
            continue;
        }
        let path = entry.path();

        // Skip if this is a child of an already-found project
        if project_roots
            .iter()
            .any(|root| path.starts_with(root) && path != root)
        {
            continue;
        }

        if is_project(path) {
            project_roots.push(path.to_path_buf());
            found.push(path.to_path_buf());
        }
    }

    found
}
