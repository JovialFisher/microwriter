use crossterm::event::{KeyCode, KeyEvent};

/// Every palette command, in display order.
///
/// Kept in one place so construction, filtering, and dispatch cannot drift
/// apart — they previously each held their own copy of this list.
pub const COMMANDS: &[&str] = &[
    "new note",
    "open note",
    "toggle wrap",
    "toggle typewriter scroll",
    "toggle line numbers",
    "export html",
    "align left",
    "align center",
    "align right",
    "align justified",
    "focus mode",
    "goals",
    "settings",
    "help",
    "quit",
];

pub enum PaletteAction {
    Cancel,
    Confirm,
    Autocomplete,
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
            items: COMMANDS.iter().map(|s| s.to_string()).collect(),
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
            KeyCode::Tab => {
                if !self.items.is_empty() {
                    PaletteAction::Autocomplete
                } else {
                    PaletteAction::None
                }
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
        self.items = COMMANDS
            .iter()
            .filter(|command| self.query.is_empty() || command.to_lowercase().contains(&self.query))
            .map(|command| command.to_string())
            .collect();
        if self.index >= self.items.len() {
            self.index = self.items.len().saturating_sub(1);
        }
    }

    pub fn selected_command(&self) -> Option<&str> {
        self.items.get(self.index).map(|s| s.as_str())
    }
}
