use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Cell, Clear, List, ListItem, Paragraph, Row, Table, Wrap};

use super::actions;
use super::app::{ListMode, ListState as TuiListState, PickerState};
use prj_core::project::Project;

pub fn render_picker(f: &mut Frame, state: &PickerState, projects: &[Project]) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // input
            Constraint::Min(1),    // results
            Constraint::Length(1), // status bar
        ])
        .split(f.area());

    // Input
    let input = Paragraph::new(Line::from(vec![
        Span::styled("> ", Style::default().fg(Color::Cyan)),
        Span::raw(&state.query),
    ]))
    .block(Block::default().borders(Borders::ALL).title("Search"));

    f.render_widget(input, chunks[0]);

    // Set cursor position
    let cursor_x = chunks[0].x + 2 + state.query.len() as u16;
    let cursor_y = chunks[0].y + 1;
    f.set_cursor_position((cursor_x, cursor_y));

    // Results
    let items: Vec<ListItem> = state
        .filtered
        .iter()
        .enumerate()
        .map(|(i, fm)| {
            let project = &projects[fm.index];
            let style = if i == state.selected {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            let prefix = if i == state.selected { "> " } else { "  " };
            let mut spans = vec![
                Span::styled(prefix, style),
                Span::styled(&project.name, style),
                Span::styled(
                    format!("  {}", project.path.display()),
                    Style::default().fg(Color::DarkGray),
                ),
            ];
            if !project.tags.is_empty() {
                spans.push(Span::styled(
                    format!("  [{}]", project.tags.join(", ")),
                    Style::default().fg(Color::Magenta),
                ));
            }
            ListItem::new(Line::from(spans))
        })
        .collect();

    let list = List::new(items).block(Block::default().borders(Borders::ALL).title("Projects"));
    f.render_widget(list, chunks[1]);

    // Status bar
    let status = Paragraph::new(Line::from(vec![
        Span::styled(
            format!(" {}/{} ", state.filtered.len(), projects.len()),
            Style::default().fg(Color::Cyan),
        ),
        Span::styled(
            " | ESC: cancel | Enter: select",
            Style::default().fg(Color::DarkGray),
        ),
    ]));
    f.render_widget(status, chunks[2]);
}

pub fn render_list(f: &mut Frame, state: &TuiListState, projects: &[Project]) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(1),    // table
            Constraint::Length(1), // status bar
        ])
        .split(f.area());

    // Build rows
    let rows: Vec<Row> = projects
        .iter()
        .enumerate()
        .map(|(i, p)| {
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
                p.tags.join(", ")
            };

            let status_str =
                if let Some(git_status) = &state.git_statuses.get(i).and_then(|s| s.as_ref()) {
                    if git_status.is_dirty {
                        "dirty"
                    } else {
                        "clean"
                    }
                } else if p.vcs.is_empty() {
                    "-"
                } else {
                    "..."
                };

            let style = if i == state.selected {
                Style::default().add_modifier(Modifier::REVERSED)
            } else {
                Style::default()
            };

            Row::new(vec![
                Cell::from(p.name.clone()),
                Cell::from(format!("{}", p.path.display())),
                Cell::from(if vcs.is_empty() { "-".to_string() } else { vcs }),
                Cell::from(if bs.is_empty() { "-".to_string() } else { bs }),
                Cell::from(tags),
                Cell::from(status_str.to_string()),
            ])
            .style(style)
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Percentage(15),
            Constraint::Percentage(30),
            Constraint::Percentage(8),
            Constraint::Percentage(12),
            Constraint::Percentage(15),
            Constraint::Percentage(10),
        ],
    )
    .header(
        Row::new(vec!["Name", "Path", "VCS", "Build", "Tags", "Status"]).style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
    )
    .block(Block::default().borders(Borders::ALL).title("Projects"));

    // Dim the table when overlay is active
    let is_overlay = !matches!(state.mode, ListMode::Browsing);
    if is_overlay {
        let dimmed_table = table.style(Style::default().fg(Color::DarkGray));
        f.render_widget(dimmed_table, chunks[0]);
    } else {
        f.render_widget(table, chunks[0]);
    }

    // Status bar
    let status_text = if let Some(msg) = &state.message {
        msg.clone()
    } else {
        String::new()
    };
    let status = Paragraph::new(Line::from(vec![
        Span::styled(
            format!(" {} projects ", projects.len()),
            Style::default().fg(Color::Cyan),
        ),
        Span::styled(
            " | q: quit | j/k: navigate | Enter: actions",
            Style::default().fg(Color::DarkGray),
        ),
        if !status_text.is_empty() {
            Span::styled(
                format!("  {status_text}"),
                Style::default().fg(Color::Yellow),
            )
        } else {
            Span::raw("")
        },
    ]));
    f.render_widget(status, chunks[1]);

    // Render overlays based on mode
    match &state.mode {
        ListMode::Browsing => {}
        ListMode::ActionMenu { menu_selected } => {
            if let Some(project) = projects.get(state.selected) {
                render_action_menu(f, project, *menu_selected);
            }
        }
        ListMode::ViewingStats { stats } => {
            render_stats_view(f, stats);
        }
        ListMode::Confirming { action, .. } => {
            render_confirm_dialog(f, action);
        }
        ListMode::CleanResult { message } => {
            render_message_popup(f, "Result", message);
        }
    }
}

fn centered_rect(width: u16, height: u16, area: Rect) -> Rect {
    let x = area.x + area.width.saturating_sub(width) / 2;
    let y = area.y + area.height.saturating_sub(height) / 2;
    Rect::new(x, y, width.min(area.width), height.min(area.height))
}

fn render_action_menu(f: &mut Frame, project: &Project, menu_selected: usize) {
    let items = actions::menu_items(project);
    let height = items.len() as u16 + 4; // borders + footer
    let width = 35;
    let area = centered_rect(width, height, f.area());

    f.render_widget(Clear, area);

    let title = format!(" Actions: {} ", project.name);
    let block = Block::default().borders(Borders::ALL).title(title);
    let inner = block.inner(area);
    f.render_widget(block, area);

    let menu_items: Vec<ListItem> = items
        .iter()
        .enumerate()
        .map(|(i, item)| {
            let style = if i == menu_selected {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            let prefix = if i == menu_selected { "> " } else { "  " };
            ListItem::new(Span::styled(format!("{prefix}{}", item.label), style))
        })
        .collect();

    // Split inner into list and footer
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(1)])
        .split(inner);

    let list = List::new(menu_items);
    f.render_widget(list, chunks[0]);

    let footer = Paragraph::new(Span::styled(
        " Enter: select  Esc: cancel",
        Style::default().fg(Color::DarkGray),
    ));
    f.render_widget(footer, chunks[1]);
}

fn render_stats_view(f: &mut Frame, stats: &prj_core::stats::ProjectStats) {
    let width = 60u16.min(f.area().width.saturating_sub(4));
    let height = f.area().height.saturating_sub(4);
    let area = centered_rect(width, height, f.area());

    f.render_widget(Clear, area);

    let title = format!(" Stats: {} ", stats.name);
    let block = Block::default().borders(Borders::ALL).title(title);
    let inner = block.inner(area);
    f.render_widget(block, area);

    let mut lines = Vec::new();

    // Git info
    if let Some(git) = &stats.git {
        let branch = git.branch.as_deref().unwrap_or("(detached)");
        let status = if git.is_dirty { "dirty" } else { "clean" };
        lines.push(Line::from(vec![
            Span::styled("Git: ", Style::default().fg(Color::Cyan)),
            Span::raw(format!("{branch} ({status})")),
        ]));
        if git.is_dirty {
            lines.push(Line::from(format!(
                "  changed: {}, staged: {}, untracked: {}",
                git.changed, git.staged, git.untracked
            )));
        }
        if git.ahead > 0 || git.behind > 0 {
            lines.push(Line::from(format!(
                "  ahead: {}, behind: {}",
                git.ahead, git.behind
            )));
        }
        lines.push(Line::from(""));
    }

    // LOC
    lines.push(Line::from(vec![
        Span::styled("Lines of Code: ", Style::default().fg(Color::Cyan)),
        Span::raw(format!("{}", stats.loc.total_code)),
    ]));
    for (lang, ls) in &stats.loc.languages {
        lines.push(Line::from(format!(
            "  {lang}: {} code, {} comments, {} blanks ({} files)",
            ls.code, ls.comments, ls.blanks, ls.files
        )));
    }
    lines.push(Line::from(""));

    // Disk
    lines.push(Line::from(vec![
        Span::styled("Disk: ", Style::default().fg(Color::Cyan)),
        Span::raw(format!(
            "{} total, {} artifacts",
            stats.disk.total_display(),
            stats.disk.artifact_display()
        )),
    ]));

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "Press Esc/Enter/q to close",
        Style::default().fg(Color::DarkGray),
    )));

    let paragraph = Paragraph::new(lines).wrap(Wrap { trim: false });
    f.render_widget(paragraph, inner);
}

fn render_confirm_dialog(f: &mut Frame, action: &str) {
    let width = 40;
    let height = 5;
    let area = centered_rect(width, height, f.area());

    f.render_widget(Clear, area);

    let block = Block::default()
        .borders(Borders::ALL)
        .title(format!(" {action} "));
    let inner = block.inner(area);
    f.render_widget(block, area);

    let text = Paragraph::new(vec![
        Line::from("Are you sure?"),
        Line::from(""),
        Line::from(Span::styled(
            "y: yes  n/Esc: cancel",
            Style::default().fg(Color::DarkGray),
        )),
    ]);
    f.render_widget(text, inner);
}

fn render_message_popup(f: &mut Frame, title: &str, message: &str) {
    let width = 50u16.min(f.area().width.saturating_sub(4));
    let height = 5;
    let area = centered_rect(width, height, f.area());

    f.render_widget(Clear, area);

    let block = Block::default()
        .borders(Borders::ALL)
        .title(format!(" {title} "));
    let inner = block.inner(area);
    f.render_widget(block, area);

    let text = Paragraph::new(vec![
        Line::from(message.to_string()),
        Line::from(""),
        Line::from(Span::styled(
            "Press Enter/Esc to close",
            Style::default().fg(Color::DarkGray),
        )),
    ]);
    f.render_widget(text, inner);
}
