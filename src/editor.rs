use std::cmp;

pub struct Editor {
    pub lines: Vec<String>,
    pub cursor_row: usize,
    pub cursor_col: usize,
    pub scroll_row: usize,
    pub scroll_col: usize,
    pub file_path: Option<String>,
    pub modified: bool,
    pub preferred_col: Option<usize>, // for vertical movement
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
        self.lines.iter()
            .map(|line| line.split_whitespace().count())
            .sum()
    }

    pub fn char_count(&self) -> usize {
        self.lines.iter().map(|line| line.chars().count()).sum()
    }

    pub fn reading_time_minutes(&self) -> usize {
        let words = self.word_count();
        if words == 0 { 0 } else { cmp::max(1, words / 200) }
    }

    // ─── Cursor Movement ──────────────────────────────────────

    pub fn move_left(&mut self) {
        if self.cursor_col > 0 {
            self.cursor_col -= 1;
            self.preferred_col = Some(self.cursor_col);
        } else if self.cursor_row > 0 {
            self.cursor_row -= 1;
            self.cursor_col = self.lines[self.cursor_row].len();
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
        while col > 0 && line.chars().nth(col - 1).map_or(false, |c| c.is_whitespace()) {
            col -= 1;
        }
        // Skip word chars
        while col > 0 && line.chars().nth(col - 1).map_or(false, |c| !c.is_whitespace()) {
            col -= 1;
        }
        self.cursor_col = col;
        self.preferred_col = Some(col);
        self.ensure_scroll();
    }

    pub fn move_word_right(&mut self) {
        let line = &self.lines[self.cursor_row];
        let mut col = self.cursor_col;
        let len = line.len();
        // Skip word chars
        while col < len && line.chars().nth(col).map_or(false, |c| !c.is_whitespace()) {
            col += 1;
        }
        // Skip whitespace
        while col < len && line.chars().nth(col).map_or(false, |c| c.is_whitespace()) {
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
        let byte_pos = line.char_indices()
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
        let byte_pos = line.char_indices()
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
            let byte_pos = line.char_indices()
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
            self.cursor_col = self.lines[self.cursor_row].len();
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
            let byte_pos = line.char_indices()
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

    // ─── Helpers ──────────────────────────────────────────────

    fn current_line_len(&self) -> usize {
        self.lines.get(self.cursor_row).map_or(0, |l| l.chars().count())
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
