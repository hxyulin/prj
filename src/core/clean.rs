use std::path::Path;

use walkdir::WalkDir;

/// Summary of artifact directories that would be removed by a clean operation.
pub struct CleanPreview {
    pub dirs: Vec<(String, u64)>,
    pub total_bytes: u64,
}

/// Preview what would be cleaned for a project.
pub fn preview_clean(project_path: &Path, artifact_dirs: &[String]) -> CleanPreview {
    let mut dirs = Vec::new();
    let mut total_bytes = 0;

    for dir_name in artifact_dirs {
        let dir_path = project_path.join(dir_name);
        if !dir_path.exists() {
            continue;
        }
        let mut size = 0u64;
        for entry in WalkDir::new(&dir_path)
            .follow_links(false)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if entry.file_type().is_file() {
                size += entry.metadata().map(|m| m.len()).unwrap_or(0);
            }
        }
        dirs.push((dir_name.clone(), size));
        total_bytes += size;
    }

    CleanPreview { dirs, total_bytes }
}

/// Delete artifact directories, returning total bytes freed.
pub fn execute_clean(project_path: &Path, artifact_dirs: &[String]) -> std::io::Result<u64> {
    let preview = preview_clean(project_path, artifact_dirs);
    let total = preview.total_bytes;

    for (dir_name, _) in &preview.dirs {
        let dir_path = project_path.join(dir_name);
        if dir_path.exists() {
            std::fs::remove_dir_all(&dir_path)?;
        }
    }

    Ok(total)
}
