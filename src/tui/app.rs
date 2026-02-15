use std::io;
use std::path::PathBuf;
use std::process;
use std::time::Duration;

use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use crossterm::terminal::{self, EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::{cursor, execute};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;

use super::actions::{self, ListAction};
use super::fuzzy::{FuzzyMatch, FuzzyMatcher};
use super::view;
use crate::core::clean;
use crate::core::config::Config;
use crate::core::project::{Project, ProjectDatabase};
use crate::core::stats::{self, ProjectStats};

pub struct PickerState {
    pub query: String,
    pub filtered: Vec<FuzzyMatch>,
    pub selected: usize,
}

pub enum ListMode {
    Browsing,
    ActionMenu {
        menu_selected: usize,
    },
    ViewingStats {
        stats: ProjectStats,
    },
    Confirming {
        action: &'static str,
        on_confirm: PendingAction,
    },
    CleanResult {
        message: String,
    },
}

pub enum PendingAction {
    Remove,
    CleanArtifacts,
}

pub struct ListState {
    pub selected: usize,
    pub git_statuses: Vec<Option<stats::GitStatus>>,
    pub mode: ListMode,
    pub message: Option<String>,
}

/// Run the fuzzy picker TUI on stderr. Returns the selected project path or None.
pub fn run_picker(projects: &[Project]) -> color_eyre::Result<Option<PathBuf>> {
    let mut stderr = io::stderr();
    execute!(stderr, EnterAlternateScreen, cursor::Hide)?;
    terminal::enable_raw_mode()?;

    let backend = CrosstermBackend::new(io::stderr());
    let mut terminal = Terminal::new(backend)?;

    let names: Vec<String> = projects.iter().map(|p| p.name.clone()).collect();
    let mut matcher = FuzzyMatcher::new();

    let mut state = PickerState {
        query: String::new(),
        filtered: matcher.filter("", &names),
        selected: 0,
    };

    let result = loop {
        terminal.draw(|f| view::render_picker(f, &state, projects))?;

        if event::poll(Duration::from_millis(50))?
            && let Event::Key(key) = event::read()?
        {
            if key.kind != KeyEventKind::Press {
                continue;
            }
            match key.code {
                KeyCode::Esc => break None,
                KeyCode::Enter => {
                    if let Some(fm) = state.filtered.get(state.selected) {
                        break Some(projects[fm.index].path.clone());
                    }
                    break None;
                }
                KeyCode::Up => {
                    if state.selected > 0 {
                        state.selected -= 1;
                    }
                }
                KeyCode::Down => {
                    if state.selected + 1 < state.filtered.len() {
                        state.selected += 1;
                    }
                }
                KeyCode::Backspace => {
                    state.query.pop();
                    state.filtered = matcher.filter(&state.query, &names);
                    state.selected = 0;
                }
                KeyCode::Char(c) => {
                    state.query.push(c);
                    state.filtered = matcher.filter(&state.query, &names);
                    state.selected = 0;
                }
                _ => {}
            }
        }
    };

    terminal::disable_raw_mode()?;
    execute!(io::stderr(), LeaveAlternateScreen, cursor::Show)?;

    Ok(result)
}

/// Run the interactive list TUI on stderr.
/// Returns Some(path) if the user chose "cd to project".
pub fn run_list(
    projects: &mut Vec<Project>,
    config: &Config,
) -> color_eyre::Result<Option<PathBuf>> {
    if projects.is_empty() {
        eprintln!("No projects registered. Use `prj add` or `prj scan` to add projects.");
        return Ok(None);
    }

    let mut stderr = io::stderr();
    execute!(stderr, EnterAlternateScreen, cursor::Hide)?;
    terminal::enable_raw_mode()?;

    let backend = CrosstermBackend::new(io::stderr());
    let mut terminal = Terminal::new(backend)?;

    let git_statuses: Vec<Option<stats::GitStatus>> = projects
        .iter()
        .map(|p| stats::collect_git_status(&p.path))
        .collect();

    let mut state = ListState {
        selected: 0,
        git_statuses,
        mode: ListMode::Browsing,
        message: None,
    };

    let result = loop {
        terminal.draw(|f| view::render_list(f, &state, projects))?;

        if event::poll(Duration::from_millis(50))?
            && let Event::Key(key) = event::read()?
        {
            if key.kind != KeyEventKind::Press {
                continue;
            }

            match &state.mode {
                ListMode::Browsing => match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => break None,
                    KeyCode::Up | KeyCode::Char('k') => {
                        if state.selected > 0 {
                            state.selected -= 1;
                        }
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        if state.selected + 1 < projects.len() {
                            state.selected += 1;
                        }
                    }
                    KeyCode::Enter => {
                        if !projects.is_empty() {
                            state.mode = ListMode::ActionMenu { menu_selected: 0 };
                        }
                    }
                    _ => {}
                },

                ListMode::ActionMenu { menu_selected } => {
                    let menu_selected = *menu_selected;
                    let items = actions::menu_items(&projects[state.selected]);
                    match key.code {
                        KeyCode::Esc => {
                            state.mode = ListMode::Browsing;
                        }
                        KeyCode::Up | KeyCode::Char('k') => {
                            if menu_selected > 0 {
                                state.mode = ListMode::ActionMenu {
                                    menu_selected: menu_selected - 1,
                                };
                            }
                        }
                        KeyCode::Down | KeyCode::Char('j') => {
                            if menu_selected + 1 < items.len() {
                                state.mode = ListMode::ActionMenu {
                                    menu_selected: menu_selected + 1,
                                };
                            }
                        }
                        KeyCode::Enter => {
                            if let Some(item) = items.get(menu_selected) {
                                match item.action {
                                    ListAction::ViewStats => {
                                        let project = &projects[state.selected];
                                        let ps = stats::collect_project_stats(project);
                                        state.mode = ListMode::ViewingStats { stats: ps };
                                    }
                                    ListAction::CleanArtifacts => {
                                        state.mode = ListMode::Confirming {
                                            action: "Clean artifacts",
                                            on_confirm: PendingAction::CleanArtifacts,
                                        };
                                    }
                                    ListAction::OpenEditor => {
                                        let path = projects[state.selected].path.clone();
                                        // Leave alternate screen, run editor, re-enter
                                        terminal::disable_raw_mode()?;
                                        execute!(io::stderr(), LeaveAlternateScreen, cursor::Show)?;

                                        let editor = std::env::var("EDITOR")
                                            .unwrap_or_else(|_| "vi".to_string());
                                        if let Err(e) =
                                            process::Command::new(&editor).arg(&path).status()
                                        {
                                            state.message =
                                                Some(format!("Failed to launch editor: {e}"));
                                        }

                                        execute!(io::stderr(), EnterAlternateScreen, cursor::Hide)?;
                                        terminal::enable_raw_mode()?;
                                        state.mode = ListMode::Browsing;
                                    }
                                    ListAction::OpenExplorer => {
                                        let path = projects[state.selected].path.clone();
                                        let result = {
                                            #[cfg(target_os = "macos")]
                                            {
                                                process::Command::new("open").arg(&path).spawn()
                                            }
                                            #[cfg(target_os = "linux")]
                                            {
                                                process::Command::new("xdg-open").arg(&path).spawn()
                                            }
                                            #[cfg(target_os = "windows")]
                                            {
                                                process::Command::new("explorer").arg(&path).spawn()
                                            }
                                        };
                                        state.mode = ListMode::Browsing;
                                        state.message = Some(match result {
                                            Ok(_) => "Opened in file manager".to_string(),
                                            Err(e) => {
                                                format!("Failed to open file manager: {e}")
                                            }
                                        });
                                    }
                                    ListAction::CdToProject => {
                                        let path = projects[state.selected].path.clone();
                                        break Some(path);
                                    }
                                    ListAction::Remove => {
                                        state.mode = ListMode::Confirming {
                                            action: "Remove project",
                                            on_confirm: PendingAction::Remove,
                                        };
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                }

                ListMode::ViewingStats { .. } => match key.code {
                    KeyCode::Esc | KeyCode::Char('q') | KeyCode::Enter => {
                        state.mode = ListMode::Browsing;
                    }
                    _ => {}
                },

                ListMode::Confirming { on_confirm, .. } => {
                    match key.code {
                        KeyCode::Char('y') | KeyCode::Char('Y') => {
                            match on_confirm {
                                PendingAction::Remove => {
                                    let name = projects[state.selected].name.clone();
                                    projects.remove(state.selected);
                                    // Save database
                                    let db = ProjectDatabase {
                                        projects: projects.clone(),
                                    };
                                    let _ = db.save(config);
                                    // Refresh git statuses
                                    state.git_statuses = projects
                                        .iter()
                                        .map(|p| stats::collect_git_status(&p.path))
                                        .collect();
                                    if state.selected >= projects.len() && !projects.is_empty() {
                                        state.selected = projects.len() - 1;
                                    }
                                    state.message = Some(format!("Removed: {name}"));
                                    state.mode = ListMode::Browsing;
                                    if projects.is_empty() {
                                        break None;
                                    }
                                }
                                PendingAction::CleanArtifacts => {
                                    let project = &projects[state.selected];
                                    match clean::execute_clean(
                                        &project.path,
                                        &project.artifact_dirs,
                                    ) {
                                        Ok(bytes) => {
                                            let msg = format!(
                                                "Cleaned {}: freed {}",
                                                project.name,
                                                bytesize::ByteSize(bytes)
                                            );
                                            state.mode = ListMode::CleanResult { message: msg };
                                        }
                                        Err(e) => {
                                            state.mode = ListMode::CleanResult {
                                                message: format!("Error: {e}"),
                                            };
                                        }
                                    }
                                }
                            }
                        }
                        KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                            state.mode = ListMode::Browsing;
                        }
                        _ => {}
                    }
                }

                ListMode::CleanResult { .. } => match key.code {
                    KeyCode::Esc | KeyCode::Enter => {
                        state.mode = ListMode::Browsing;
                    }
                    _ => {}
                },
            }
        }
    };

    terminal::disable_raw_mode()?;
    execute!(io::stderr(), LeaveAlternateScreen, cursor::Show)?;

    Ok(result)
}
