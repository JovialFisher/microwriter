use crossterm::event::{KeyCode, KeyEvent};

pub enum PaletteAction {
    Cancel,
    Confirm,
    None,
}

pub struct PaletteState {
    pub query: String,
    pub items: Vec<String>,
    pub index: usize,
}

impl PaletteState {
    pub fn new() -> Self {
        Self {
            query: String::new(),
            items: vec![
                "new note".into(),
                "open note".into(),
                "toggle wrap".into(),
                "toggle line numbers".into(),
                "focus mode".into(),
                "goals".into(),
                "settings".into(),
                "help".into(),
                "quit".into(),
            ],
            index: 0,
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> PaletteAction {
        match key.code {
            KeyCode::Esc => {
                self.query.clear();
                PaletteAction::Cancel
            }
            KeyCode::Enter => PaletteAction::Confirm,
            KeyCode::Backspace => {
                self.query.pop();
                self.filter();
                PaletteAction::None
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if self.index > 0 {
                    self.index -= 1;
                }
                PaletteAction::None
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if self.index < self.items.len().saturating_sub(1) {
                    self.index += 1;
                }
                PaletteAction::None
            }
            KeyCode::Char(c) => {
                self.query.push(c.to_ascii_lowercase());
                self.filter();
                PaletteAction::None
            }
            _ => PaletteAction::None,
        }
    }

    pub fn filter(&mut self) {
        let all_items = vec![
            "new note",
            "open note",
            "toggle wrap",
            "toggle line numbers",
            "focus mode",
            "goals",
            "settings",
            "help",
            "quit",
        ];
        if self.query.is_empty() {
            self.items = all_items.iter().map(|s| s.to_string()).collect();
        } else {
            self.items = all_items
                .iter()
                .filter(|s| s.to_lowercase().contains(&self.query))
                .map(|s| s.to_string())
                .collect();
        }
        if self.index >= self.items.len() {
            self.index = self.items.len().saturating_sub(1);
        }
    }

    pub fn selected_command(&self) -> Option<&str> {
        self.items.get(self.index).map(|s| s.as_str())
    }
}
