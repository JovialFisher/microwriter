use crossterm::event::{KeyCode, KeyEvent};

use crate::drives::Drive;

pub enum BrowserAction {
    Select,
    GoBack,
    GoHome,
    StartSearch,
    FilterChanged,
    OpenDrives,
    CloseDrives,
    SelectDrive,
    None,
}

pub struct BrowserState {
    pub items: Vec<String>,
    pub index: usize,
    pub path: String,
    pub search: String,
    pub searching: bool,
    /// Volume roots offered by the `d` overlay (system drive included).
    pub drives: Vec<Drive>,
    pub drives_open: bool,
    pub drives_index: usize,
}

impl BrowserState {
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            index: 0,
            path: String::new(),
            search: String::new(),
            searching: false,
            drives: Vec::new(),
            drives_open: false,
            drives_index: 0,
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> BrowserAction {
        if self.drives_open {
            return self.handle_drives_key(key);
        }
        if self.searching {
            return self.handle_search_key(key);
        }
        self.handle_browse_key(key)
    }

    fn handle_drives_key(&mut self, key: KeyEvent) -> BrowserAction {
        match key.code {
            KeyCode::Esc | KeyCode::Char('q') => BrowserAction::CloseDrives,
            KeyCode::Up | KeyCode::Char('k') => {
                if self.drives_index > 0 {
                    self.drives_index -= 1;
                }
                BrowserAction::None
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if self.drives_index < self.drives.len().saturating_sub(1) {
                    self.drives_index += 1;
                }
                BrowserAction::None
            }
            KeyCode::Enter => BrowserAction::SelectDrive,
            _ => BrowserAction::None,
        }
    }

    fn handle_search_key(&mut self, key: KeyEvent) -> BrowserAction {
        match key.code {
            KeyCode::Esc => {
                self.searching = false;
                self.search.clear();
                BrowserAction::None
            }
            KeyCode::Enter => {
                self.searching = false;
                BrowserAction::None
            }
            KeyCode::Backspace => {
                self.search.pop();
                BrowserAction::FilterChanged
            }
            KeyCode::Tab => {
                if !self.items.is_empty() {
                    self.search = self.items[0].clone();
                    BrowserAction::FilterChanged
                } else {
                    BrowserAction::None
                }
            }
            KeyCode::Char(c) => {
                self.search.push(c);
                BrowserAction::FilterChanged
            }
            _ => BrowserAction::None,
        }
    }

    fn handle_browse_key(&mut self, key: KeyEvent) -> BrowserAction {
        match key.code {
            KeyCode::Esc | KeyCode::Backspace => BrowserAction::GoBack,
            KeyCode::Up | KeyCode::Char('k') => {
                if self.index > 0 {
                    self.index -= 1;
                }
                BrowserAction::None
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if self.index < self.items.len().saturating_sub(1) {
                    self.index += 1;
                }
                BrowserAction::None
            }
            KeyCode::Enter => BrowserAction::Select,
            KeyCode::Char('/') => BrowserAction::StartSearch,
            KeyCode::Char('h') => BrowserAction::GoHome,
            KeyCode::Char('d') => BrowserAction::OpenDrives,
            _ => BrowserAction::None,
        }
    }

    pub fn selected_item(&self) -> Option<&str> {
        self.items.get(self.index).map(|s| s.as_str())
    }
}
