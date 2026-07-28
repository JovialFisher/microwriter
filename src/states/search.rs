use crossterm::event::{KeyCode, KeyEvent};

/// Fuzzy matching score. Returns -1 for no match, higher = better match.
pub fn fuzzy_match(pattern: &str, text: &str) -> i32 {
    if pattern.is_empty() {
        return 0;
    }

    let pattern_chars: Vec<char> = pattern.chars().collect();
    let text_chars: Vec<char> = text.chars().collect();

    let mut pattern_idx = 0;
    let mut score: i32 = 0;
    let mut prev_match = false;
    let mut consecutive_bonus = 0;

    for (i, &tc) in text_chars.iter().enumerate() {
        if pattern_idx >= pattern_chars.len() {
            break;
        }

        let pc = pattern_chars[pattern_idx].to_ascii_lowercase();

        if tc.to_ascii_lowercase() == pc {
            pattern_idx += 1;

            let position_bonus = 100 - (i as i32).min(99);
            score += position_bonus;

            if prev_match {
                consecutive_bonus += 10;
                score += consecutive_bonus;
            } else {
                consecutive_bonus = 0;
            }

            if i == 0
                || text_chars[i - 1] == ' '
                || text_chars[i - 1] == '_'
                || text_chars[i - 1] == '-'
            {
                score += 50;
            }

            prev_match = true;
        } else {
            prev_match = false;
            consecutive_bonus = 0;
        }
    }

    if pattern_idx == pattern_chars.len() {
        score
    } else {
        -1
    }
}

pub enum SearchAction {
    Cancel,
    Confirm,
    UpdateQuery,
    None,
}

pub struct SearchState {
    pub query: String,
    pub results: Vec<String>,
    pub index: usize,
}

impl SearchState {
    pub fn new() -> Self {
        Self {
            query: String::new(),
            results: Vec::new(),
            index: 0,
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> SearchAction {
        match key.code {
            KeyCode::Esc => {
                self.query.clear();
                SearchAction::Cancel
            }
            KeyCode::Enter => SearchAction::Confirm,
            KeyCode::Backspace => {
                self.query.pop();
                self.index = 0;
                SearchAction::UpdateQuery
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if self.index > 0 {
                    self.index -= 1;
                }
                SearchAction::None
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if self.index < self.results.len().saturating_sub(1) {
                    self.index += 1;
                }
                SearchAction::None
            }
            KeyCode::Char(c) => {
                self.query.push(c);
                self.index = 0;
                SearchAction::UpdateQuery
            }
            _ => SearchAction::None,
        }
    }

    pub fn selected_path(&self) -> Option<String> {
        if self.results.is_empty() {
            return None;
        }
        let entry = &self.results[self.index];
        Some(entry.split('|').nth(1).unwrap_or(entry).to_string())
    }
}
