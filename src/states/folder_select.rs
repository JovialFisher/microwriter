use crossterm::event::{KeyCode, KeyEvent};

use crate::drives::Drive;

pub enum FolderSelectAction {
    Select,      // enter on the highlighted entry
    Cancel,      // esc — back to the menu
    GoUp,        // backspace — parent directory
    GoHome,      // h — jump to $HOME
    StartFilter, // / — filter entries
    FilterChanged,
    OpenDrives,  // d — show volume roots
    CloseDrives,
    SelectDrive,
    None,
}

pub struct FolderSelectState {
    pub path: String,
    pub items: Vec<String>, // "create here", "..", then "name/" entries
    pub index: usize,
    pub filter: String,
    pub filtering: bool,
    /// Volume roots offered by the `d` overlay (system drive included).
    pub drives: Vec<Drive>,
    pub drives_open: bool,
    pub drives_index: usize,
}

impl FolderSelectState {
    pub fn new() -> Self {
        Self {
            path: String::new(),
            items: Vec::new(),
            index: 0,
            filter: String::new(),
            filtering: false,
            drives: Vec::new(),
            drives_open: false,
            drives_index: 0,
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> FolderSelectAction {
        if self.drives_open {
            return self.handle_drives_key(key);
        }
        if self.filtering {
            return self.handle_filter_key(key);
        }
        match key.code {
            KeyCode::Esc => FolderSelectAction::Cancel,
            KeyCode::Backspace => FolderSelectAction::GoUp,
            KeyCode::Up | KeyCode::Char('k') => {
                if self.index > 0 {
                    self.index -= 1;
                }
                FolderSelectAction::None
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if self.index < self.items.len().saturating_sub(1) {
                    self.index += 1;
                }
                FolderSelectAction::None
            }
            KeyCode::Enter => FolderSelectAction::Select,
            KeyCode::Char('/') => FolderSelectAction::StartFilter,
            KeyCode::Char('h') => FolderSelectAction::GoHome,
            KeyCode::Char('d') => FolderSelectAction::OpenDrives,
            _ => FolderSelectAction::None,
        }
    }

    fn handle_drives_key(&mut self, key: KeyEvent) -> FolderSelectAction {
        match key.code {
            KeyCode::Esc | KeyCode::Char('q') => FolderSelectAction::CloseDrives,
            KeyCode::Up | KeyCode::Char('k') => {
                if self.drives_index > 0 {
                    self.drives_index -= 1;
                }
                FolderSelectAction::None
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if self.drives_index < self.drives.len().saturating_sub(1) {
                    self.drives_index += 1;
                }
                FolderSelectAction::None
            }
            KeyCode::Enter => FolderSelectAction::SelectDrive,
            _ => FolderSelectAction::None,
        }
    }

    fn handle_filter_key(&mut self, key: KeyEvent) -> FolderSelectAction {
        match key.code {
            KeyCode::Esc => {
                self.filtering = false;
                self.filter.clear();
                FolderSelectAction::FilterChanged
            }
            KeyCode::Enter => {
                self.filtering = false;
                FolderSelectAction::None
            }
            KeyCode::Backspace => {
                self.filter.pop();
                FolderSelectAction::FilterChanged
            }
            KeyCode::Tab => {
                if let Some(item) = self.items.get(self.index) {
                    self.filter = item.trim_end_matches('/').to_string();
                }
                FolderSelectAction::FilterChanged
            }
            KeyCode::Char(c) => {
                self.filter.push(c);
                FolderSelectAction::FilterChanged
            }
            _ => FolderSelectAction::None,
        }
    }

    pub fn selected_item(&self) -> Option<&str> {
        self.items.get(self.index).map(|s| s.as_str())
    }
}
