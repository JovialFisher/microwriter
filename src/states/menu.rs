use crossterm::event::{KeyCode, KeyEvent};

pub enum MenuAction {
    Select,
    Shortcut(char),
    Quit,
    None,
}

pub struct MenuItem {
    pub label: String,
    pub key: char,
}

impl MenuItem {
    pub fn new(label: &str, key: char) -> Self {
        Self {
            label: label.to_string(),
            key,
        }
    }
}

pub struct MenuState {
    pub index: usize,
    pub items: Vec<MenuItem>,
}

impl MenuState {
    pub fn new() -> Self {
        Self {
            index: 0,
            items: vec![
                MenuItem::new("new note", 'n'),
                MenuItem::new("open note", 'o'),
                MenuItem::new("recent notes", 'r'),
                MenuItem::new("search", 'f'),
                MenuItem::new("settings", 's'),
                MenuItem::new("help", 'h'),
                MenuItem::new("exit", 'q'),
            ],
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> MenuAction {
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                if self.index > 0 {
                    self.index -= 1;
                }
                MenuAction::None
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if self.index < self.items.len() - 1 {
                    self.index += 1;
                }
                MenuAction::None
            }
            KeyCode::Enter => MenuAction::Select,
            KeyCode::Esc => MenuAction::Quit,
            KeyCode::Char(c) => MenuAction::Shortcut(c),
            _ => MenuAction::None,
        }
    }

    pub fn selected_label(&self) -> &str {
        &self.items[self.index].label
    }

    pub fn select_by_shortcut(&mut self, c: char) -> bool {
        for (i, item) in self.items.iter().enumerate() {
            if item.key == c {
                self.index = i;
                return true;
            }
        }
        false
    }
}
