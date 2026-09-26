use crate::app::{App, Mode};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

/// Version codename for the current release.
pub const VERSION_CODENAME: &str = "v1.0.0 \"White Mesa\"";

pub fn render(f: &mut Frame, app: &App) {
    let bg = app.get_theme_bg();
    let fg = app.get_theme_fg();

    let area = f.area();

    // Apply background
    let bg_block = Block::default().style(Style::default().bg(bg).fg(fg));
    f.render_widget(bg_block, area);

    match app.mode {
        Mode::Startup => render_startup(f, app, area),
        Mode::Editor => render_editor(f, app, area),
        Mode::FolderSelect => render_folder_select(f, app, area),
        Mode::Focus => render_focus(f, app, area),
        Mode::FileBrowser => render_file_browser(f, app, area),
        Mode::Search => render_search(f, app, area),
        Mode::RecentNotes => render_recent(f, app, area),
        Mode::Settings => render_settings(f, app, area),
        Mode::CommandPalette => render_command_palette(f, app, area),
        Mode::Help => render_help(f, app, area),
        Mode::RecoveryPrompt => render_recovery(f, app, area),
        Mode::Goals => render_goals(f, app, area),
    }
}

fn render_startup(f: &mut Frame, app: &App, area: Rect) {
    let dim = app.get_dim_color();
    let fg = app.get_theme_fg();
    let accent = app.get_accent_color();

    // Center vertically, reserving bottom rows for the version codename
    let v_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Fill(1),
            Constraint::Length(1),                           // title
            Constraint::Length(1),                           // tagline
            Constraint::Length(1),                           // spacing
            Constraint::Length(app.menu.items.len() as u16), // menu
            Constraint::Fill(1),                             // padding above version
            Constraint::Length(1),                           // gap
            Constraint::Length(1),                           // version codename
        ])
        .split(area);

    // Title
    let title = Paragraph::new(Line::from(vec![Span::styled(
        "microwriter",
        Style::default().fg(fg).add_modifier(Modifier::BOLD),
    )]))
    .alignment(Alignment::Center);
    f.render_widget(title, v_chunks[1]);

    // Tagline
    let tagline = Paragraph::new(Line::from(vec![Span::styled(
        "distraction-free writing environment",
        Style::default().fg(dim),
    )]))
    .alignment(Alignment::Center);
    f.render_widget(tagline, v_chunks[2]);

    // Menu
    let menu_lines: Vec<Line> = app
        .menu
        .items
        .iter()
        .enumerate()
        .map(|(i, item)| {
            let is_selected = i == app.menu.index;
            let prefix = if is_selected { "> " } else { "  " };
            let item_style = if is_selected {
                Style::default().fg(accent)
            } else {
                Style::default().fg(dim)
            };
            Line::from(vec![
                Span::styled(prefix, item_style),
                Span::styled(&item.label, item_style),
            ])
        })
        .collect();

    let menu = Paragraph::new(menu_lines).alignment(Alignment::Center);
    f.render_widget(menu, v_chunks[4]);

    // Version codename near the bottom with a one-line gap
    let version = Paragraph::new(Line::from(vec![Span::styled(
        VERSION_CODENAME,
        Style::default().fg(dim),
    )]))
    .alignment(Alignment::Center);
    f.render_widget(version, v_chunks[7]);
}

fn render_editor(f: &mut Frame, app: &App, area: Rect) {
    let fg = app.get_theme_fg();
    let dim = app.get_dim_color();
    let status_visible = app.status_visible();
    let line_number_width = match app.config.line_numbers.as_str() {
        "off" => 0u16,
        _ => 6u16,
    };

    let mut constraints = vec![Constraint::Fill(1)];
    if status_visible {
        constraints.push(Constraint::Length(1));
    }
    let v_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(area);

    let editor_area = v_chunks[0];

    // Render lines
    let visible_lines: Vec<Line> = app
        .editor
        .lines
        .iter()
        .enumerate()
        .skip(app.editor.visible_start())
        .take(editor_area.height as usize)
        .map(|(i, line)| {
            let line_num = i + 1;
            let cursor_line = app.editor.cursor_row + 1;
            let is_cursor_line = i == app.editor.cursor_row;

            let mut spans = Vec::new();

            // Line numbers
            if line_number_width > 0 {
                let num_str = match app.config.line_numbers.as_str() {
                    "relative" => {
                        if is_cursor_line {
                            format!("{:>4}  ", line_num)
                        } else {
                            let diff = line_num.abs_diff(cursor_line);
                            format!("{:>4}  ", diff)
                        }
                    }
                    "absolute" => format!("{:>4}  ", line_num),
                    _ => String::new(),
                };
                spans.push(Span::styled(num_str, Style::default().fg(dim)));
            }

            // Handle cursor rendering
            if is_cursor_line {
                let col = app.editor.cursor_col;
                let before: String = line.chars().take(col).collect();
                let at: String = line.chars().skip(col).take(1).collect();
                let after: String = line.chars().skip(col + 1).collect();
                let cursor_char = if at.is_empty() { " ".to_string() } else { at };
                spans.push(Span::styled(before, Style::default().fg(fg)));
                spans.push(Span::styled(
                    cursor_char,
                    Style::default().fg(fg).bg(Color::DarkGray),
                ));
                // Render ghost suggestion in dim gray after the cursor
                // Show when cursor is at a word boundary (next char is non-word or end of line)
                let at_word_boundary = after.is_empty()
                    || after
                        .chars()
                        .next()
                        .is_some_and(|c| !c.is_alphanumeric() && c != '_');
                if at_word_boundary && !app.editor.ghost_suggestion.is_empty() {
                    spans.push(Span::styled(
                        &app.editor.ghost_suggestion,
                        Style::default().fg(Color::DarkGray),
                    ));
                }
                spans.push(Span::styled(after, Style::default().fg(fg)));
                Line::from(spans)
            } else {
                spans.push(Span::styled(line, Style::default().fg(fg)));
                Line::from(spans)
            }
        })
        .collect();

    let editor_text = Text::from(visible_lines);
    let editor_para = Paragraph::new(editor_text);
    f.render_widget(editor_para, editor_area);

    // Status line
    if status_visible {
        let status_text = format!(
            "{} · {} words · {}",
            app.display_path(),
            app.editor.word_count(),
            if app.editor.modified {
                "modified"
            } else {
                "saved"
            }
        );
        let combined = format!("{} {}", "─".repeat(area.width as usize / 2), status_text);
        let status_line = Paragraph::new(Line::from(vec![Span::styled(
            &combined,
            Style::default().fg(dim),
        )]));
        f.render_widget(status_line, v_chunks[1]);
    }

    // Brief status message (saved notification)
    if !app.status_message.is_empty() {
        if let Some(timer) = app.status_timer {
            if timer.elapsed().as_secs() < 2 {
                let msg_area = Rect {
                    x: area.width.saturating_sub(7),
                    y: area.height.saturating_sub(1),
                    width: 6,
                    height: 1,
                };
                let msg = Paragraph::new(Line::from(vec![Span::styled(
                    &app.status_message,
                    Style::default().fg(dim),
                )]))
                .alignment(Alignment::Right);
                f.render_widget(msg, msg_area);
            }
        }
    }
}

fn render_folder_select(f: &mut Frame, app: &App, area: Rect) {
    let fg = app.get_theme_fg();
    let dim = app.get_dim_color();
    let accent = app.get_accent_color();

    let v_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // title
            Constraint::Length(1), // breadcrumb
            Constraint::Length(1), // separator
            Constraint::Fill(1),   // folders
            Constraint::Length(1), // filter / hint
        ])
        .split(area);

    // Title
    let title = Paragraph::new(Line::from(vec![Span::styled(
        "new note — select folder",
        Style::default().fg(dim),
    )]))
    .alignment(Alignment::Center);
    f.render_widget(title, v_chunks[0]);

    // Breadcrumb
    let breadcrumb = Paragraph::new(Line::from(vec![Span::styled(
        &app.folder_select.path,
        Style::default().fg(dim),
    )]))
    .alignment(Alignment::Left);
    f.render_widget(breadcrumb, v_chunks[1]);

    // Separator
    let sep = Paragraph::new(Line::from(vec![Span::styled(
        "─".repeat(area.width as usize),
        Style::default().fg(dim),
    )]));
    f.render_widget(sep, v_chunks[2]);

    // Folder list
    let lines: Vec<Line> = app
        .folder_select
        .items
        .iter()
        .enumerate()
        .take(v_chunks[3].height as usize)
        .map(|(i, item)| {
            let is_selected = i == app.folder_select.index;
            let prefix = if is_selected { "  > " } else { "    " };
            let style = if item == "create here" {
                if is_selected {
                    Style::default().fg(fg).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(fg)
                }
            } else if item == ".." || item.ends_with('/') {
                Style::default().fg(accent)
            } else if is_selected {
                Style::default().fg(fg)
            } else {
                Style::default().fg(dim)
            };
            Line::from(vec![Span::styled(prefix, style), Span::styled(item, style)])
        })
        .collect();

    let list = Paragraph::new(lines);
    f.render_widget(list, v_chunks[3]);

    // Filter line or hint
    if app.folder_select.filtering {
        let filter_line = Paragraph::new(Line::from(vec![
            Span::styled("/", Style::default().fg(accent)),
            Span::styled(&app.folder_select.filter, Style::default().fg(fg)),
            Span::styled("█", Style::default().fg(fg)),
        ]));
        f.render_widget(filter_line, v_chunks[4]);
    } else {
        let hint = Paragraph::new(Line::from(vec![Span::styled(
            "enter create here  / filter  h home  esc cancel",
            Style::default().fg(dim),
        )]));
        f.render_widget(hint, v_chunks[4]);
    }
}

fn render_focus(f: &mut Frame, app: &App, area: Rect) {
    let fg = app.get_theme_fg();

    let visible_lines: Vec<Line> = app
        .editor
        .lines
        .iter()
        .enumerate()
        .skip(app.editor.visible_start())
        .take(area.height as usize)
        .map(|(i, line)| {
            if i == app.editor.cursor_row {
                let col = app.editor.cursor_col;
                let before: String = line.chars().take(col).collect();
                let at: String = line.chars().skip(col).take(1).collect();
                let after: String = line.chars().skip(col + 1).collect();
                let cursor_char = if at.is_empty() { " ".to_string() } else { at };
                let mut spans = vec![
                    Span::styled(before, Style::default().fg(fg)),
                    Span::styled(cursor_char, Style::default().fg(fg).bg(Color::DarkGray)),
                ];
                if after.is_empty() && !app.editor.ghost_suggestion.is_empty() {
                    spans.push(Span::styled(
                        &app.editor.ghost_suggestion,
                        Style::default().fg(Color::DarkGray),
                    ));
                }
                spans.push(Span::styled(after, Style::default().fg(fg)));
                Line::from(spans)
            } else {
                Line::from(Span::styled(line, Style::default().fg(fg)))
            }
        })
        .collect();

    let text = Text::from(visible_lines);
    let para = Paragraph::new(text);
    f.render_widget(para, area);
}

fn render_file_browser(f: &mut Frame, app: &App, area: Rect) {
    let fg = app.get_theme_fg();
    let dim = app.get_dim_color();
    let accent = app.get_accent_color();

    let v_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // breadcrumb
            Constraint::Length(1), // separator
            Constraint::Fill(1),
            Constraint::Length(1), // search bar
        ])
        .split(area);

    // Breadcrumb
    let breadcrumb = Paragraph::new(Line::from(vec![Span::styled(
        &app.browser.path,
        Style::default().fg(dim),
    )]))
    .alignment(Alignment::Left);
    f.render_widget(breadcrumb, v_chunks[0]);

    // Separator
    let sep = Paragraph::new(Line::from(vec![Span::styled(
        "─".repeat(area.width as usize),
        Style::default().fg(dim),
    )]));
    f.render_widget(sep, v_chunks[1]);

    // Files
    let file_lines: Vec<Line> = app
        .browser
        .items
        .iter()
        .enumerate()
        .take(v_chunks[2].height as usize)
        .map(|(i, item)| {
            let is_selected = i == app.browser.index;
            let prefix = if is_selected { "  > " } else { "    " };
            let style = if item.ends_with('/') || item == ".." {
                Style::default().fg(accent)
            } else if is_selected {
                Style::default().fg(fg)
            } else {
                Style::default().fg(dim)
            };
            Line::from(vec![Span::styled(prefix, style), Span::styled(item, style)])
        })
        .collect();

    let file_list = Paragraph::new(file_lines);
    f.render_widget(file_list, v_chunks[2]);

    // Search bar
    if app.browser.searching {
        let search_line = Paragraph::new(Line::from(vec![
            Span::styled("/", Style::default().fg(accent)),
            Span::styled(&app.browser.search, Style::default().fg(fg)),
            Span::styled("█", Style::default().fg(fg)),
        ]));
        f.render_widget(search_line, v_chunks[3]);
    } else {
        let hint = Paragraph::new(Line::from(vec![Span::styled(
            "/ search  tab complete  h home  enter open  esc back",
            Style::default().fg(dim),
        )]));
        f.render_widget(hint, v_chunks[3]);
    }
}

fn render_search(f: &mut Frame, app: &App, area: Rect) {
    let fg = app.get_theme_fg();
    let dim = app.get_dim_color();
    let accent = app.get_accent_color();

    let v_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Fill(1),
            Constraint::Length(1), // label
            Constraint::Length(1), // query
            Constraint::Fill(1),   // results
            Constraint::Length(1), // gap
            Constraint::Length(1), // version
        ])
        .split(area);

    let label = Paragraph::new(Line::from(vec![Span::styled(
        "search",
        Style::default().fg(dim),
    )]))
    .alignment(Alignment::Center);
    f.render_widget(label, v_chunks[1]);

    let query_line = Paragraph::new(Line::from(vec![
        Span::styled("> ", Style::default().fg(accent)),
        Span::styled(&app.search.query, Style::default().fg(fg)),
        Span::styled("█", Style::default().fg(fg)),
    ]))
    .alignment(Alignment::Center);
    f.render_widget(query_line, v_chunks[2]);

    let results: Vec<Line> = app
        .search
        .results
        .iter()
        .enumerate()
        .take(v_chunks[3].height as usize)
        .map(|(i, result)| {
            let is_selected = i == app.search.index;
            let style = if is_selected {
                Style::default().fg(fg)
            } else {
                Style::default().fg(dim)
            };
            let prefix = if is_selected { "> " } else { "  " };
            let display = result.split('|').next().unwrap_or(result);
            Line::from(vec![
                Span::styled(prefix, style),
                Span::styled(display, style),
            ])
        })
        .collect();

    if results.is_empty() && !app.search.query.is_empty() {
        let empty = Paragraph::new(Line::from(vec![Span::styled(
            "no results",
            Style::default().fg(dim),
        )]))
        .alignment(Alignment::Center);
        f.render_widget(empty, v_chunks[3]);
    } else {
        let results_para = Paragraph::new(results).alignment(Alignment::Center);
        f.render_widget(results_para, v_chunks[3]);
    }

    // Version codename
    let version = Paragraph::new(Line::from(vec![Span::styled(
        VERSION_CODENAME,
        Style::default().fg(dim),
    )]))
    .alignment(Alignment::Center);
    f.render_widget(version, v_chunks[5]);
}

fn render_recent(f: &mut Frame, app: &App, area: Rect) {
    let fg = app.get_theme_fg();
    let dim = app.get_dim_color();
    let accent = app.get_accent_color();

    let v_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Fill(1),   // top padding
            Constraint::Length(1), // title
            Constraint::Length(1), // separator
            Constraint::Fill(1),   // list
            Constraint::Length(1), // gap
            Constraint::Length(1), // version
        ])
        .split(area);

    let title = Paragraph::new(Line::from(vec![Span::styled(
        "recent notes",
        Style::default().fg(dim),
    )]))
    .alignment(Alignment::Center);
    f.render_widget(title, v_chunks[1]);

    let sep = Paragraph::new(Line::from(vec![Span::styled(
        "─".repeat(area.width as usize),
        Style::default().fg(dim),
    )]))
    .alignment(Alignment::Center);
    f.render_widget(sep, v_chunks[2]);

    // Group by time period
    let now = chrono::Utc::now();
    let mut today = Vec::new();
    let mut yesterday = Vec::new();
    let mut this_week = Vec::new();
    let mut older = Vec::new();

    for entry in &app.storage.recent_files {
        let parsed = chrono::DateTime::parse_from_rfc3339(&entry.accessed)
            .ok()
            .map(|dt| dt.with_timezone(&chrono::Utc));
        if let Some(dt) = parsed {
            let days = (now - dt).num_days();
            if days == 0 {
                today.push(entry.display_name.clone());
            } else if days == 1 {
                yesterday.push(entry.display_name.clone());
            } else if days < 7 {
                this_week.push(entry.display_name.clone());
            } else {
                older.push(entry.display_name.clone());
            }
        } else {
            older.push(entry.display_name.clone());
        }
    }

    let mut lines: Vec<Line> = Vec::new();
    let mut global_idx = 0;
    let recent_idx = app.recent_index;

    let ctx = RecentGroupCtx { fg, dim, accent };
    add_recent_group(
        &mut lines,
        &mut global_idx,
        recent_idx,
        "today",
        &today,
        &ctx,
    );
    add_recent_group(
        &mut lines,
        &mut global_idx,
        recent_idx,
        "yesterday",
        &yesterday,
        &ctx,
    );
    add_recent_group(
        &mut lines,
        &mut global_idx,
        recent_idx,
        "this week",
        &this_week,
        &ctx,
    );
    add_recent_group(
        &mut lines,
        &mut global_idx,
        recent_idx,
        "older",
        &older,
        &ctx,
    );

    if lines.is_empty() {
        lines.push(Line::from(vec![Span::styled(
            "no recent notes",
            Style::default().fg(dim),
        )]));
    }

    let recent_para = Paragraph::new(lines).alignment(Alignment::Center);
    f.render_widget(recent_para, v_chunks[3]);

    // Version codename
    let version = Paragraph::new(Line::from(vec![Span::styled(
        VERSION_CODENAME,
        Style::default().fg(dim),
    )]))
    .alignment(Alignment::Center);
    f.render_widget(version, v_chunks[5]);
}

fn render_settings(f: &mut Frame, app: &App, area: Rect) {
    let fg = app.get_theme_fg();
    let dim = app.get_dim_color();
    let accent = app.get_accent_color();

    let v_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Fill(1),   // top padding
            Constraint::Length(1), // title
            Constraint::Length(1), // separator
            Constraint::Fill(1),   // settings
            Constraint::Length(1), // hint
            Constraint::Length(1), // gap
            Constraint::Length(1), // version
        ])
        .split(area);

    let title = Paragraph::new(Line::from(vec![Span::styled(
        "settings",
        Style::default().fg(dim),
    )]))
    .alignment(Alignment::Center);
    f.render_widget(title, v_chunks[1]);

    let sep = Paragraph::new(Line::from(vec![Span::styled(
        "─".repeat(area.width as usize),
        Style::default().fg(dim),
    )]));
    f.render_widget(sep, v_chunks[2]);

    // Render settings categories + current values
    let lines: Vec<Line> = app
        .settings
        .categories
        .iter()
        .enumerate()
        .map(|(i, cat)| {
            let is_selected = i == app.settings.category_index;
            let value = if i < app.settings.options.len() && !app.settings.options[i].is_empty() {
                let opt_idx = if app.settings.editing && is_selected {
                    app.settings.option_index
                } else {
                    // Find current setting for this specific category
                    let current_val = app.get_setting_value_for_category(i);
                    app.settings.options[i]
                        .iter()
                        .position(|o| *o == current_val)
                        .unwrap_or(0)
                };
                format!(" · {}", app.settings.options[i][opt_idx])
            } else if cat == "default folder" {
                format!(" · {}", app.config.default_folder)
            } else {
                String::new()
            };

            let prefix = if is_selected {
                if app.settings.editing {
                    "* "
                } else {
                    "> "
                }
            } else {
                "  "
            };

            let style = if is_selected {
                Style::default().fg(accent)
            } else {
                Style::default().fg(dim)
            };

            // Show available options inline for the selected (non-editing) category
            let options_hint = if is_selected
                && !app.settings.editing
                && i < app.settings.options.len()
                && !app.settings.options[i].is_empty()
            {
                format!("  [{}]", app.settings.options[i].join(" / "))
            } else {
                String::new()
            };

            let value_owned = value;
            Line::from(vec![
                Span::styled(prefix, style),
                Span::styled(cat, style),
                Span::styled(value_owned, Style::default().fg(fg)),
                Span::styled(options_hint, Style::default().fg(dim)),
            ])
        })
        .collect();

    let settings_para = Paragraph::new(lines);
    f.render_widget(settings_para, v_chunks[3]);

    let hint = Paragraph::new(Line::from(vec![Span::styled(
        "enter edit  tab complete  esc back  j/k navigate",
        Style::default().fg(dim),
    )]))
    .alignment(Alignment::Center);
    f.render_widget(hint, v_chunks[4]);

    // Version codename
    let version = Paragraph::new(Line::from(vec![Span::styled(
        VERSION_CODENAME,
        Style::default().fg(dim),
    )]))
    .alignment(Alignment::Center);
    f.render_widget(version, v_chunks[6]);
}

fn render_command_palette(f: &mut Frame, app: &App, area: Rect) {
    let fg = app.get_theme_fg();
    let dim = app.get_dim_color();
    let accent = app.get_accent_color();

    // Darken background
    let overlay = Block::default().style(Style::default().bg(Color::Black));
    f.render_widget(overlay, area);

    // Center the palette
    let palette_width = 40u16;
    let palette_height = (app.palette.items.len() as u16 + 3).min(area.height);
    let palette_x = (area.width.saturating_sub(palette_width)) / 2;
    let palette_y = (area.height.saturating_sub(palette_height)) / 2;

    let palette_area = Rect {
        x: area.x + palette_x,
        y: area.y + palette_y,
        width: palette_width,
        height: palette_height,
    };

    // Background for palette
    let bg_block = Block::default()
        .style(Style::default().bg(app.get_theme_bg()))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(dim));
    f.render_widget(bg_block, palette_area);

    let inner = Rect {
        x: palette_area.x + 1,
        y: palette_area.y + 1,
        width: palette_area.width.saturating_sub(2),
        height: palette_area.height.saturating_sub(2),
    };

    let v_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // query
            Constraint::Length(1), // separator
            Constraint::Fill(1),
        ])
        .split(inner);

    let query_line = Paragraph::new(Line::from(vec![
        Span::styled("> ", Style::default().fg(accent)),
        Span::styled(&app.palette.query, Style::default().fg(fg)),
        Span::styled("█", Style::default().fg(fg)),
    ]));
    f.render_widget(query_line, v_chunks[0]);

    let sep = Paragraph::new(Line::from(vec![Span::styled(
        "─".repeat(inner.width as usize),
        Style::default().fg(dim),
    )]));
    f.render_widget(sep, v_chunks[1]);

    let cmd_lines: Vec<Line> = app
        .palette
        .items
        .iter()
        .enumerate()
        .take(v_chunks[2].height as usize)
        .map(|(i, cmd)| {
            let is_selected = i == app.palette.index;
            let style = if is_selected {
                Style::default().fg(fg)
            } else {
                Style::default().fg(dim)
            };
            let prefix = if is_selected { "> " } else { "  " };
            Line::from(vec![Span::styled(prefix, style), Span::styled(cmd, style)])
        })
        .collect();

    let cmd_para = Paragraph::new(cmd_lines);
    f.render_widget(cmd_para, v_chunks[2]);
}

fn render_help(f: &mut Frame, app: &App, area: Rect) {
    let dim = app.get_dim_color();
    let accent = app.get_accent_color();

    let v_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Fill(1),   // top padding
            Constraint::Length(1), // title
            Constraint::Length(1), // separator
            Constraint::Fill(1),   // shortcuts
            Constraint::Length(1), // gap
            Constraint::Length(1), // version
        ])
        .split(area);

    let title = Paragraph::new(Line::from(vec![Span::styled(
        "help",
        Style::default().fg(dim),
    )]))
    .alignment(Alignment::Center);
    f.render_widget(title, v_chunks[1]);

    let sep = Paragraph::new(Line::from(vec![Span::styled(
        "─".repeat(area.width as usize),
        Style::default().fg(dim),
    )]));
    f.render_widget(sep, v_chunks[2]);

    let shortcuts = vec![
        ("ctrl+s", "save"),
        ("ctrl+o", "open"),
        ("ctrl+n", "new"),
        ("ctrl+p", "commands"),
        ("ctrl+q", "quit"),
        ("ctrl+f", "search"),
        ("esc", "menu"),
        ("ctrl+left/right", "jump words"),
        ("ctrl+up/down", "scroll"),
        ("ctrl+home/end", "top/bottom"),
    ];

    let lines: Vec<Line> = shortcuts
        .iter()
        .map(|(key, desc)| {
            Line::from(vec![
                Span::styled(format!("{:>18}    ", key), Style::default().fg(accent)),
                Span::styled(*desc, Style::default().fg(dim)),
            ])
        })
        .collect();

    let help_para = Paragraph::new(lines).alignment(Alignment::Center);
    f.render_widget(help_para, v_chunks[3]);

    // Version codename
    let version = Paragraph::new(Line::from(vec![Span::styled(
        VERSION_CODENAME,
        Style::default().fg(dim),
    )]))
    .alignment(Alignment::Center);
    f.render_widget(version, v_chunks[5]);
}

fn render_recovery(f: &mut Frame, app: &App, area: Rect) {
    let fg = app.get_theme_fg();
    let dim = app.get_dim_color();
    let accent = app.get_accent_color();

    let v_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Fill(1),
            Constraint::Length(1), // question
            Constraint::Length(1), // spacing
            Constraint::Length(2), // options
            Constraint::Fill(1),
        ])
        .split(area);

    let question = Paragraph::new(Line::from(vec![Span::styled(
        "recover previous session?",
        Style::default().fg(fg),
    )]))
    .alignment(Alignment::Center);
    f.render_widget(question, v_chunks[1]);

    let options: Vec<Line> = ["yes", "no"]
        .iter()
        .enumerate()
        .map(|(i, opt)| {
            let is_selected = i == app.recovery_index;
            let style = if is_selected {
                Style::default().fg(accent)
            } else {
                Style::default().fg(dim)
            };
            let prefix = if is_selected { "> " } else { "  " };
            Line::from(vec![Span::styled(prefix, style), Span::styled(*opt, style)])
        })
        .collect();

    let options_para = Paragraph::new(options).alignment(Alignment::Center);
    f.render_widget(options_para, v_chunks[3]);
}

fn render_goals(f: &mut Frame, app: &App, area: Rect) {
    let fg = app.get_theme_fg();
    let dim = app.get_dim_color();

    let v_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Fill(1),   // top padding
            Constraint::Length(1), // title
            Constraint::Length(1), // separator
            Constraint::Fill(1),   // stats
            Constraint::Length(1), // gap
            Constraint::Length(1), // version
        ])
        .split(area);

    let title = Paragraph::new(Line::from(vec![Span::styled(
        "writing statistics",
        Style::default().fg(dim),
    )]))
    .alignment(Alignment::Center);
    f.render_widget(title, v_chunks[1]);

    let sep = Paragraph::new(Line::from(vec![Span::styled(
        "─".repeat(area.width as usize),
        Style::default().fg(dim),
    )]));
    f.render_widget(sep, v_chunks[2]);

    let words = app.editor.word_count();
    let chars = app.editor.char_count();
    let reading_time = app.editor.reading_time_minutes();

    let stats_lines = vec![
        Line::from(vec![
            Span::styled("word count        ", Style::default().fg(dim)),
            Span::styled(format!("{}", words), Style::default().fg(fg)),
        ]),
        Line::from(vec![
            Span::styled("character count   ", Style::default().fg(dim)),
            Span::styled(format!("{}", chars), Style::default().fg(fg)),
        ]),
        Line::from(vec![
            Span::styled("reading time      ", Style::default().fg(dim)),
            Span::styled(format!("{} min", reading_time), Style::default().fg(fg)),
        ]),
    ];

    let goals_para = Paragraph::new(stats_lines).alignment(Alignment::Center);
    f.render_widget(goals_para, v_chunks[3]);

    // Version codename
    let version = Paragraph::new(Line::from(vec![Span::styled(
        VERSION_CODENAME,
        Style::default().fg(dim),
    )]))
    .alignment(Alignment::Center);
    f.render_widget(version, v_chunks[5]);
}

struct RecentGroupCtx {
    fg: Color,
    dim: Color,
    accent: Color,
}

fn add_recent_group(
    lines: &mut Vec<Line>,
    global_idx: &mut usize,
    search_idx: usize,
    label: &str,
    items: &[String],
    ctx: &RecentGroupCtx,
) {
    if items.is_empty() {
        return;
    }
    lines.push(Line::from(vec![Span::styled(
        "",
        Style::default().fg(ctx.dim),
    )]));
    lines.push(Line::from(vec![Span::styled(
        label.to_string(),
        Style::default().fg(ctx.accent),
    )]));
    for item in items.iter() {
        let is_selected = *global_idx == search_idx;
        let style = if is_selected {
            Style::default().fg(ctx.fg)
        } else {
            Style::default().fg(ctx.dim)
        };
        let prefix = if is_selected { "> " } else { "  " };
        lines.push(Line::from(vec![
            Span::styled(prefix, style),
            Span::styled(item.clone(), style),
        ]));
        *global_idx += 1;
    }
}
