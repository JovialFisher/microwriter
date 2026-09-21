use std::cmp;
use std::collections::BTreeSet;

/// Strip leading/trailing non-alphanumeric chars (except underscore) from a word.
pub fn clean_word(word: &str) -> &str {
    word.trim_matches(|c: char| !c.is_alphanumeric() && c != '_')
}

pub struct Editor {
    pub lines: Vec<String>,
    pub cursor_row: usize,
    pub cursor_col: usize,
    pub scroll_row: usize,
    pub scroll_col: usize,
    pub file_path: Option<String>,
    pub modified: bool,
    pub preferred_col: Option<usize>, // for vertical movement
    pub autocomplete_matches: Vec<String>,
    pub autocomplete_index: usize,
    pub autocomplete_prefix: String,
    pub cross_file_words: Vec<String>,
    pub ghost_suggestion: String,
}

impl Editor {
    pub fn new() -> Self {
        Self {
            lines: vec![String::new()],
            cursor_row: 0,
            cursor_col: 0,
            scroll_row: 0,
            scroll_col: 0,
            file_path: None,
            modified: false,
            preferred_col: None,
            autocomplete_matches: Vec::new(),
            autocomplete_index: 0,
            autocomplete_prefix: String::new(),
            cross_file_words: Vec::new(),
            ghost_suggestion: String::new(),
        }
    }

    pub fn set_content(&mut self, content: &str) {
        self.lines = content.lines().map(|l| l.to_string()).collect();
        if self.lines.is_empty() {
            self.lines.push(String::new());
        }
        self.cursor_row = 0;
        self.cursor_col = 0;
        self.scroll_row = 0;
        self.scroll_col = 0;
        self.preferred_col = None;
    }

    pub fn get_content(&self) -> String {
        self.lines.join("\n")
    }

    pub fn word_count(&self) -> usize {
        self.lines
            .iter()
            .map(|line| line.split_whitespace().count())
            .sum()
    }

    pub fn char_count(&self) -> usize {
        self.lines.iter().map(|line| line.chars().count()).sum()
    }

    pub fn reading_time_minutes(&self) -> usize {
        let words = self.word_count();
        if words == 0 {
            0
        } else {
            cmp::max(1, words / 200)
        }
    }

    // ─── Cursor Movement ──────────────────────────────────────

    pub fn move_left(&mut self) {
        if self.cursor_col > 0 {
            self.cursor_col -= 1;
            self.preferred_col = Some(self.cursor_col);
        } else if self.cursor_row > 0 {
            self.cursor_row -= 1;
            self.cursor_col = self.current_line_len();
            self.preferred_col = Some(self.cursor_col);
        }
        self.ensure_scroll();
    }

    pub fn move_right(&mut self) {
        if self.cursor_col < self.current_line_len() {
            self.cursor_col += 1;
        } else if self.cursor_row < self.lines.len() - 1 {
            self.cursor_row += 1;
            self.cursor_col = 0;
        }
        self.preferred_col = Some(self.cursor_col);
        self.ensure_scroll();
    }

    pub fn move_up(&mut self) {
        if self.cursor_row > 0 {
            self.cursor_row -= 1;
            let target = self.preferred_col.unwrap_or(self.cursor_col);
            self.cursor_col = cmp::min(target, self.current_line_len());
        }
        self.ensure_scroll();
    }

    pub fn move_down(&mut self) {
        if self.cursor_row < self.lines.len() - 1 {
            self.cursor_row += 1;
            let target = self.preferred_col.unwrap_or(self.cursor_col);
            self.cursor_col = cmp::min(target, self.current_line_len());
        }
        self.ensure_scroll();
    }

    pub fn move_word_left(&mut self) {
        let line = &self.lines[self.cursor_row];
        let mut col = self.cursor_col;
        // Skip whitespace
        while col > 0 && line.chars().nth(col - 1).is_some_and(|c| c.is_whitespace()) {
            col -= 1;
        }
        // Skip word chars
        while col > 0
            && line
                .chars()
                .nth(col - 1)
                .is_some_and(|c| !c.is_whitespace())
        {
            col -= 1;
        }
        self.cursor_col = col;
        self.preferred_col = Some(col);
        self.ensure_scroll();
    }

    pub fn move_word_right(&mut self) {
        let line = &self.lines[self.cursor_row];
        let mut col = self.cursor_col;
        let len = line.chars().count();
        // Skip word chars
        while col < len && line.chars().nth(col).is_some_and(|c| !c.is_whitespace()) {
            col += 1;
        }
        // Skip whitespace
        while col < len && line.chars().nth(col).is_some_and(|c| c.is_whitespace()) {
            col += 1;
        }
        self.cursor_col = col;
        self.preferred_col = Some(col);
        self.ensure_scroll();
    }

    pub fn move_line_start(&mut self) {
        self.cursor_col = 0;
        self.preferred_col = Some(0);
        self.ensure_scroll();
    }

    pub fn move_line_end(&mut self) {
        self.cursor_col = self.current_line_len();
        self.preferred_col = Some(self.cursor_col);
        self.ensure_scroll();
    }

    pub fn move_top(&mut self) {
        self.cursor_row = 0;
        self.cursor_col = 0;
        self.preferred_col = Some(0);
        self.ensure_scroll();
    }

    pub fn move_bottom(&mut self) {
        self.cursor_row = self.lines.len() - 1;
        self.cursor_col = self.current_line_len();
        self.preferred_col = Some(self.cursor_col);
        self.ensure_scroll();
    }

    pub fn page_up(&mut self) {
        self.cursor_row = self.cursor_row.saturating_sub(20);
        self.ensure_scroll();
    }

    pub fn page_down(&mut self) {
        self.cursor_row = cmp::min(self.cursor_row + 20, self.lines.len() - 1);
        self.ensure_scroll();
    }

    pub fn scroll_up(&mut self, amount: usize) {
        self.scroll_row = self.scroll_row.saturating_sub(amount);
    }

    pub fn scroll_down(&mut self, amount: usize) {
        self.scroll_row = cmp::min(self.scroll_row + amount, self.lines.len().saturating_sub(1));
    }

    // ─── Editing ──────────────────────────────────────────────

    pub fn insert_char(&mut self, c: char) {
        self.modified = true;
        let line = &mut self.lines[self.cursor_row];
        let byte_pos = line
            .char_indices()
            .nth(self.cursor_col)
            .map(|(i, _)| i)
            .unwrap_or(line.len());
        line.insert(byte_pos, c);
        self.cursor_col += 1;
        self.preferred_col = Some(self.cursor_col);
    }

    pub fn insert_newline(&mut self) {
        self.modified = true;
        let line = &mut self.lines[self.cursor_row];
        let byte_pos = line
            .char_indices()
            .nth(self.cursor_col)
            .map(|(i, _)| i)
            .unwrap_or(line.len());
        let rest = line[byte_pos..].to_string();
        line.truncate(byte_pos);
        self.lines.insert(self.cursor_row + 1, rest);
        self.cursor_row += 1;
        self.cursor_col = 0;
        self.preferred_col = Some(0);
        self.ensure_scroll();
    }

    pub fn backspace(&mut self) {
        if self.cursor_col > 0 {
            self.modified = true;
            let line = &mut self.lines[self.cursor_row];
            let byte_pos = line
                .char_indices()
                .nth(self.cursor_col - 1)
                .map(|(i, _)| i)
                .unwrap_or(0);
            line.remove(byte_pos);
            self.cursor_col -= 1;
            self.preferred_col = Some(self.cursor_col);
        } else if self.cursor_row > 0 {
            self.modified = true;
            let current_line = self.lines.remove(self.cursor_row);
            self.cursor_row -= 1;
            self.cursor_col = self.current_line_len();
            self.lines[self.cursor_row].push_str(&current_line);
            self.preferred_col = Some(self.cursor_col);
        }
        self.ensure_scroll();
    }

    pub fn delete_forward(&mut self) {
        let line_len = self.current_line_len();
        if self.cursor_col < line_len {
            self.modified = true;
            let line = &mut self.lines[self.cursor_row];
            let byte_pos = line
                .char_indices()
                .nth(self.cursor_col)
                .map(|(i, _)| i)
                .unwrap_or(line.len());
            line.remove(byte_pos);
        } else if self.cursor_row < self.lines.len() - 1 {
            self.modified = true;
            let next_line = self.lines.remove(self.cursor_row + 1);
            self.lines[self.cursor_row].push_str(&next_line);
        }
    }

    // ─── Autocomplete ─────────────────────────────────────────

    /// Collect all unique words from the document and cross-file sources (len >= 2, sorted)
    fn collect_words(&self) -> Vec<String> {
        let mut words: BTreeSet<String> = BTreeSet::new();
        for line in &self.lines {
            for word in line.split_whitespace() {
                let cleaned = clean_word(word);
                if cleaned.len() >= 2 {
                    words.insert(cleaned.to_string());
                }
            }
        }
        // Also include cross-file words from recent files
        for word in &self.cross_file_words {
            if word.len() >= 2 {
                words.insert(word.clone());
            }
        }
        words.into_iter().collect()
    }

    /// Extract the word prefix before the cursor on the current line
    fn current_word_prefix(&self) -> (usize, String) {
        let line = &self.lines[self.cursor_row];
        let chars: Vec<char> = line.chars().collect();
        let mut start = self.cursor_col;
        while start > 0 {
            let c = chars[start - 1];
            if c.is_alphanumeric() || c == '_' {
                start -= 1;
            } else {
                break;
            }
        }
        let prefix: String = chars[start..self.cursor_col].iter().collect();
        (start, prefix)
    }

    /// Refresh the ghost suggestion — the top-match suffix shown in gray after the cursor.
    pub fn refresh_ghost(&mut self) {
        self.ghost_suggestion.clear();
        let (_, prefix) = self.current_word_prefix();
        if prefix.is_empty() {
            return;
        }
        let all_words = self.collect_words();
        let prefix_lower = prefix.to_lowercase();
        let prefix_len = prefix.chars().count();
        if let Some(matched) = all_words.into_iter().find(|w| {
            w.to_lowercase().starts_with(&prefix_lower) && w.to_lowercase() != prefix_lower
        }) {
            let ghost: String = matched.chars().skip(prefix_len).collect();
            self.ghost_suggestion = ghost;
        }
    }

    /// Check if the cursor is at a word boundary (next char is non-word or end of line).
    pub fn at_word_boundary(&self) -> bool {
        let line = &self.lines[self.cursor_row];
        let next_char = line.chars().nth(self.cursor_col);
        next_char.map_or(true, |c| !c.is_alphanumeric() && c != '_')
    }

    /// Accept the ghost suggestion, inserting its text at the cursor.
    /// Only accepts when the cursor is at a word boundary (ghost is visible).
    pub fn accept_ghost(&mut self) -> bool {
        if self.ghost_suggestion.is_empty() || !self.at_word_boundary() {
            self.ghost_suggestion.clear();
            return false;
        }
        let ghost = std::mem::take(&mut self.ghost_suggestion);
        self.autocomplete_matches.clear();
        self.autocomplete_prefix.clear();
        let line = &mut self.lines[self.cursor_row];
        let byte_pos = line
            .char_indices()
            .nth(self.cursor_col)
            .map(|(i, _)| i)
            .unwrap_or(line.len());
        line.insert_str(byte_pos, &ghost);
        self.cursor_col += ghost.chars().count();
        self.preferred_col = Some(self.cursor_col);
        self.modified = true;
        true
    }

    /// Attempt word autocompletion. Returns true if a completion was applied.
    pub fn try_autocomplete(&mut self) -> bool {
        let (word_start, prefix) = self.current_word_prefix();

        if prefix.is_empty() {
            self.autocomplete_matches.clear();
            self.autocomplete_prefix.clear();
            return false;
        }

        // Same prefix as before → cycle to next match
        if prefix == self.autocomplete_prefix && !self.autocomplete_matches.is_empty() {
            self.autocomplete_index =
                (self.autocomplete_index + 1) % self.autocomplete_matches.len();
        } else {
            // New prefix → find fresh matches
            let all_words = self.collect_words();
            let prefix_lower = prefix.to_lowercase();
            self.autocomplete_matches = all_words
                .into_iter()
                .filter(|w| {
                    w.to_lowercase().starts_with(&prefix_lower) && w.to_lowercase() != prefix_lower
                })
                .collect();
            if self.autocomplete_matches.is_empty() {
                self.autocomplete_prefix.clear();
                return false;
            }
            self.autocomplete_index = 0;
            self.autocomplete_prefix = prefix.clone();
        }

        // Replace the prefix with the completed word
        let completed = self.autocomplete_matches[self.autocomplete_index].clone();
        let line = &mut self.lines[self.cursor_row];
        let byte_start = line
            .char_indices()
            .nth(word_start)
            .map(|(i, _)| i)
            .unwrap_or(line.len());
        let byte_end = line
            .char_indices()
            .nth(self.cursor_col)
            .map(|(i, _)| i)
            .unwrap_or(line.len());
        line.drain(byte_start..byte_end);
        line.insert_str(byte_start, &completed);
        self.cursor_col = word_start + completed.chars().count();
        self.preferred_col = Some(self.cursor_col);
        self.modified = true;
        true
    }

    // ─── Helpers ──────────────────────────────────────────────

    fn current_line_len(&self) -> usize {
        self.lines
            .get(self.cursor_row)
            .map_or(0, |l| l.chars().count())
    }

    fn ensure_scroll(&mut self) {
        // Vertical scroll
        let visible_rows = 20; // approximate
        if self.cursor_row < self.scroll_row {
            self.scroll_row = self.cursor_row;
        }
        if self.cursor_row >= self.scroll_row + visible_rows {
            self.scroll_row = self.cursor_row - visible_rows + 1;
        }
    }

    pub fn visible_start(&self) -> usize {
        self.scroll_row
    }
}
