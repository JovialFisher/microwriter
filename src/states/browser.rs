use crossterm::event::{KeyCode, KeyEvent};

pub enum BrowserAction {
    Select,
    GoBack,
    GoHome,
    StartSearch,
    FilterChanged,
    None,
}

pub struct BrowserState {
    pub items: Vec<String>,
    pub index: usize,
    pub path: String,
    pub search: String,
    pub searching: bool,
}

impl BrowserState {
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            index: 0,
            path: String::new(),
            search: String::new(),
            searching: false,
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> BrowserAction {
        if self.searching {
            return self.handle_search_key(key);
        }
        self.handle_browse_key(key)
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
            _ => BrowserAction::None,
        }
    }

    pub fn selected_item(&self) -> Option<&str> {
        self.items.get(self.index).map(|s| s.as_str())
    }
}
