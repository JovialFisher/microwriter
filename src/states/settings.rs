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
                "alignment".into(),
                "autosave".into(),
                "default folder".into(),
                "timestamp filenames".into(),
                "tabs/spaces".into(),
                "typewriter scroll".into(),
            ],
            category_index: 0,
            options: vec![
                vec![
                    "dark".into(),
                    "light".into(),
                    "paper".into(),
                    "amber terminal".into(),
                    "green phosphor".into(),
                    "nord".into(),
                    "solarized dark".into(),
                    "terminal default".into(),
                ],
                vec!["block".into(), "beam".into(), "underline".into()],
                vec!["off".into(), "relative".into(), "absolute".into()],
                vec!["on".into(), "off".into()],
                vec![
                    "left".into(),
                    "center".into(),
                    "right".into(),
                    "justified".into(),
                ],
                vec![
                    "disabled".into(),
                    "15 sec".into(),
                    "30 sec".into(),
                    "1 min".into(),
                    "5 min".into(),
                ],
                vec![],
                vec!["on".into(), "off".into()],
                vec!["tabs".into(), "spaces".into()],
                vec!["on".into(), "off".into()],
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
                    // Wrap so you never have to backtrack: up past the first
                    // option lands on the last.
                    let count = self.options[self.category_index].len();
                    if count > 0 {
                        self.option_index = (self.option_index + count - 1) % count;
                    }
                } else {
                    let count = self.categories.len();
                    if count > 0 {
                        self.category_index = (self.category_index + count - 1) % count;
                        self.option_index = 0;
                    }
                }
                SettingsAction::None
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if self.editing {
                    // Wrap so holding down cycles the options: down past the
                    // last option lands on the first.
                    let count = self.options[self.category_index].len();
                    if count > 0 {
                        self.option_index = (self.option_index + 1) % count;
                    }
                } else {
                    let count = self.categories.len();
                    if count > 0 {
                        self.category_index = (self.category_index + 1) % count;
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

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::KeyModifiers;

    fn press(state: &mut SettingsState, code: KeyCode) {
        state.handle_key(KeyEvent::new(code, KeyModifiers::NONE), "");
    }

    #[test]
    fn option_selection_wraps_while_editing() {
        let mut state = SettingsState::new();
        state.category_index = 3; // "word wrap": on / off
        state.editing = true;

        state.option_index = 1;
        press(&mut state, KeyCode::Down);
        assert_eq!(
            state.option_index, 0,
            "down from the last wraps to the first"
        );

        press(&mut state, KeyCode::Up);
        assert_eq!(state.option_index, 1, "up from the first wraps to the last");
    }

    #[test]
    fn category_selection_wraps_at_the_ends() {
        let mut state = SettingsState::new();
        let last = state.categories.len() - 1;

        state.category_index = last;
        press(&mut state, KeyCode::Down);
        assert_eq!(
            state.category_index, 0,
            "down from the last wraps to the first"
        );

        press(&mut state, KeyCode::Up);
        assert_eq!(
            state.category_index, last,
            "up from the first wraps to the last"
        );
    }
}
