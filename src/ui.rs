use crate::app::{App, Mode};
use crate::editor::{justify_line, should_justify, Alignment as TextAlign};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use unicode_width::UnicodeWidthStr;

/// Version codename for the current release.
pub const VERSION_CODENAME: &str = "v1.0.0 \"White Mesa\"";

/// Map the document alignment onto ratatui's line alignment. Justified text is
/// widened ahead of time by the editor, so it renders as left-aligned text.
fn ratatui_alignment(alignment: TextAlign) -> Alignment {
    match alignment {
        TextAlign::Center => Alignment::Center,
        TextAlign::Right => Alignment::Right,
        TextAlign::Left | TextAlign::Justify => Alignment::Left,
    }
}

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
    let alignment = app.alignment();

    // Line numbers sit in their own gutter so aligning the text never moves them.
    let (gutter_area, text_area) = if line_number_width > 0 {
        let columns = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Length(line_number_width), Constraint::Fill(1)])
            .split(editor_area);
        (Some(columns[0]), columns[1])
    } else {
        (None, editor_area)
    };
    let text_width = text_area.width as usize;
    // `wrap = false` keeps logical lines whole and slides the view sideways.
    let wrap_width = if app.config.wrap { text_width } else { 0 };
    let cursor_line = app.editor.cursor_row + 1;
    app.editor
        .set_viewport(wrap_width, editor_area.height as usize);
    let mut gutter_lines: Vec<Line> = Vec::new();
    let mut text_lines: Vec<Line> = Vec::new();
    let mut caret_column = 0usize;

    for rendered in app
        .editor
        .visible_rows(editor_area.height as usize, wrap_width)
    {
        let mut text = app
            .editor
            .segment_text(rendered.row, rendered.start, rendered.end);
        let mut caret = rendered.caret;
        // Justified text fills the margin while the paragraph has a row below.
        let justified = alignment == TextAlign::Justify
            && should_justify(&text, text_width, rendered.continues);
        if justified {
            let (stretched, mapped) = justify_line(&text, text_width, caret);
            text = stretched;
            caret = mapped;
        }

        // Wrapped continuation rows leave the numbering gutter blank.
        if line_number_width > 0 {
            let num_str = if rendered.start == 0 {
                let logical = rendered.row + 1;
                match app.config.line_numbers.as_str() {
                    "relative" if logical != cursor_line => {
                        format!("{:>4}  ", logical.abs_diff(cursor_line))
                    }
                    _ => format!("{:>4}  ", logical),
                }
            } else {
                " ".repeat(line_number_width as usize)
            };
            gutter_lines.push(Line::from(Span::styled(num_str, Style::default().fg(dim))));
        }

        // Handle cursor rendering
        let Some(offset) = caret else {
            text_lines.push(Line::from(Span::styled(text, Style::default().fg(fg))));
            continue;
        };
        let before: String = text.chars().take(offset).collect();
        let at: String = text.chars().skip(offset).take(1).collect();
        let after: String = text.chars().skip(offset + 1).collect();
        caret_column = UnicodeWidthStr::width(before.as_str());
        let cursor_char = if at.is_empty() { " ".to_string() } else { at };
        let mut spans = vec![
            Span::styled(before, Style::default().fg(fg)),
            Span::styled(cursor_char, Style::default().fg(fg).bg(Color::DarkGray)),
        ];
        // Render ghost suggestion in dim gray after the cursor, when the
        // cursor is at a word boundary. A justified row has no spare room for
        // the hint, but Tab still accepts the completion.
        let at_word_boundary = after.is_empty()
            || after
                .chars()
                .next()
                .is_some_and(|c| !c.is_alphanumeric() && c != '_');
        if !justified && at_word_boundary && !app.editor.ghost_suggestion.is_empty() {
            spans.push(Span::styled(
                &app.editor.ghost_suggestion,
                Style::default().fg(Color::DarkGray),
            ));
        }
        spans.push(Span::styled(after, Style::default().fg(fg)));
        text_lines.push(Line::from(spans));
    }

    if let Some(gutter) = gutter_area {
        f.render_widget(Paragraph::new(gutter_lines), gutter);
    }
    let h_offset = horizontal_offset(caret_column, text_width, wrap_width > 0);
    let editor_para = Paragraph::new(Text::from(text_lines))
        .alignment(ratatui_alignment(alignment))
        .scroll((0, h_offset));
    f.render_widget(editor_para, text_area);

    // Status line
    if status_visible {
        let alignment_note = match alignment {
            TextAlign::Left => String::new(),
            other => format!(" · {}", other.label()),
        };
        let status_text = format!(
            "{} · {} words{} · {}",
            app.display_path(),
            app.editor.word_count(),
            alignment_note,
            if app.editor.is_modified() {
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
                let msg_width = (app.status_message.chars().count() as u16 + 1).min(area.width);
                let msg_area = Rect {
                    x: area.width.saturating_sub(msg_width),
                    y: area.height.saturating_sub(1),
                    width: msg_width,
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
    let alignment = app.alignment();
    let text_width = area.width as usize;
    let wrap_width = if app.config.wrap { text_width } else { 0 };
    app.editor.set_viewport(wrap_width, area.height as usize);
    let mut caret_column = 0usize;

    let visible_lines: Vec<Line> = app
        .editor
        .visible_rows(area.height as usize, wrap_width)
        .into_iter()
        .map(|rendered| {
            let mut text = app
                .editor
                .segment_text(rendered.row, rendered.start, rendered.end);
            let mut caret = rendered.caret;
            let justified = alignment == TextAlign::Justify
                && should_justify(&text, text_width, rendered.continues);
            if justified {
                let (stretched, mapped) = justify_line(&text, text_width, caret);
                text = stretched;
                caret = mapped;
            }

            let Some(offset) = caret else {
                return Line::from(Span::styled(text, Style::default().fg(fg)));
            };
            let before: String = text.chars().take(offset).collect();
            let at: String = text.chars().skip(offset).take(1).collect();
            let after: String = text.chars().skip(offset + 1).collect();
            caret_column = UnicodeWidthStr::width(before.as_str());
            let cursor_char = if at.is_empty() { " ".to_string() } else { at };
            let mut spans = vec![
                Span::styled(before, Style::default().fg(fg)),
                Span::styled(cursor_char, Style::default().fg(fg).bg(Color::DarkGray)),
            ];
            if after.is_empty() && !justified && !app.editor.ghost_suggestion.is_empty() {
                spans.push(Span::styled(
                    &app.editor.ghost_suggestion,
                    Style::default().fg(Color::DarkGray),
                ));
            }
            spans.push(Span::styled(after, Style::default().fg(fg)));
            Line::from(spans)
        })
        .collect();

    let text = Text::from(visible_lines);
    let h_offset = horizontal_offset(caret_column, text_width, wrap_width > 0);
    let para = Paragraph::new(text)
        .alignment(ratatui_alignment(alignment))
        .scroll((0, h_offset));
    f.render_widget(para, area);
}

/// Columns to slide the writing surface left so a caret past the right margin
/// stays on screen. Wrapped text never scrolls sideways.
fn horizontal_offset(caret_column: usize, text_width: usize, wrapping: bool) -> u16 {
    if wrapping || text_width == 0 || caret_column < text_width {
        0
    } else {
        u16::try_from(caret_column + 1 - text_width).unwrap_or(u16::MAX)
    }
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
        ("ctrl+l", "alignment"),
        ("ctrl+z / ctrl+y", "undo / redo"),
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

/// Days of writing history the goals screen lists.
const WEEK_LENGTH: usize = 7;
/// Spaces between a row's label column and its value column.
const LABEL_GAP: usize = 3;

fn render_goals(f: &mut Frame, app: &App, area: Rect) {
    let fg = app.get_theme_fg();
    let dim = app.get_dim_color();

    let v_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Fill(1),   // top padding
            Constraint::Length(1), // title
            Constraint::Length(1), // separator
            Constraint::Fill(2),   // stats
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

    // Build label/value pairs, then pad every row to one width so centred rows
    // line their columns up instead of drifting apart.
    let mut rows: Vec<(String, String)> = vec![
        ("word count".into(), app.editor.word_count().to_string()),
        (
            "character count".into(),
            app.editor.char_count().to_string(),
        ),
        (
            "reading time".into(),
            format!("{} min", app.editor.reading_time_minutes()),
        ),
        (
            "session time".into(),
            format!("{} min", app.session_minutes()),
        ),
        (String::new(), String::new()),
        ("recent days".into(), String::new()),
    ];

    for day in app.recent_days(WEEK_LENGTH) {
        let mut value = format!("{} words", day.words);
        if day.minutes > 0 {
            value.push_str(&format!(" · {} min", day.minutes));
        }
        // Documents only mean something for the day being written now.
        if day.label == "today" && day.documents > 0 {
            let noun = if day.documents == 1 {
                "document"
            } else {
                "documents"
            };
            value.push_str(&format!(" · {} {noun}", day.documents));
        }
        rows.push((day.label, value));
    }

    // Lay the rows out as one block — a label column, a value column, and
    // trailing padding — so every row is the same width and centring keeps the
    // columns aligned instead of drifting apart.
    let label_column = rows
        .iter()
        .map(|(label, _)| UnicodeWidthStr::width(label.as_str()) + LABEL_GAP)
        .max()
        .unwrap_or(0);
    let row_width = |label: &str, value: &str| {
        if value.is_empty() {
            UnicodeWidthStr::width(label)
        } else {
            label_column + UnicodeWidthStr::width(value)
        }
    };
    let total = rows
        .iter()
        .map(|(label, value)| row_width(label, value))
        .max()
        .unwrap_or(0);

    let stats_lines: Vec<Line> = rows
        .iter()
        .take(v_chunks[3].height as usize)
        .map(|(label, value)| {
            // Headings and spacers just hold the block width.
            if value.is_empty() {
                let pad = total - UnicodeWidthStr::width(label.as_str());
                return Line::from(Span::styled(
                    format!("{label}{}", " ".repeat(pad)),
                    Style::default().fg(dim),
                ));
            }
            let lead = " ".repeat(label_column - UnicodeWidthStr::width(label.as_str()));
            let tail = " ".repeat(total - row_width(label, value));
            Line::from(vec![
                Span::styled(label.clone(), Style::default().fg(dim)),
                Span::styled(lead, Style::default().fg(dim)),
                Span::styled(value.clone(), Style::default().fg(fg)),
                Span::styled(tail, Style::default().fg(fg)),
            ])
        })
        .collect();

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

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::{backend::TestBackend, Terminal};

    /// Render the editor and return one string per terminal row.
    fn draw(app: &App, width: u16, height: u16) -> Vec<String> {
        let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
        terminal.draw(|f| render(f, app)).unwrap();
        let buffer = terminal.backend().buffer();
        (0..height)
            .map(|y| {
                (0..width)
                    .map(|x| buffer[(x, y)].symbol())
                    .collect::<String>()
            })
            .collect()
    }

    fn editor_app(content: &str, alignment: &str, line_numbers: &str) -> App {
        let mut app = App::new();
        app.mode = Mode::Editor;
        app.config.alignment = alignment.to_string();
        app.config.line_numbers = line_numbers.to_string();
        app.editor.set_content(content);
        app
    }

    #[test]
    fn center_alignment_centers_each_line() {
        let app = editor_app("hi", "center", "off");
        let rows = draw(&app, 10, 3);
        assert_eq!(rows[0], "    hi    ");
    }

    #[test]
    fn right_alignment_flushes_the_margin() {
        let app = editor_app("hi", "right", "off");
        let rows = draw(&app, 10, 3);
        assert_eq!(rows[0], "        hi");
    }

    #[test]
    fn left_alignment_is_unchanged() {
        let app = editor_app("hi", "left", "off");
        let rows = draw(&app, 10, 3);
        assert_eq!(rows[0], "hi        ");
    }

    #[test]
    fn justification_fills_the_width_but_not_the_paragraph_end() {
        let app = editor_app("hello world\nnext line", "justified", "off");
        let rows = draw(&app, 12, 4);
        // The paragraph line fills both margins…
        assert_eq!(rows[0], "hello  world");
        // …while its final line stays ragged ("next    line" would be stretched).
        assert_eq!(rows[1], "next line   ");
        assert_ne!(rows[1], "next    line");
    }

    #[test]
    fn line_numbers_stay_left_when_text_is_centered() {
        let app = editor_app("hi", "center", "absolute");
        let rows = draw(&app, 16, 3);
        let gutter = &rows[0][..6];
        assert_eq!(gutter.trim(), "1");
        assert!(gutter.starts_with(' '));
        assert_eq!(&rows[0][6..], "    hi    ");
    }

    #[test]
    fn justified_caret_stays_on_its_character() {
        // `cd` is the paragraph's last line only if another line follows it…
        let mut app = editor_app("ab cd\nxy", "justified", "off");
        // …so line 1 stretches to "ab    cd" and the caret (before `c`) follows.
        app.editor.cursor_col = 3;
        let mut terminal = Terminal::new(TestBackend::new(8, 2)).unwrap();
        terminal.draw(|f| render(f, &app)).unwrap();
        let buffer = terminal.backend().buffer();
        let caret = (0..8u16)
            .find(|&x| buffer[(x, 0)].bg == Color::DarkGray)
            .expect("caret cell");
        assert_eq!(caret, 6);
        assert_eq!(buffer[(caret, 0)].symbol(), "c");
    }

    #[test]
    fn long_lines_wrap_instead_of_truncating() {
        let app = editor_app("abcdefghijklmnopqrstuvwxyz", "left", "off");
        let rows = draw(&app, 10, 4);
        assert_eq!(rows[0], "abcdefghij");
        assert_eq!(rows[1], "klmnopqrst");
        assert_eq!(rows[2], "uvwxyz    ");
    }

    #[test]
    fn wrapped_rows_justify_to_both_margins() {
        // One long logical paragraph: every wrapped row but the last stretches.
        let app = editor_app("aaa bbb ccc ddd", "justified", "off");
        let rows = draw(&app, 8, 4);
        assert_eq!(rows[0], "aaa  bbb");
        assert_eq!(rows[1], "ccc ddd ");
    }

    #[test]
    fn wrap_off_scrolls_sideways_to_keep_the_caret_visible() {
        let mut app = editor_app("abcdefghijklmnopqrst", "left", "off");
        app.config.wrap = false;
        app.editor.cursor_col = 15;
        let mut terminal = Terminal::new(TestBackend::new(10, 3)).unwrap();
        terminal.draw(|f| render(f, &app)).unwrap();
        let buffer = terminal.backend().buffer();
        let row: String = (0..10u16).map(|x| buffer[(x, 0)].symbol()).collect();
        assert_eq!(row, "ghijklmnop");
        // The caret block sits on `p`, the character it belongs to.
        assert_eq!(buffer[(9, 0)].bg, Color::DarkGray);
        assert_eq!(buffer[(9, 0)].symbol(), "p");
    }

    #[test]
    fn line_numbers_label_only_the_first_wrapped_row() {
        let app = editor_app("abcdefghijklmnopqrst", "left", "absolute");
        let rows = draw(&app, 16, 4);
        assert_eq!(rows[0][..6].trim(), "1");
        assert_eq!(rows[1][..6].trim(), "");
    }

    #[test]
    fn goals_screen_lists_the_recent_days() {
        let mut app = editor_app("one two three", "left", "off");
        app.mode = Mode::Goals;
        app.storage.today().seconds = 3 * 60;
        app.storage.today().words = 42;
        app.storage.track_document("/notes/a.txt");
        // An older day, to prove the list runs past today.
        app.storage.session_log.insert(
            0,
            crate::storage::SessionStats {
                date: "1999-01-01".to_string(),
                words: 7,
                seconds: 60,
                ..Default::default()
            },
        );

        let rows = draw(&app, 80, 24);
        let screen = rows.join("\n");
        assert!(screen.contains("word count"));
        assert!(screen.contains("session time"));
        assert!(screen.contains("recent days"));
        assert!(screen.contains("today"));
        assert!(screen.contains("42 words"));
        assert!(screen.contains("3 min"));
        assert!(screen.contains("1 document"));
        // The older day is labelled by its date, not by a weekday name.
        assert!(screen.contains("01-01"));
        assert!(screen.contains("7 words"));
    }

    #[test]
    fn focus_mode_honors_alignment() {
        let mut app = editor_app("hi", "right", "off");
        app.mode = Mode::Focus;
        let rows = draw(&app, 10, 3);
        assert_eq!(rows[0], "        hi");
    }
}
