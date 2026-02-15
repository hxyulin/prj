pub mod args;

use std::io::{self, IsTerminal, Write};
use std::path::{Path, PathBuf};
use std::process;

use bytesize::ByteSize;

use prj_core::clean;
use prj_core::config::Config;
use prj_core::detect;
use prj_core::error::PrjError;
use prj_core::manifest;
use prj_core::project::ProjectDatabase;
use prj_core::stats;

use self::args::{Cli, Command};

pub fn run(cli: Cli) -> color_eyre::Result<()> {
    let config = Config::load()?;

    match cli.command {
        None => cmd_picker(&config)?,
        Some(Command::Add { path, name }) => cmd_add(&config, path, name.as_deref())?,
        Some(Command::Scan { dir, depth }) => cmd_scan(&config, &dir, depth)?,
        Some(Command::New { git }) => cmd_new(&config, &git)?,
        Some(Command::Remove { project }) => cmd_remove(&config, &project)?,
        Some(Command::List { plain, tag }) => cmd_list(&config, plain, tag.as_deref())?,
        Some(Command::Stats { project, json }) => cmd_stats(&config, project.as_deref(), json)?,
        Some(Command::Init { shell, cmd }) => cmd_init(&shell, &cmd)?,
        Some(Command::Tag { project, tags }) => cmd_tag(&config, &project, &tags)?,
        Some(Command::Untag { project, tags }) => cmd_untag(&config, &project, &tags)?,
        Some(Command::Status { json }) => cmd_status(&config, json)?,
        Some(Command::Gc { dry_run }) => cmd_gc(&config, dry_run)?,
        Some(Command::Clean {
            project,
            all,
            dry_run,
        }) => cmd_clean(&config, project.as_deref(), all, dry_run)?,
        Some(Command::Run {
            cmd,
            project,
            tag,
            all,
        }) => cmd_run(&config, &cmd, project.as_deref(), tag.as_deref(), all)?,
        Some(Command::Export { output, base_dir }) => {
            cmd_export(&config, output.as_deref(), base_dir.as_deref())?
        }
        Some(Command::Import { file, base_dir }) => {
            cmd_import(&config, &file, base_dir.as_deref())?
        }
    }

    Ok(())
}

fn cmd_add(config: &Config, path: Option<PathBuf>, name: Option<&str>) -> color_eyre::Result<()> {
    let path = path.unwrap_or_else(|| std::env::current_dir().expect("could not get cwd"));
    let mut db = ProjectDatabase::load(config)?;
    let project = db.register(&path, name)?;
    eprintln!(
        "Added project: {} ({})",
        project.name,
        project.path.display()
    );
    db.save(config)?;
    Ok(())
}

fn cmd_scan(config: &Config, dir: &Path, depth: usize) -> color_eyre::Result<()> {
    let dir = dir
        .canonicalize()
        .map_err(|_| PrjError::PathNotFound(dir.to_path_buf()))?;

    if !dir.is_dir() {
        return Err(PrjError::NotADirectory(dir).into());
    }

    let mut db = ProjectDatabase::load(config)?;
    let found = detect::scan_projects(&dir, depth);

    let mut added = 0;
    for path in &found {
        match db.register(path, None) {
            Ok(p) => {
                eprintln!("  + {}", p.name);
                added += 1;
            }
            Err(PrjError::ProjectAlreadyExists(_)) => {}
            Err(e) => {
                eprintln!("  ! {}: {e}", path.display());
            }
        }
    }

    db.save(config)?;
    eprintln!(
        "Scan complete: found {} projects, added {} new",
        found.len(),
        added
    );
    Ok(())
}

fn cmd_new(config: &Config, git_args: &str) -> color_eyre::Result<()> {
    let args = shell_words::split(git_args)
        .map_err(|e| PrjError::CloneFailed(format!("failed to parse args: {e}")))?;

    let status = process::Command::new("git")
        .arg("clone")
        .args(&args)
        .status()?;

    if !status.success() {
        return Err(
            PrjError::CloneFailed("git clone exited with non-zero status".to_string()).into(),
        );
    }

    let dest = determine_clone_dest(&args)?;

    let mut db = ProjectDatabase::load(config)?;
    let project = db.register(&dest, None)?;
    eprintln!("Registered: {} ({})", project.name, project.path.display());
    db.save(config)?;
    Ok(())
}

fn determine_clone_dest(args: &[String]) -> Result<PathBuf, PrjError> {
    let non_flag_args: Vec<&String> = args.iter().filter(|a| !a.starts_with('-')).collect();

    match non_flag_args.len() {
        0 => Err(PrjError::CloneDestUnknown("no URL provided".to_string())),
        1 => {
            let url = non_flag_args[0];
            let name = url
                .rsplit('/')
                .next()
                .unwrap_or(url)
                .trim_end_matches(".git");
            if name.is_empty() {
                return Err(PrjError::CloneDestUnknown(url.clone()));
            }
            let dest = std::env::current_dir()?.join(name);
            Ok(dest)
        }
        _ => {
            let dest = PathBuf::from(non_flag_args.last().unwrap());
            Ok(dest)
        }
    }
}

fn cmd_remove(config: &Config, name: &str) -> color_eyre::Result<()> {
    let mut db = ProjectDatabase::load(config)?;
    let removed = db.remove(name)?;
    db.save(config)?;
    eprintln!(
        "Removed project: {} ({})",
        removed.name,
        removed.path.display()
    );
    Ok(())
}

fn cmd_list(config: &Config, plain: bool, tag: Option<&str>) -> color_eyre::Result<()> {
    let mut db = ProjectDatabase::load(config)?;

    // Filter by tag if specified
    if let Some(tag) = tag {
        db.projects.retain(|p| p.tags.iter().any(|t| t == tag));
    }

    if plain || !std::io::stdout().is_terminal() {
        if db.projects.is_empty() {
            eprintln!("No projects registered. Use `prj add` or `prj scan` to add projects.");
            return Ok(());
        }
        for p in &db.projects {
            let vcs = p
                .vcs
                .iter()
                .map(|v| v.to_string())
                .collect::<Vec<_>>()
                .join(",");
            let bs = p
                .build_systems
                .iter()
                .map(|b| b.to_string())
                .collect::<Vec<_>>()
                .join(",");
            let tags = if p.tags.is_empty() {
                "-".to_string()
            } else {
                p.tags.join(",")
            };
            println!(
                "{}\t{}\t{}\t{}\t{}",
                p.name,
                p.path.display(),
                if vcs.is_empty() { "-" } else { &vcs },
                if bs.is_empty() { "-" } else { &bs },
                tags,
            );
        }
    } else if let Some(path) = crate::tui::run_list(&mut db.projects, config)? {
        println!("{}", path.display());
    }

    Ok(())
}

fn cmd_stats(config: &Config, project: Option<&str>, json: bool) -> color_eyre::Result<()> {
    let db = ProjectDatabase::load(config)?;

    if let Some(name) = project {
        let proj = db
            .find(name)
            .ok_or_else(|| PrjError::ProjectNotFound(name.to_string()))?;
        let s = stats::collect_project_stats(proj);
        if json {
            println!("{}", serde_json::to_string_pretty(&s)?);
        } else {
            print_project_stats(&s);
        }
    } else {
        let overview = stats::collect_overview_stats(&db.projects);
        if json {
            println!("{}", serde_json::to_string_pretty(&overview)?);
        } else {
            print_overview_stats(&overview);
        }
    }

    Ok(())
}

fn print_project_stats(s: &stats::ProjectStats) {
    eprintln!("Project: {}", s.name);
    eprintln!();

    if let Some(git) = &s.git {
        let branch = git.branch.as_deref().unwrap_or("(detached)");
        let status = if git.is_dirty { "dirty" } else { "clean" };
        eprintln!("  Git: {branch} ({status})");
        if git.is_dirty {
            eprintln!(
                "    changed: {}, staged: {}, untracked: {}",
                git.changed, git.staged, git.untracked
            );
        }
        if git.ahead > 0 || git.behind > 0 {
            eprintln!("    ahead: {}, behind: {}", git.ahead, git.behind);
        }
    }

    eprintln!();
    eprintln!("  Lines of Code: {}", s.loc.total_code);
    for (lang, ls) in &s.loc.languages {
        eprintln!(
            "    {lang}: {} code, {} comments, {} blanks ({} files)",
            ls.code, ls.comments, ls.blanks, ls.files
        );
    }

    eprintln!();
    eprintln!(
        "  Disk: {} total, {} artifacts",
        s.disk.total_display(),
        s.disk.artifact_display()
    );
}

fn print_overview_stats(o: &stats::OverviewStats) {
    eprintln!("Projects: {}", o.total_projects);
    eprintln!("Total code lines: {}", o.total_code_lines);
    eprintln!(
        "Total disk: {}, artifacts: {}",
        ByteSize(o.total_disk_bytes),
        ByteSize(o.total_artifact_bytes)
    );
    eprintln!("Dirty projects: {}", o.dirty_projects);
    eprintln!();

    eprintln!(
        "  {:<20} {:<12} {:<10} {:<10} {:<10}",
        "Name", "Branch", "Status", "LOC", "Disk"
    );
    eprintln!("  {}", "-".repeat(62));

    for s in &o.projects {
        let branch = s
            .git
            .as_ref()
            .and_then(|g| g.branch.as_deref())
            .unwrap_or("-");
        let status = s
            .git
            .as_ref()
            .map(|g| if g.is_dirty { "dirty" } else { "clean" })
            .unwrap_or("-");

        eprintln!(
            "  {:<20} {:<12} {:<10} {:<10} {:<10}",
            s.name,
            branch,
            status,
            s.loc.total_code,
            s.disk.total_display(),
        );
    }
}

fn cmd_init(shell: &str, cmd: &str) -> color_eyre::Result<()> {
    let script = crate::shell::generate_init(shell, cmd)?;
    print!("{script}");
    Ok(())
}

fn cmd_picker(config: &Config) -> color_eyre::Result<()> {
    let db = ProjectDatabase::load(config)?;
    if db.projects.is_empty() {
        eprintln!("No projects registered. Use `prj add` or `prj scan` to add projects.");
        return Ok(());
    }
    if let Some(path) = crate::tui::run_picker(&db.projects)? {
        println!("{}", path.display());
    }
    Ok(())
}

// --- Phase 1: Tags ---

fn cmd_tag(config: &Config, project: &str, tags: &[String]) -> color_eyre::Result<()> {
    let mut db = ProjectDatabase::load(config)?;
    db.add_tags(project, tags)?;
    let p = db.find(project).expect("project was just found");
    eprintln!("Tags for {}: {}", p.name, p.tags.join(", "));
    db.save(config)?;
    Ok(())
}

fn cmd_untag(config: &Config, project: &str, tags: &[String]) -> color_eyre::Result<()> {
    let mut db = ProjectDatabase::load(config)?;
    db.remove_tags(project, tags)?;
    let p = db.find(project).expect("project was just found");
    let tag_display = if p.tags.is_empty() {
        "(none)".to_string()
    } else {
        p.tags.join(", ")
    };
    eprintln!("Tags for {}: {tag_display}", p.name);
    db.save(config)?;
    Ok(())
}

// --- Phase 3: Status ---

fn cmd_status(config: &Config, json: bool) -> color_eyre::Result<()> {
    use rayon::prelude::*;
    use serde::Serialize;

    let db = ProjectDatabase::load(config)?;

    if db.projects.is_empty() {
        eprintln!("No projects registered.");
        return Ok(());
    }

    #[derive(Serialize)]
    struct StatusEntry {
        name: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        branch: Option<String>,
        status: String,
        changed: usize,
        staged: usize,
        untracked: usize,
        ahead: usize,
        behind: usize,
    }

    let entries: Vec<StatusEntry> = db
        .projects
        .par_iter()
        .map(|p| {
            let git = stats::collect_git_status(&p.path);
            match git {
                Some(g) => StatusEntry {
                    name: p.name.clone(),
                    branch: g.branch,
                    status: if g.is_dirty {
                        "dirty".to_string()
                    } else {
                        "clean".to_string()
                    },
                    changed: g.changed,
                    staged: g.staged,
                    untracked: g.untracked,
                    ahead: g.ahead,
                    behind: g.behind,
                },
                None => StatusEntry {
                    name: p.name.clone(),
                    branch: None,
                    status: "no-vcs".to_string(),
                    changed: 0,
                    staged: 0,
                    untracked: 0,
                    ahead: 0,
                    behind: 0,
                },
            }
        })
        .collect();

    if json {
        println!("{}", serde_json::to_string_pretty(&entries)?);
        return Ok(());
    }

    eprintln!(
        "  {:<20} {:<15} {:<10} {:<8} {:<8} {:<10} {:<10}",
        "Name", "Branch", "Status", "Changed", "Staged", "Untracked", "Ahead/Behind"
    );
    eprintln!("  {}", "-".repeat(81));

    for e in &entries {
        let branch = e.branch.as_deref().unwrap_or("-");
        let status_color = match e.status.as_str() {
            "clean" => "\x1b[32m", // green
            "dirty" => {
                if e.staged > 0 {
                    "\x1b[31m" // red
                } else {
                    "\x1b[33m" // yellow
                }
            }
            _ => "\x1b[37m", // white
        };
        let reset = "\x1b[0m";
        let ahead_behind = if e.ahead > 0 || e.behind > 0 {
            format!("{}↑ {}↓", e.ahead, e.behind)
        } else {
            "-".to_string()
        };
        eprintln!(
            "  {:<20} {:<15} {status_color}{:<10}{reset} {:<8} {:<8} {:<10} {:<10}",
            e.name, branch, e.status, e.changed, e.staged, e.untracked, ahead_behind
        );
    }

    Ok(())
}

// --- Phase 4: GC ---

fn cmd_gc(config: &Config, dry_run: bool) -> color_eyre::Result<()> {
    let mut db = ProjectDatabase::load(config)?;
    let orphaned = db.find_orphaned();

    if orphaned.is_empty() {
        eprintln!("No orphaned projects found.");
        return Ok(());
    }

    eprintln!("Orphaned projects (path no longer exists):");
    for p in &orphaned {
        eprintln!("  {} ({})", p.name, p.path.display());
    }

    if dry_run {
        eprintln!("\nDry run: {} projects would be removed.", orphaned.len());
        return Ok(());
    }

    eprint!("\nRemove {} orphaned projects? [y/N] ", orphaned.len());
    io::stderr().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    if input.trim().eq_ignore_ascii_case("y") {
        let removed = db.remove_orphaned();
        db.save(config)?;
        eprintln!("Removed {} orphaned projects.", removed.len());
    } else {
        eprintln!("Cancelled.");
    }

    Ok(())
}

// --- Phase 5: Clean ---

fn cmd_clean(
    config: &Config,
    project: Option<&str>,
    all: bool,
    dry_run: bool,
) -> color_eyre::Result<()> {
    let db = ProjectDatabase::load(config)?;

    let targets: Vec<_> = if let Some(name) = project {
        let p = db
            .find(name)
            .ok_or_else(|| PrjError::ProjectNotFound(name.to_string()))?;
        vec![p]
    } else if all {
        db.projects.iter().collect()
    } else {
        return Err(PrjError::NoTargetProjects.into());
    };

    let mut total_freed = 0u64;

    for p in &targets {
        if p.artifact_dirs.is_empty() {
            continue;
        }
        let preview = clean::preview_clean(&p.path, &p.artifact_dirs);
        if preview.dirs.is_empty() {
            continue;
        }

        eprintln!("{}:", p.name);
        for (dir, size) in &preview.dirs {
            eprintln!("  {dir}: {}", ByteSize(*size));
        }
        eprintln!("  Total: {}", ByteSize(preview.total_bytes));

        if !dry_run {
            match clean::execute_clean(&p.path, &p.artifact_dirs) {
                Ok(bytes) => {
                    total_freed += bytes;
                    eprintln!("  -> Cleaned");
                }
                Err(e) => {
                    eprintln!("  -> Error: {e}");
                }
            }
        }
        eprintln!();
    }

    if dry_run {
        eprintln!("Dry run complete. No files were deleted.");
    } else {
        eprintln!("Total freed: {}", ByteSize(total_freed));
    }

    Ok(())
}

// --- Phase 6: Run ---

fn cmd_run(
    config: &Config,
    cmd: &str,
    project: Option<&str>,
    tag: Option<&str>,
    all: bool,
) -> color_eyre::Result<()> {
    let db = ProjectDatabase::load(config)?;

    let targets: Vec<_> = if let Some(name) = project {
        let p = db
            .find(name)
            .ok_or_else(|| PrjError::ProjectNotFound(name.to_string()))?;
        vec![p]
    } else if let Some(tag) = tag {
        let filtered: Vec<_> = db
            .projects
            .iter()
            .filter(|p| p.tags.contains(&tag.to_string()))
            .collect();
        if filtered.is_empty() {
            eprintln!("No projects found with tag: {tag}");
            return Ok(());
        }
        filtered
    } else if all {
        db.projects.iter().collect()
    } else {
        return Err(PrjError::NoTargetProjects.into());
    };

    for (i, p) in targets.iter().enumerate() {
        if i > 0 {
            eprintln!();
        }
        eprintln!("=== {} ({}) ===", p.name, p.path.display());

        #[cfg(windows)]
        let status = process::Command::new("cmd")
            .args(["/C", cmd])
            .current_dir(&p.path)
            .status();
        #[cfg(not(windows))]
        let status = process::Command::new("sh")
            .arg("-c")
            .arg(cmd)
            .current_dir(&p.path)
            .status();

        match status {
            Ok(s) => {
                if !s.success() {
                    eprintln!("[{}] exited with code {}", p.name, s.code().unwrap_or(-1));
                }
            }
            Err(e) => {
                eprintln!("[{}] failed to execute: {e}", p.name);
            }
        }
    }

    Ok(())
}

// --- Phase 7: Export/Import ---

fn cmd_export(
    config: &Config,
    output: Option<&std::path::Path>,
    base_dir: Option<&std::path::Path>,
) -> color_eyre::Result<()> {
    let db = ProjectDatabase::load(config)?;

    if db.projects.is_empty() {
        eprintln!("No projects to export.");
        return Ok(());
    }

    let m = manifest::export(&db.projects, base_dir);
    let content = manifest::serialize(&m)
        .map_err(|e| PrjError::Manifest(format!("serialization failed: {e}")))?;

    if let Some(path) = output {
        std::fs::write(path, &content)?;
        eprintln!(
            "Exported {} projects to {}",
            m.projects.len(),
            path.display()
        );
    } else {
        print!("{content}");
    }

    Ok(())
}

fn cmd_import(
    config: &Config,
    file: &std::path::Path,
    base_dir: Option<&std::path::Path>,
) -> color_eyre::Result<()> {
    let content = std::fs::read_to_string(file)
        .map_err(|e| PrjError::Manifest(format!("could not read {}: {e}", file.display())))?;
    let m = manifest::parse(&content)
        .map_err(|e| PrjError::Manifest(format!("invalid manifest: {e}")))?;

    let mut db = ProjectDatabase::load(config)?;
    let targets = manifest::import_targets(&m, base_dir);

    let mut cloned = 0;
    let mut skipped = 0;

    for (entry, target_path) in &targets {
        if target_path.exists() {
            eprintln!(
                "  skip {} (already exists: {})",
                entry.name,
                target_path.display()
            );
            skipped += 1;
            continue;
        }

        if let Some(url) = &entry.remote_url {
            eprintln!("  cloning {} -> {}", entry.name, target_path.display());
            if let Some(parent) = target_path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            let status = process::Command::new("git")
                .arg("clone")
                .arg(url)
                .arg(target_path)
                .status()?;

            if status.success() {
                match db.register(target_path, Some(&entry.name)) {
                    Ok(_) => {
                        if !entry.tags.is_empty() {
                            let _ = db.add_tags(&entry.name, &entry.tags);
                        }
                        cloned += 1;
                    }
                    Err(e) => {
                        eprintln!("    warning: cloned but failed to register: {e}");
                    }
                }
            } else {
                eprintln!("    warning: git clone failed for {}", entry.name);
            }
        } else {
            eprintln!("  skip {} (no remote URL)", entry.name);
            skipped += 1;
        }
    }

    db.save(config)?;
    eprintln!("\nImport complete: cloned {cloned}, skipped {skipped}");

    Ok(())
}
