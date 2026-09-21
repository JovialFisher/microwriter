use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::time::Instant;

use crate::config::Config;
use crate::editor::Editor;
use crate::states::search::fuzzy_match;
use crate::states::{
    BrowserAction, BrowserState, MenuAction, MenuState, PaletteAction, PaletteState, SearchAction,
    SearchState, SettingsAction, SettingsState,
};
use crate::storage::Storage;

#[derive(Debug, Clone, PartialEq)]
pub enum Mode {
    Startup,
    Editor,
    FileBrowser,
    Search,
    RecentNotes,
    Settings,
    CommandPalette,
    Help,
    Focus,
    RecoveryPrompt,
    Goals,
}

pub struct App {
    pub mode: Mode,
    pub should_quit: bool,
    pub config: Config,
    pub storage: Storage,
    pub editor: Editor,

    // Mode-specific state
    pub menu: MenuState,
    pub browser: BrowserState,
    pub search: SearchState,
    pub palette: PaletteState,
    pub settings: SettingsState,

    // Status
    pub status_message: String,
    pub status_timer: Option<Instant>,

    // Recovery
    pub has_recovery: bool,
    pub recovery_path: String,
    pub recovery_index: usize,

    // Recent notes index
    pub recent_index: usize,

    // Autosave
    pub last_autosave: Instant,
    pub last_save: Instant,
}

impl App {
    pub fn new() -> Self {
        Self {
            mode: Mode::Startup,
            should_quit: false,
            config: Config::default(),
            storage: Storage::default(),
            editor: Editor::new(),
            menu: MenuState::new(),
            browser: BrowserState::new(),
            search: SearchState::new(),
            palette: PaletteState::new(),
            settings: SettingsState::new(),
            status_message: String::new(),
            status_timer: None,
            has_recovery: false,
            recovery_path: String::new(),
            recovery_index: 0,
            recent_index: 0,
            last_autosave: Instant::now(),
            last_save: Instant::now(),
        }
    }

    pub fn init(&mut self) -> std::io::Result<()> {
        if let Ok(cfg) = Config::load() {
            self.config = cfg;
        }

        if let Ok(s) = Storage::load() {
            self.storage = s;
        }

        if let Some(path) = &self.storage.last_session_file {
            if std::path::Path::new(path).exists() {
                self.has_recovery = true;
                self.recovery_path = path.clone();
                self.mode = Mode::RecoveryPrompt;
            }
        }

        self.apply_theme();

        if self.config.default_folder.is_empty() {
            if let Some(docs) = dirs::document_dir() {
                self.config.default_folder = docs.to_string_lossy().to_string();
            }
        }

        self.browser.path = self.config.default_folder.clone();

        Ok(())
    }

    pub fn apply_theme(&mut self) {}

    pub fn handle_key(&mut self, key: KeyEvent) -> bool {
        // Global shortcuts
        if key.modifiers == KeyModifiers::CONTROL {
            match key.code {
                KeyCode::Char('c') => {
                    if self.mode == Mode::Editor || self.mode == Mode::Focus {
                        self.save_current_file();
                    }
                    self.should_quit = true;
                    return false;
                }
                KeyCode::Char('q') => {
                    if self.mode == Mode::Editor || self.mode == Mode::Focus {
                        self.save_current_file();
                    }
                    self.should_quit = true;
                    return false;
                }
                KeyCode::Char('s') => {
                    if self.mode == Mode::Editor || self.mode == Mode::Focus {
                        self.save_current_file();
                    }
                    return true;
                }
                KeyCode::Char('p') => {
                    self.open_command_palette();
                    return true;
                }
                KeyCode::Char('n') => {
                    self.new_note();
                    return true;
                }
                KeyCode::Char('o') => {
                    self.open_file_browser();
                    return true;
                }
                KeyCode::Char('f') => {
                    self.open_search();
                    return true;
                }
                _ => {}
            }
        }

        match self.mode {
            Mode::Startup => self.dispatch_menu(key),
            Mode::Editor => self.dispatch_editor(key),
            Mode::Focus => self.dispatch_focus(key),
            Mode::FileBrowser => self.dispatch_browser(key),
            Mode::Search => self.dispatch_search(key),
            Mode::RecentNotes => self.dispatch_recent(key),
            Mode::Settings => self.dispatch_settings(key),
            Mode::CommandPalette => self.dispatch_palette(key),
            Mode::Help => self.dispatch_help(key),
            Mode::RecoveryPrompt => self.dispatch_recovery(key),
            Mode::Goals => self.dispatch_goals(key),
        }
    }

    // ─── Dispatch helpers ─────────────────────────────────────

    fn dispatch_menu(&mut self, key: KeyEvent) -> bool {
        match self.menu.handle_key(key) {
            MenuAction::Quit => {
                self.should_quit = true;
                return false;
            }
            MenuAction::Select => self.select_menu_item(),
            MenuAction::Shortcut(c) if self.menu.select_by_shortcut(c) => {
                self.select_menu_item();
            }
            _ => {}
        }
        true
    }

    fn dispatch_editor(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Esc => {
                self.editor.ghost_suggestion.clear();
                self.editor.autocomplete_matches.clear();
                self.editor.autocomplete_prefix.clear();
                self.mode = Mode::Startup;
                self.menu.index = 0;
            }
            KeyCode::Char(' ') if key.modifiers == KeyModifiers::CONTROL => {
                if self.editor.try_autocomplete() {
                    let total = self.editor.autocomplete_matches.len();
                    let current = self.editor.autocomplete_index + 1;
                    self.show_status(&format!("match {}/{}", current, total));
                } else {
                    self.show_status("no match");
                }
                self.editor.refresh_ghost();
            }
            KeyCode::Char(c) => {
                if key.modifiers.is_empty() || key.modifiers == KeyModifiers::SHIFT {
                    self.editor.autocomplete_matches.clear();
                    self.editor.autocomplete_prefix.clear();
                    self.editor.insert_char(c);
                    self.editor.refresh_ghost();
                    self.clear_status();
                }
            }
            KeyCode::Enter => {
                if self.editor.accept_ghost() {
                    self.clear_status();
                } else {
                    self.editor.insert_newline();
                    self.editor.refresh_ghost();
                    self.clear_status();
                }
            }
            KeyCode::Backspace => {
                self.editor.backspace();
                self.editor.refresh_ghost();
                self.clear_status();
            }
            KeyCode::Delete => {
                self.editor.delete_forward();
                self.editor.refresh_ghost();
                self.clear_status();
            }
            KeyCode::Tab => {
                // Accept ghost suggestion if present, else try autocomplete, else indent
                if self.editor.accept_ghost() {
                    self.clear_status();
                } else if self.editor.try_autocomplete() {
                    let total = self.editor.autocomplete_matches.len();
                    let current = self.editor.autocomplete_index + 1;
                    self.show_status(&format!("match {}/{}", current, total));
                    self.editor.refresh_ghost();
                } else if self.config.use_tabs {
                    self.editor.insert_char('\t');
                    self.clear_status();
                } else {
                    for _ in 0..self.config.tab_spaces {
                        self.editor.insert_char(' ');
                    }
                    self.clear_status();
                }
            }
            KeyCode::Left => {
                if key.modifiers == KeyModifiers::CONTROL {
                    self.editor.move_word_left();
                } else {
                    self.editor.move_left();
                }
                self.editor.refresh_ghost();
            }
            KeyCode::Right => {
                if key.modifiers == KeyModifiers::CONTROL {
                    self.editor.move_word_right();
                    self.editor.refresh_ghost();
                } else if self.editor.accept_ghost() {
                    // Ghost accepted — no further action
                } else {
                    self.editor.move_right();
                    self.editor.refresh_ghost();
                }
            }
            KeyCode::Up => {
                if key.modifiers == KeyModifiers::CONTROL {
                    self.editor.scroll_up(3);
                } else {
                    self.editor.move_up();
                }
                self.editor.refresh_ghost();
            }
            KeyCode::Down => {
                if key.modifiers == KeyModifiers::CONTROL {
                    self.editor.scroll_down(3);
                } else {
                    self.editor.move_down();
                }
                self.editor.refresh_ghost();
            }
            KeyCode::Home => {
                if key.modifiers == KeyModifiers::CONTROL {
                    self.editor.move_top();
                } else {
                    self.editor.move_line_start();
                }
                self.editor.refresh_ghost();
            }
            KeyCode::End => {
                if key.modifiers == KeyModifiers::CONTROL {
                    self.editor.move_bottom();
                } else {
                    self.editor.move_line_end();
                }
                self.editor.refresh_ghost();
            }
            KeyCode::PageUp => {
                self.editor.page_up();
                self.editor.refresh_ghost();
            }
            KeyCode::PageDown => {
                self.editor.page_down();
                self.editor.refresh_ghost();
            }
            _ => {}
        }
        true
    }

    fn dispatch_focus(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Esc | KeyCode::F(1) => {
                self.mode = Mode::Editor;
            }
            _ => {
                return self.dispatch_editor(key);
            }
        }
        true
    }

    fn dispatch_browser(&mut self, key: KeyEvent) -> bool {
        match self.browser.handle_key(key) {
            BrowserAction::GoBack => {
                if let Some(parent) = std::path::Path::new(&self.browser.path).parent() {
                    self.browser.path = parent.to_string_lossy().to_string();
                    self.refresh_browser();
                } else {
                    self.mode = Mode::Startup;
                }
            }
            BrowserAction::GoHome => {
                if let Some(home) = dirs::home_dir() {
                    self.browser.path = home.to_string_lossy().to_string();
                    self.refresh_browser();
                }
            }
            BrowserAction::Select => self.open_selected_browser_item(),
            BrowserAction::StartSearch => {
                self.browser.searching = true;
                self.browser.search.clear();
            }
            BrowserAction::FilterChanged => {
                self.filter_browser_items();
            }
            _ => {}
        }
        true
    }

    fn dispatch_search(&mut self, key: KeyEvent) -> bool {
        match self.search.handle_key(key) {
            SearchAction::Cancel => {
                self.mode = Mode::Startup;
            }
            SearchAction::Confirm => {
                if let Some(path) = self.search.selected_path() {
                    self.open_file(&path);
                }
            }
            SearchAction::UpdateQuery => {
                self.update_search_results();
            }
            SearchAction::Autocomplete => {
                if let Some(first) = self.search.results.first() {
                    let display = first.split('|').next().unwrap_or(first);
                    self.search.query = display.to_string();
                    self.search.index = 0;
                    self.update_search_results();
                }
            }
            _ => {}
        }
        true
    }

    fn dispatch_recent(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Esc => {
                self.mode = Mode::Startup;
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if self.recent_index > 0 {
                    self.recent_index -= 1;
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                let max = self.storage.recent_files.len().saturating_sub(1);
                if self.recent_index < max {
                    self.recent_index += 1;
                }
            }
            KeyCode::Enter if !self.storage.recent_files.is_empty() => {
                let path = self.storage.recent_files[self.recent_index].path.clone();
                self.open_file(&path);
            }
            _ => {}
        }
        true
    }

    fn dispatch_settings(&mut self, key: KeyEvent) -> bool {
        let current = self.get_current_setting_value();
        match self.settings.handle_key(key, &current) {
            SettingsAction::SaveAndExit => {
                self.mode = Mode::Startup;
                self.config.save().ok();
                self.apply_theme();
            }
            SettingsAction::ConfirmEdit => {
                self.apply_setting();
            }
            _ => {}
        }
        true
    }

    fn dispatch_palette(&mut self, key: KeyEvent) -> bool {
        match self.palette.handle_key(key) {
            PaletteAction::Cancel => {
                self.mode = Mode::Startup;
            }
            PaletteAction::Confirm => {
                if let Some(cmd) = self.palette.selected_command().map(|s| s.to_string()) {
                    self.execute_palette_command(&cmd);
                }
            }
            PaletteAction::Autocomplete => {
                if let Some(first) = self.palette.items.first() {
                    self.palette.query = first.clone();
                    self.palette.filter();
                }
            }
            _ => {}
        }
        true
    }

    fn dispatch_help(&mut self, _key: KeyEvent) -> bool {
        self.mode = Mode::Startup;
        true
    }

    fn dispatch_recovery(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Up | KeyCode::Char('k') | KeyCode::Down | KeyCode::Char('j') => {
                self.recovery_index = if self.recovery_index == 0 { 1 } else { 0 };
            }
            KeyCode::Enter => {
                if self.recovery_index == 0 {
                    self.open_file(&self.recovery_path.clone());
                } else {
                    self.storage.last_session_file = None;
                    self.has_recovery = false;
                }
                self.mode = Mode::Startup;
            }
            KeyCode::Esc => {
                self.mode = Mode::Startup;
            }
            _ => {}
        }
        true
    }

    fn dispatch_goals(&mut self, key: KeyEvent) -> bool {
        if key.code == KeyCode::Esc {
            self.mode = Mode::Startup;
        }
        true
    }

    // ─── Actions ───────────────────────────────────────────────

    fn select_menu_item(&mut self) {
        match self.menu.selected_label() {
            "new note" => self.new_note(),
            "open note" => self.open_file_browser(),
            "recent notes" => self.open_recent_notes(),
            "search" => self.open_search(),
            "settings" => self.open_settings(),
            "help" => self.open_help(),
            "exit" => self.should_quit = true,
            _ => {}
        }
    }

    pub fn new_note(&mut self) {
        let filename = if self.config.timestamp_filenames {
            chrono::Local::now()
                .format("%Y-%m-%d-%H-%M.txt")
                .to_string()
        } else {
            "untitled.txt".to_string()
        };
        let path = format!("{}/{}", self.config.default_folder, filename);
        self.editor = Editor::new();
        self.editor.file_path = Some(path.clone());
        self.editor.modified = true;
        self.storage.last_session_file = Some(path);
        self.mode = Mode::Editor;
    }

    pub fn open_file_browser(&mut self) {
        self.browser.path = self.config.default_folder.clone();
        self.refresh_browser();
        self.browser.index = 0;
        self.mode = Mode::FileBrowser;
    }

    fn refresh_browser(&mut self) {
        self.browser.items.clear();
        if let Ok(entries) = std::fs::read_dir(&self.browser.path) {
            let mut dirs = Vec::new();
            let mut files = Vec::new();
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                if name.starts_with('.') {
                    continue;
                }
                let path = entry.path();
                if path.is_dir() {
                    dirs.push(format!("{}/", name));
                } else if let Some(ext) = path.extension() {
                    let ext = ext.to_string_lossy().to_lowercase();
                    if matches!(ext.as_str(), "txt" | "md" | "rst" | "log") {
                        files.push(name);
                    }
                }
            }
            dirs.sort();
            files.sort();
            self.browser.items.push("..".to_string());
            self.browser.items.extend(dirs);
            self.browser.items.extend(files);
        }
    }

    fn filter_browser_items(&mut self) {
        self.refresh_browser();
        let query = self.browser.search.to_lowercase();
        self.browser
            .items
            .retain(|item| item.to_lowercase().contains(&query));
        if self.browser.index >= self.browser.items.len() {
            self.browser.index = self.browser.items.len().saturating_sub(1);
        }
    }

    fn open_selected_browser_item(&mut self) {
        if self.browser.items.is_empty() {
            return;
        }
        match self.browser.selected_item() {
            Some("..") => {
                if let Some(parent) = std::path::Path::new(&self.browser.path).parent() {
                    self.browser.path = parent.to_string_lossy().to_string();
                    self.refresh_browser();
                    self.browser.index = 0;
                }
            }
            Some(item) if item.ends_with('/') => {
                let dir_name = item.trim_end_matches('/');
                self.browser.path = format!("{}/{}", self.browser.path, dir_name);
                self.refresh_browser();
                self.browser.index = 0;
            }
            Some(item) => {
                let path = format!("{}/{}", self.browser.path, item);
                self.open_file(&path);
            }
            None => {}
        }
    }

    pub fn open_search(&mut self) {
        self.search.query.clear();
        self.search.index = 0;
        self.update_search_results();
        self.mode = Mode::Search;
    }

    fn update_search_results(&mut self) {
        self.search.results.clear();
        let query = self.search.query.to_lowercase();
        let mut candidates: Vec<(String, String, i32)> = Vec::new();

        if let Ok(entries) = std::fs::read_dir(&self.config.default_folder) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    if let Some(ext) = path.extension() {
                        let ext = ext.to_string_lossy().to_lowercase();
                        if matches!(ext.as_str(), "txt" | "md" | "rst" | "log") {
                            let full_path = path.to_string_lossy().to_string();
                            let display = path
                                .file_name()
                                .map(|n| n.to_string_lossy().to_string())
                                .unwrap_or_default();
                            let boost = if self
                                .storage
                                .recent_files
                                .iter()
                                .any(|r| r.path == full_path)
                            {
                                10
                            } else {
                                0
                            };
                            if query.is_empty() {
                                candidates.push((display, full_path, boost));
                            } else {
                                let score = fuzzy_match(&query, &display.to_lowercase());
                                if score >= 0 {
                                    candidates.push((display, full_path, score + boost));
                                }
                            }
                        }
                    }
                }
            }
        }

        if query.is_empty() {
            candidates.sort_by_key(|(_, _, s)| -(*s));
        } else {
            candidates.sort_by(|(_, _, a), (_, _, b)| b.cmp(a));
        }

        self.search.results = candidates
            .into_iter()
            .map(|(d, p, _)| format!("{}|{}", d, p))
            .collect();

        for recent in &self.storage.recent_files {
            let entry = format!("{}|{}", recent.display_name, recent.path);
            if (query.is_empty() || recent.display_name.to_lowercase().contains(&query))
                && !self.search.results.contains(&entry)
            {
                self.search.results.push(entry);
            }
        }

        self.search.results.truncate(20);
    }

    pub fn open_recent_notes(&mut self) {
        self.recent_index = 0;
        self.mode = Mode::RecentNotes;
    }

    pub fn open_settings(&mut self) {
        self.settings.category_index = 0;
        self.settings.option_index = 0;
        self.settings.editing = false;
        self.mode = Mode::Settings;
    }

    pub fn open_help(&mut self) {
        self.mode = Mode::Help;
    }

    pub fn open_command_palette(&mut self) {
        self.palette.query.clear();
        self.palette.index = 0;
        self.palette.items = vec![
            "new note".into(),
            "open note".into(),
            "toggle wrap".into(),
            "toggle line numbers".into(),
            "focus mode".into(),
            "goals".into(),
            "settings".into(),
            "help".into(),
            "quit".into(),
        ];
        self.mode = Mode::CommandPalette;
    }

    fn execute_palette_command(&mut self, cmd: &str) {
        match cmd {
            "new note" => {
                self.mode = Mode::Startup;
                self.new_note();
            }
            "open note" => {
                self.mode = Mode::Startup;
                self.open_file_browser();
            }
            "toggle wrap" => {
                self.config.wrap = !self.config.wrap;
                self.mode = Mode::Editor;
            }
            "toggle line numbers" => {
                self.config.line_numbers = match self.config.line_numbers.as_str() {
                    "off" => "absolute".to_string(),
                    "absolute" => "relative".to_string(),
                    "relative" => "off".to_string(),
                    _ => "off".to_string(),
                };
                self.mode = Mode::Editor;
            }
            "focus mode" => {
                self.mode = Mode::Focus;
            }
            "goals" => {
                self.mode = Mode::Goals;
            }
            "settings" => {
                self.mode = Mode::Startup;
                self.open_settings();
            }
            "help" => {
                self.mode = Mode::Startup;
                self.open_help();
            }
            "quit" => {
                self.save_current_file();
                self.should_quit = true;
            }
            _ => {
                self.mode = Mode::Editor;
            }
        }
    }

    pub fn open_file(&mut self, path: &str) {
        match std::fs::read_to_string(path) {
            Ok(content) => {
                self.editor = Editor::new();
                self.editor.file_path = Some(path.to_string());
                self.editor.set_content(&content);
                self.editor.modified = false;
                self.load_cross_file_words(path);
                self.storage.track_file(path);
                self.storage.last_session_file = Some(path.to_string());
                self.mode = Mode::Editor;
            }
            Err(_) => {
                self.editor = Editor::new();
                self.editor.file_path = Some(path.to_string());
                self.editor.modified = true;
                self.storage.last_session_file = Some(path.to_string());
                self.mode = Mode::Editor;
            }
        }
    }

    fn load_cross_file_words(&mut self, current_path: &str) {
        use std::collections::BTreeSet;
        let mut words: BTreeSet<String> = BTreeSet::new();
        for entry in self.storage.recent_files.iter().take(10) {
            if entry.path == current_path {
                continue;
            }
            if let Ok(content) = std::fs::read_to_string(&entry.path) {
                for word in content.split_whitespace() {
                    let cleaned = crate::editor::clean_word(word);
                    if cleaned.len() >= 2 {
                        words.insert(cleaned.to_string());
                    }
                }
            }
        }
        self.editor.cross_file_words = words.into_iter().collect();
    }

    pub fn save_current_file(&mut self) {
        if let Some(path) = &self.editor.file_path.clone() {
            let content = self.editor.get_content();
            let tmp_path = format!("{}.tmp", path);
            if std::fs::write(&tmp_path, &content).is_ok() {
                if std::fs::rename(&tmp_path, path).is_ok() {
                    self.editor.modified = false;
                    self.last_save = Instant::now();
                    self.show_status("saved");
                    self.storage.track_file(path);
                    self.storage.last_session_file = Some(path.clone());
                } else {
                    if std::fs::write(path, &content).is_ok() {
                        self.editor.modified = false;
                        self.last_save = Instant::now();
                        self.show_status("saved");
                        self.storage.track_file(path);
                        self.storage.last_session_file = Some(path.clone());
                    }
                    let _ = std::fs::remove_file(&tmp_path);
                }
            }
        }
    }

    pub fn autosave_tick(&mut self) {
        if self.config.autosave == "disabled" {
            return;
        }
        if self.mode != Mode::Editor && self.mode != Mode::Focus {
            return;
        }
        if !self.editor.modified {
            return;
        }

        let interval = match self.config.autosave.as_str() {
            "15 sec" => 15,
            "30 sec" => 30,
            "1 min" => 60,
            "5 min" => 300,
            _ => return,
        };

        if self.last_autosave.elapsed().as_secs() >= interval {
            self.save_current_file();
            self.last_autosave = Instant::now();
        }
    }

    fn show_status(&mut self, msg: &str) {
        self.status_message = msg.to_string();
        self.status_timer = Some(Instant::now());
    }

    fn clear_status(&mut self) {
        self.status_message.clear();
        self.status_timer = None;
    }

    pub fn save_state(&self) -> std::io::Result<()> {
        self.config.save().map_err(std::io::Error::other)?;
        self.storage.save()?;
        Ok(())
    }

    pub fn get_current_setting_value(&self) -> String {
        self.get_setting_value_for_category(self.settings.category_index)
    }

    pub fn get_setting_value_for_category(&self, category_index: usize) -> String {
        if category_index >= self.settings.categories.len() {
            return String::new();
        }
        match self.settings.categories[category_index].as_str() {
            "appearance" => self.config.theme.clone(),
            "cursor" => self.config.cursor_style.clone(),
            "line numbers" => self.config.line_numbers.clone(),
            "word wrap" => {
                if self.config.wrap {
                    "on".into()
                } else {
                    "off".into()
                }
            }
            "autosave" => self.config.autosave.clone(),
            "timestamp filenames" => {
                if self.config.timestamp_filenames {
                    "on".into()
                } else {
                    "off".into()
                }
            }
            "tabs/spaces" => {
                if self.config.use_tabs {
                    "tabs".into()
                } else {
                    "spaces".into()
                }
            }
            _ => String::new(),
        }
    }

    fn apply_setting(&mut self) {
        let value = self.settings.selected_value().to_string();
        match self.settings.selected_category() {
            "appearance" => {
                self.config.theme = value;
                self.apply_theme();
            }
            "cursor" => self.config.cursor_style = value,
            "line numbers" => self.config.line_numbers = value,
            "word wrap" => self.config.wrap = value == "on",
            "autosave" => self.config.autosave = value,
            "default folder" => {}
            "timestamp filenames" => self.config.timestamp_filenames = value == "on",
            "tabs/spaces" => {
                self.config.use_tabs = value == "tabs";
            }
            _ => {}
        }
    }

    pub fn status_visible(&self) -> bool {
        self.config.show_status || self.mode == Mode::Goals
    }

    pub fn display_path(&self) -> String {
        if let Some(path) = &self.editor.file_path {
            std::path::Path::new(path)
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| path.clone())
        } else {
            "untitled".to_string()
        }
    }

    pub fn get_theme_bg(&self) -> ratatui::style::Color {
        match self.config.theme.as_str() {
            "light" | "paper" => ratatui::style::Color::Rgb(245, 245, 240),
            "solarized dark" => ratatui::style::Color::Rgb(0, 43, 54),
            "nord" => ratatui::style::Color::Rgb(46, 52, 64),
            "amber terminal" => ratatui::style::Color::Rgb(30, 20, 0),
            "green phosphor" => ratatui::style::Color::Rgb(0, 20, 0),
            _ => ratatui::style::Color::Black,
        }
    }

    pub fn get_theme_fg(&self) -> ratatui::style::Color {
        match self.config.theme.as_str() {
            "light" | "paper" => ratatui::style::Color::Rgb(40, 40, 40),
            "amber terminal" => ratatui::style::Color::Rgb(255, 176, 0),
            "green phosphor" => ratatui::style::Color::Rgb(0, 255, 65),
            _ => ratatui::style::Color::White,
        }
    }

    pub fn get_dim_color(&self) -> ratatui::style::Color {
        match self.config.theme.as_str() {
            "light" | "paper" => ratatui::style::Color::Rgb(160, 160, 155),
            "amber terminal" => ratatui::style::Color::Rgb(120, 90, 0),
            "green phosphor" => ratatui::style::Color::Rgb(0, 120, 30),
            _ => ratatui::style::Color::Rgb(80, 80, 80),
        }
    }

    pub fn get_accent_color(&self) -> ratatui::style::Color {
        match self.config.theme.as_str() {
            "light" | "paper" => ratatui::style::Color::Rgb(60, 60, 60),
            "amber terminal" => ratatui::style::Color::Rgb(200, 140, 0),
            "green phosphor" => ratatui::style::Color::Rgb(0, 200, 50),
            "nord" => ratatui::style::Color::Rgb(136, 192, 208),
            "solarized dark" => ratatui::style::Color::Rgb(38, 139, 210),
            _ => ratatui::style::Color::Rgb(120, 120, 120),
        }
    }
}
