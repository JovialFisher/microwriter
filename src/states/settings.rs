use crossterm::event::{KeyCode, KeyEvent};

pub enum SettingsAction {
    SaveAndExit,
    ConfirmEdit,
    None,
}

pub struct SettingsState {
    pub categories: Vec<String>,
    pub category_index: usize,
    pub options: Vec<Vec<String>>,
    pub option_index: usize,
    pub editing: bool,
}

impl SettingsState {
    pub fn new() -> Self {
        Self {
            categories: vec![
                "appearance".into(),
                "cursor".into(),
                "line numbers".into(),
                "word wrap".into(),
                "autosave".into(),
                "default folder".into(),
                "timestamp filenames".into(),
                "tabs/spaces".into(),
            ],
            category_index: 0,
            options: vec![
                vec!["dark".into(), "light".into(), "terminal default".into()],
                vec!["block".into(), "beam".into(), "underline".into()],
                vec!["off".into(), "relative".into(), "absolute".into()],
                vec!["on".into(), "off".into()],
                vec!["disabled".into(), "15 sec".into(), "30 sec".into(), "1 min".into(), "5 min".into()],
                vec![],
                vec!["on".into(), "off".into()],
                vec!["tabs".into(), "spaces".into()],
            ],
            option_index: 0,
            editing: false,
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent, current_value: &str) -> SettingsAction {
        match key.code {
            KeyCode::Esc => {
                if self.editing {
                    self.editing = false;
                    SettingsAction::None
                } else {
                    SettingsAction::SaveAndExit
                }
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if self.editing {
                    if self.option_index > 0 {
                        self.option_index -= 1;
                    }
                } else {
                    if self.category_index > 0 {
                        self.category_index -= 1;
                        self.option_index = 0;
                    }
                }
                SettingsAction::None
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if self.editing {
                    let options = &self.options[self.category_index];
                    if self.option_index < options.len().saturating_sub(1) {
                        self.option_index += 1;
                    }
                } else {
                    if self.category_index < self.categories.len().saturating_sub(1) {
                        self.category_index += 1;
                        self.option_index = 0;
                    }
                }
                SettingsAction::None
            }
            KeyCode::Enter | KeyCode::Right => {
                if !self.editing && !self.options[self.category_index].is_empty() {
                    if let Some(idx) = self.options[self.category_index]
                        .iter()
                        .position(|o| o == current_value)
                    {
                        self.option_index = idx;
                    }
                    self.editing = true;
                    SettingsAction::None
                } else if self.editing {
                    self.editing = false;
                    SettingsAction::ConfirmEdit
                } else {
                    SettingsAction::None
                }
            }
            KeyCode::Left => {
                if self.editing {
                    self.editing = false;
                }
                SettingsAction::None
            }
            _ => SettingsAction::None,
        }
    }

    pub fn selected_value(&self) -> &str {
        if self.options[self.category_index].is_empty() {
            return "";
        }
        &self.options[self.category_index][self.option_index]
    }

    pub fn selected_category(&self) -> &str {
        &self.categories[self.category_index]
    }
}
