use std::collections::BTreeMap;
use std::path::Path;

use bytesize::ByteSize;
use serde::Serialize;

use crate::project::Project;

#[derive(Debug, Serialize)]
pub struct GitStatus {
    pub branch: Option<String>,
    pub is_dirty: bool,
    pub changed: usize,
    pub staged: usize,
    pub untracked: usize,
    pub ahead: usize,
    pub behind: usize,
}

#[derive(Debug, Serialize)]
pub struct LangStats {
    pub code: usize,
    pub comments: usize,
    pub blanks: usize,
    pub files: usize,
}

#[derive(Debug, Serialize)]
pub struct LocStats {
    pub languages: BTreeMap<String, LangStats>,
    pub total_code: usize,
    pub total_comments: usize,
    pub total_blanks: usize,
    pub total_files: usize,
}

#[derive(Debug, Serialize)]
pub struct DiskStats {
    pub total_bytes: u64,
    pub artifact_bytes: u64,
}

impl DiskStats {
    pub fn total_display(&self) -> String {
        ByteSize(self.total_bytes).to_string()
    }

    pub fn artifact_display(&self) -> String {
        ByteSize(self.artifact_bytes).to_string()
    }
}

/// Aggregated statistics for a single project.
#[derive(Debug, Serialize)]
pub struct ProjectStats {
    pub name: String,
    pub git: Option<GitStatus>,
    pub loc: LocStats,
    pub disk: DiskStats,
}

/// Aggregated statistics across all registered projects.
#[derive(Debug, Serialize)]
pub struct OverviewStats {
    pub total_projects: usize,
    pub total_code_lines: usize,
    pub total_disk_bytes: u64,
    pub total_artifact_bytes: u64,
    pub dirty_projects: usize,
    pub projects: Vec<ProjectStats>,
}

/// Collect git status for a project path.
pub fn collect_git_status(path: &Path) -> Option<GitStatus> {
    let repo = git2::Repository::open(path).ok()?;

    let branch = repo
        .head()
        .ok()
        .and_then(|h| h.shorthand().map(|s| s.to_string()));

    let statuses = repo
        .statuses(Some(
            git2::StatusOptions::new()
                .include_untracked(true)
                .recurse_untracked_dirs(false),
        ))
        .ok()?;

    let mut changed = 0;
    let mut staged = 0;
    let mut untracked = 0;

    for entry in statuses.iter() {
        let s = entry.status();
        if s.intersects(
            git2::Status::INDEX_NEW
                | git2::Status::INDEX_MODIFIED
                | git2::Status::INDEX_DELETED
                | git2::Status::INDEX_RENAMED
                | git2::Status::INDEX_TYPECHANGE,
        ) {
            staged += 1;
        }
        if s.intersects(
            git2::Status::WT_MODIFIED
                | git2::Status::WT_DELETED
                | git2::Status::WT_RENAMED
                | git2::Status::WT_TYPECHANGE,
        ) {
            changed += 1;
        }
        if s.intersects(git2::Status::WT_NEW) {
            untracked += 1;
        }
    }

    let is_dirty = changed > 0 || staged > 0 || untracked > 0;

    // ahead/behind
    let (ahead, behind) = (|| -> Option<(usize, usize)> {
        let head = repo.head().ok()?;
        let local_oid = head.target()?;
        let upstream = repo.branch_upstream_name(head.name()?).ok()?;
        let upstream_ref = repo.find_reference(upstream.as_str()?).ok()?;
        let upstream_oid = upstream_ref.target()?;
        repo.graph_ahead_behind(local_oid, upstream_oid).ok()
    })()
    .unwrap_or((0, 0));

    Some(GitStatus {
        branch,
        is_dirty,
        changed,
        staged,
        untracked,
        ahead,
        behind,
    })
}

/// Collect lines-of-code stats using tokei.
pub fn collect_loc_stats(path: &Path) -> LocStats {
    let config = tokei::Config {
        hidden: Some(false),
        no_ignore: Some(false),
        ..tokei::Config::default()
    };

    let mut languages = tokei::Languages::new();
    languages.get_statistics(&[path], &[], &config);

    let mut lang_map = BTreeMap::new();
    let mut total_code = 0;
    let mut total_comments = 0;
    let mut total_blanks = 0;
    let mut total_files = 0;

    for (lang_type, lang) in &languages {
        if lang.code == 0 && lang.comments == 0 && lang.blanks == 0 {
            continue;
        }
        let files = lang.reports.len();
        lang_map.insert(
            lang_type.to_string(),
            LangStats {
                code: lang.code,
                comments: lang.comments,
                blanks: lang.blanks,
                files,
            },
        );
        total_code += lang.code;
        total_comments += lang.comments;
        total_blanks += lang.blanks;
        total_files += files;
    }

    LocStats {
        languages: lang_map,
        total_code,
        total_comments,
        total_blanks,
        total_files,
    }
}

/// Collect disk usage stats.
pub fn collect_disk_stats(path: &Path, artifact_dirs: &[String]) -> DiskStats {
    let mut total_bytes: u64 = 0;
    let mut artifact_bytes: u64 = 0;

    let walker = walkdir::WalkDir::new(path).follow_links(false);

    for entry in walker.into_iter().filter_map(|e| e.ok()) {
        if !entry.file_type().is_file() {
            continue;
        }
        let size = entry.metadata().map(|m| m.len()).unwrap_or(0);
        total_bytes += size;

        // Check if this file is inside an artifact directory
        if let Ok(rel) = entry.path().strip_prefix(path)
            && let Some(first_component) = rel.components().next()
        {
            let component = first_component.as_os_str().to_string_lossy();
            if artifact_dirs.iter().any(|a| a == component.as_ref()) {
                artifact_bytes += size;
            }
        }
    }

    DiskStats {
        total_bytes,
        artifact_bytes,
    }
}

/// Collect full stats for a single project.
pub fn collect_project_stats(project: &Project) -> ProjectStats {
    let git = collect_git_status(&project.path);
    let loc = collect_loc_stats(&project.path);
    let disk = collect_disk_stats(&project.path, &project.artifact_dirs);

    ProjectStats {
        name: project.name.clone(),
        git,
        loc,
        disk,
    }
}

/// Collect overview stats across all projects (parallelized with rayon).
pub fn collect_overview_stats(projects: &[Project]) -> OverviewStats {
    use rayon::prelude::*;

    let project_stats: Vec<ProjectStats> = projects.par_iter().map(collect_project_stats).collect();

    let total_projects = project_stats.len();
    let total_code_lines: usize = project_stats.iter().map(|s| s.loc.total_code).sum();
    let total_disk_bytes: u64 = project_stats.iter().map(|s| s.disk.total_bytes).sum();
    let total_artifact_bytes: u64 = project_stats.iter().map(|s| s.disk.artifact_bytes).sum();
    let dirty_projects = project_stats
        .iter()
        .filter(|s| s.git.as_ref().is_some_and(|g| g.is_dirty))
        .count();

    OverviewStats {
        total_projects,
        total_code_lines,
        total_disk_bytes,
        total_artifact_bytes,
        dirty_projects,
        projects: project_stats,
    }
}
