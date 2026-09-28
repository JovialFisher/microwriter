use std::cell::Cell;
use std::cmp;
use std::collections::BTreeSet;
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

/// Strip leading/trailing non-alphanumeric chars (except underscore) from a word.
pub fn clean_word(word: &str) -> &str {
    word.trim_matches(|c: char| !c.is_alphanumeric() && c != '_')
}

/// Text alignment for the writing surface.
///
/// Alignment is a view setting only: the saved file always keeps the raw
/// text, so switching alignment never rewrites the document.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Alignment {
    #[default]
    Left,
    Center,
    Right,
    Justify,
}

impl Alignment {
    /// Parse a config value, falling back to left for anything unknown.
    pub fn parse(value: &str) -> Self {
        match value {
            "center" => Alignment::Center,
            "right" => Alignment::Right,
            "justified" => Alignment::Justify,
            _ => Alignment::Left,
        }
    }

    /// The name used in `config.toml` and the status line.
    pub fn label(self) -> &'static str {
        match self {
            Alignment::Left => "left",
            Alignment::Center => "center",
            Alignment::Right => "right",
            Alignment::Justify => "justified",
        }
    }

    /// Cycle left → center → right → justified → left.
    pub fn next(self) -> Self {
        match self {
            Alignment::Left => Alignment::Center,
            Alignment::Center => Alignment::Right,
            Alignment::Right => Alignment::Justify,
            Alignment::Justify => Alignment::Left,
        }
    }
}

/// Whether widening `row` to `width` columns would look like justification.
///
/// A row is stretched only when it has an interior run of spaces to grow and
/// the paragraph continues below it — the final row of a paragraph stays
/// ragged, exactly as in a printed book.
pub fn should_justify(row: &str, width: usize, continues: bool) -> bool {
    if width == 0 || UnicodeWidthStr::width(row) >= width {
        return false;
    }
    continues && row.trim_matches(' ').contains(' ')
}

/// Widen the interior space runs of `line` so it fills exactly `width` columns.
///
/// Free space is spread across the gaps from the left. When `cursor_col` is
/// given (a char index into the original line) the returned cursor index is
/// remapped onto the padded text, so the caret keeps its place while typing.
/// Lines that are already full width, empty, or have no interior gap are
/// returned unchanged.
pub fn justify_line(
    line: &str,
    width: usize,
    cursor_col: Option<usize>,
) -> (String, Option<usize>) {
    let line_width = UnicodeWidthStr::width(line);
    if width == 0 || line_width >= width || line.is_empty() {
        return (line.to_string(), cursor_col);
    }

    let chars: Vec<char> = line.chars().collect();
    // Interior space runs: maximal runs of spaces with text on both sides.
    let mut gaps: Vec<(usize, usize)> = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == ' ' {
            let start = i;
            while i < chars.len() && chars[i] == ' ' {
                i += 1;
            }
            if start > 0 && i < chars.len() {
                gaps.push((start, i - start));
            }
        } else {
            i += 1;
        }
    }
    if gaps.is_empty() {
        return (line.to_string(), cursor_col);
    }

    let free = width - line_width;
    let base = free / gaps.len();
    let remainder = free % gaps.len();

    let mut out = String::with_capacity(line.len() + free);
    let mut cursor = cursor_col;
    let mut gap_index = 0;
    let mut index = 0;
    while index < chars.len() {
        if gap_index < gaps.len() && index == gaps[gap_index].0 {
            let (_, run_len) = gaps[gap_index];
            let extra = base + usize::from(gap_index < remainder);
            if extra > 0 {
                // Padding lands after the caret when the caret sits at the very
                // start of the gap, so only shift carets past that point.
                if cursor.is_some_and(|c| c > index) {
                    cursor = cursor.map(|c| c + extra);
                }
                for _ in 0..extra {
                    out.push(' ');
                }
            }
            for _ in 0..run_len {
                out.push(' ');
            }
            index += run_len;
            gap_index += 1;
        } else {
            out.push(chars[index]);
            index += 1;
        }
    }

    (out, cursor)
}

/// Soft-wrap `line` into rows of at most `width` display columns.
///
/// A row breaks after the last space that fits, so words stay whole; a word
/// wider than the row is hard-broken so no character is ever hidden. Spaces at
/// a break are consumed, as in a word processor. `width == 0` disables
/// wrapping and yields the line unchanged.
///
/// Returns `(start, end)` char-index pairs into `line`, always at least one.
pub fn wrap_line(line: &str, width: usize) -> Vec<(usize, usize)> {
    // Fast path: most lines already fit, and skipping the char buffer there
    // keeps laying out a large document cheap.
    if width == 0 || UnicodeWidthStr::width(line) <= width {
        return vec![(0, line.chars().count())];
    }
    let chars: Vec<char> = line.chars().collect();

    let mut rows = Vec::new();
    let mut start = 0;
    while start < chars.len() {
        // Longest run of characters that still fits in `width` columns.
        let mut end = start;
        let mut columns = 0;
        while end < chars.len() {
            let ch_width = UnicodeWidthChar::width(chars[end]).unwrap_or(0);
            if columns + ch_width > width {
                break;
            }
            columns += ch_width;
            end += 1;
        }

        if end == chars.len() {
            rows.push((start, chars.len()));
            break;
        }

        // Breaking at the last space reads better than cutting a word in half.
        let break_at = chars[start..end]
            .iter()
            .rposition(|c| *c == ' ')
            .map_or(end, |offset| start + offset);
        // A single glyph wider than the row still has to make progress.
        let break_at = if break_at > start {
            break_at
        } else {
            end.max(start + 1)
        };
        rows.push((start, break_at));

        // Swallow the spaces at the break so the next row starts on a word.
        start = break_at;
        while start < chars.len() && chars[start] == ' ' {
            start += 1;
        }
    }
    rows
}

/// One row of the writing surface: a slice of a logical line after wrapping.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RenderedRow {
    /// The logical line this row was wrapped from.
    pub row: usize,
    /// Char range within that line: `start..end`.
    pub start: usize,
    pub end: usize,
    /// Whether the paragraph continues below, which makes the row a candidate
    /// for justification.
    pub continues: bool,
    /// Char offset of the caret inside `start..end`, when the caret is here.
    pub caret: Option<usize>,
}

/// Undo steps kept before the oldest is dropped.
const UNDO_LIMIT: usize = 500;
/// Bytes of text kept across the whole history, so opening a large document
/// cannot grow the undo stack without bound.
const UNDO_BYTE_BUDGET: usize = 4_000_000;

/// A point in the document's history that can be restored wholesale.
///
/// Snapshots are simple and impossible to get wrong, at the cost of holding a
/// copy of the text per step — which the budget above bounds.
struct Snapshot {
    lines: Vec<String>,
    cursor_row: usize,
    cursor_col: usize,
    /// Revision stamp of this text, so undo can tell whether it is the text
    /// last written to disk.
    revision: u64,
    /// Bytes held in `lines`, so the history can be budgeted.
    weight: usize,
}

/// What produced a history entry. Only runs of the same kind merge, so a
/// newline never dissolves into the typing before it.
#[derive(PartialEq, Eq)]
enum EditKind {
    Insert,
    Delete,
    Other,
}

/// One undo step: the document as it stood *before* the edit, plus where the
/// run currently ends. `anchor` is `None` when later edits must not merge into
/// this step — after a redo, for instance.
struct HistoryEntry {
    before: Snapshot,
    kind: EditKind,
    anchor: Option<(usize, usize)>,
}

/// The state an undone step returns to when redone.
struct RedoEntry {
    state: Snapshot,
    kind: EditKind,
}

pub struct Editor {
    pub lines: Vec<String>,
    pub cursor_row: usize,
    pub cursor_col: usize,
    pub scroll_row: usize,
    pub scroll_col: usize,
    pub file_path: Option<String>,
    /// Stamp of the current text, and the stamp that matches the bytes last
    /// written to disk. Stamps come from a counter that only ever rises, so a
    /// restored state can never be confused with a later edit.
    revision: u64,
    saved_revision: Option<u64>,
    stamp_counter: u64,
    pub preferred_col: Option<usize>, // for vertical movement
    pub autocomplete_matches: Vec<String>,
    pub autocomplete_index: usize,
    pub autocomplete_prefix: String,
    pub cross_file_words: Vec<String>,
    pub ghost_suggestion: String,
    /// Undo steps, oldest first, and the states `Ctrl+Y` walks back through.
    undo_stack: Vec<HistoryEntry>,
    redo_stack: Vec<RedoEntry>,
    /// Columns and rows the renderer currently gives the writing surface. Kept
    /// in `Cell`s because the renderer only holds `&App`, and `0` means "not
    /// laid out yet", which disables soft wrapping.
    viewport_width: Cell<usize>,
    viewport_height: Cell<usize>,
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
            revision: 0,
            saved_revision: Some(0),
            stamp_counter: 0,
            preferred_col: None,
            autocomplete_matches: Vec::new(),
            autocomplete_index: 0,
            autocomplete_prefix: String::new(),
            cross_file_words: Vec::new(),
            ghost_suggestion: String::new(),
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            viewport_width: Cell::new(0),
            viewport_height: Cell::new(0),
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
        // A different document shares no history with the previous one, and a
        // buffer the caller loaded starts in step with whatever it came from.
        self.undo_stack.clear();
        self.redo_stack.clear();
        self.revision = 0;
        self.saved_revision = Some(0);
        self.stamp_counter = 0;
    }

    /// Whether the buffer has changed since it was last saved.
    pub fn is_modified(&self) -> bool {
        self.saved_revision != Some(self.revision)
    }

    /// Stamp of the current text. It only changes when the text does, so
    /// callers can skip re-reading the buffer while nothing has been typed.
    pub fn revision(&self) -> u64 {
        self.revision
    }

    /// Note that the current text is what is on disk.
    pub fn mark_saved(&mut self) {
        self.saved_revision = Some(self.revision);
        // A save also ends the current undo run, so undoing afterwards cannot
        // reach back past the text that was just written.
        if let Some(entry) = self.undo_stack.last_mut() {
            entry.anchor = None;
        }
    }

    /// Treat the buffer as never saved — a new, untitled note.
    pub fn mark_unsaved(&mut self) {
        self.saved_revision = None;
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
            self.preferred_col = None;
        } else if self.cursor_row > 0 {
            self.cursor_row -= 1;
            self.cursor_col = self.current_line_len();
            self.preferred_col = None;
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
        self.preferred_col = None;
        self.ensure_scroll();
    }

    pub fn move_up(&mut self) {
        self.move_wrapped(-1);
    }

    pub fn move_down(&mut self) {
        self.move_wrapped(1);
    }

    /// Move the caret one rendered row up (`-1`) or down (`1`), holding its
    /// display column so it tracks straight across short and wrapped rows.
    fn move_wrapped(&mut self, delta: isize) {
        let width = self.viewport_width.get();
        let desired = self
            .preferred_col
            .unwrap_or_else(|| self.cursor_visual_col(width));
        let segments = self.segments(self.cursor_row, width);
        let index = self.cursor_segment_index(width);

        let target = if delta > 0 {
            if index + 1 < segments.len() {
                Some((self.cursor_row, segments[index + 1]))
            } else if self.cursor_row + 1 < self.lines.len() {
                let next = self.cursor_row + 1;
                Some((next, self.segments(next, width)[0]))
            } else {
                None
            }
        } else if index > 0 {
            Some((self.cursor_row, segments[index - 1]))
        } else if self.cursor_row > 0 {
            let previous = self.cursor_row - 1;
            let previous_segments = self.segments(previous, width);
            Some((previous, previous_segments[previous_segments.len() - 1]))
        } else {
            None
        };

        let Some((row, (start, end))) = target else {
            return;
        };
        self.cursor_row = row;
        self.cursor_col = start + self.column_within(row, start, end, desired);
        self.preferred_col = Some(desired);
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
        self.preferred_col = None;
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
        self.preferred_col = None;
        self.ensure_scroll();
    }

    pub fn move_line_start(&mut self) {
        self.cursor_col = 0;
        self.preferred_col = None;
        self.ensure_scroll();
    }

    pub fn move_line_end(&mut self) {
        self.cursor_col = self.current_line_len();
        self.preferred_col = None;
        self.ensure_scroll();
    }

    pub fn move_top(&mut self) {
        self.cursor_row = 0;
        self.cursor_col = 0;
        self.preferred_col = None;
        self.ensure_scroll();
    }

    pub fn move_bottom(&mut self) {
        self.cursor_row = self.lines.len() - 1;
        self.cursor_col = self.current_line_len();
        self.preferred_col = None;
        self.ensure_scroll();
    }

    pub fn page_up(&mut self) {
        self.page(-1);
    }

    pub fn page_down(&mut self) {
        self.page(1);
    }

    /// Move the caret by one screenful of rendered rows.
    fn page(&mut self, delta: isize) {
        let rows = self.viewport_height.get().max(1);
        for _ in 0..rows {
            self.move_wrapped(delta);
        }
    }

    pub fn scroll_up(&mut self, amount: usize) {
        self.scroll_row = self.scroll_row.saturating_sub(amount);
    }

    pub fn scroll_down(&mut self, amount: usize) {
        let width = self.viewport_width.get();
        let last = self.total_visual_rows(width).saturating_sub(1);
        self.scroll_row = cmp::min(self.scroll_row + amount, last);
    }

    // ─── Editing ──────────────────────────────────────────────

    pub fn insert_char(&mut self, c: char) {
        self.begin_edit(EditKind::Insert);
        let line = &mut self.lines[self.cursor_row];
        let byte_pos = line
            .char_indices()
            .nth(self.cursor_col)
            .map(|(i, _)| i)
            .unwrap_or(line.len());
        line.insert(byte_pos, c);
        self.cursor_col += 1;
        self.preferred_col = None;
        self.end_edit();
    }

    pub fn insert_newline(&mut self) {
        self.begin_edit(EditKind::Other);
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
        self.preferred_col = None;
        self.end_edit();
        self.ensure_scroll();
    }

    pub fn backspace(&mut self) {
        // Record only when there is something to remove, so a stray Backspace at
        // the start of the document does not leave an empty undo step.
        if self.cursor_col > 0 || self.cursor_row > 0 {
            self.begin_edit(EditKind::Delete);
            if self.cursor_col > 0 {
                let line = &mut self.lines[self.cursor_row];
                let byte_pos = line
                    .char_indices()
                    .nth(self.cursor_col - 1)
                    .map(|(i, _)| i)
                    .unwrap_or(0);
                line.remove(byte_pos);
                self.cursor_col -= 1;
                self.preferred_col = None;
            } else {
                let current_line = self.lines.remove(self.cursor_row);
                self.cursor_row -= 1;
                self.cursor_col = self.current_line_len();
                self.lines[self.cursor_row].push_str(&current_line);
                self.preferred_col = None;
            }
            self.end_edit();
        }
        self.ensure_scroll();
    }

    pub fn delete_forward(&mut self) {
        let line_len = self.current_line_len();
        if self.cursor_col >= line_len && self.cursor_row >= self.lines.len() - 1 {
            return;
        }
        self.begin_edit(EditKind::Delete);
        if self.cursor_col < line_len {
            let line = &mut self.lines[self.cursor_row];
            let byte_pos = line
                .char_indices()
                .nth(self.cursor_col)
                .map(|(i, _)| i)
                .unwrap_or(line.len());
            line.remove(byte_pos);
        } else {
            let next_line = self.lines.remove(self.cursor_row + 1);
            self.lines[self.cursor_row].push_str(&next_line);
        }
        self.end_edit();
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
        next_char.is_none_or(|c| !c.is_alphanumeric() && c != '_')
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
        self.begin_edit(EditKind::Other);
        let line = &mut self.lines[self.cursor_row];
        let byte_pos = line
            .char_indices()
            .nth(self.cursor_col)
            .map(|(i, _)| i)
            .unwrap_or(line.len());
        line.insert_str(byte_pos, &ghost);
        self.cursor_col += ghost.chars().count();
        self.preferred_col = None;
        self.end_edit();
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
        self.begin_edit(EditKind::Other);
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
        self.preferred_col = None;
        self.end_edit();
        true
    }

    // ─── History ─────────────────────────────────────────────

    /// Record the state an edit is about to change, so `Ctrl+Z` can restore it.
    ///
    /// A run of the same kind of edit at the same spot — typing a word, holding
    /// Backspace — collapses into a single undo step.
    fn begin_edit(&mut self, kind: EditKind) {
        let position = (self.cursor_row, self.cursor_col);
        self.redo_stack.clear();
        let merges = matches!(kind, EditKind::Insert | EditKind::Delete)
            && self
                .undo_stack
                .last()
                .is_some_and(|entry| entry.kind == kind && entry.anchor == Some(position));
        if merges {
            return;
        }
        self.undo_stack.push(HistoryEntry {
            before: self.snapshot(),
            kind,
            anchor: None,
        });
        self.trim_history();
    }

    /// Close the edit opened by [`Editor::begin_edit`]: stamp the changed text
    /// with a fresh revision and note where the run now ends, so the next edit
    /// can merge into it.
    fn end_edit(&mut self) {
        self.stamp_counter += 1;
        self.revision = self.stamp_counter;
        if let Some(entry) = self.undo_stack.last_mut() {
            entry.anchor = Some((self.cursor_row, self.cursor_col));
        }
    }

    /// Step the document back one edit, returning false when there is nothing
    /// left to undo.
    pub fn undo(&mut self) -> bool {
        let Some(entry) = self.undo_stack.pop() else {
            return false;
        };
        self.redo_stack.push(RedoEntry {
            state: self.snapshot(),
            kind: entry.kind,
        });
        self.restore(entry.before);
        true
    }

    /// Step forward again after an undo, returning false when there is nothing
    /// left to redo.
    pub fn redo(&mut self) -> bool {
        let Some(entry) = self.redo_stack.pop() else {
            return false;
        };
        self.undo_stack.push(HistoryEntry {
            before: self.snapshot(),
            kind: entry.kind,
            // A redo must not absorb whatever the writer types next.
            anchor: None,
        });
        self.restore(entry.state);
        true
    }

    fn snapshot(&self) -> Snapshot {
        Snapshot {
            lines: self.lines.clone(),
            cursor_row: self.cursor_row,
            cursor_col: self.cursor_col,
            revision: self.revision,
            weight: self.lines.iter().map(String::len).sum(),
        }
    }

    fn restore(&mut self, snapshot: Snapshot) {
        self.lines = snapshot.lines;
        self.cursor_row = snapshot.cursor_row.min(self.lines.len().saturating_sub(1));
        self.cursor_col = snapshot.cursor_col.min(self.current_line_len());
        self.revision = snapshot.revision;
        self.preferred_col = None;
        self.autocomplete_matches.clear();
        self.autocomplete_prefix.clear();
        // Nothing merges across an undo boundary, so the next edit is its own
        // step even when the caret lands exactly where the last run ended.
        if let Some(entry) = self.undo_stack.last_mut() {
            entry.anchor = None;
        }
        self.ensure_scroll();
    }

    /// Drop the oldest steps while the history exceeds either bound.
    fn trim_history(&mut self) {
        let mut weight: usize = self
            .undo_stack
            .iter()
            .map(|entry| entry.before.weight)
            .sum();
        while self.undo_stack.len() > 1
            && (self.undo_stack.len() > UNDO_LIMIT || weight > UNDO_BYTE_BUDGET)
        {
            weight -= self.undo_stack.remove(0).before.weight;
        }
    }

    // ─── Helpers ──────────────────────────────────────────────

    fn current_line_len(&self) -> usize {
        self.lines
            .get(self.cursor_row)
            .map_or(0, |l| l.chars().count())
    }

    /// Keep the caret inside the wrapped viewport, in rendered rows.
    fn ensure_scroll(&mut self) {
        let width = self.viewport_width.get();
        let height = self.viewport_height.get().max(1);
        let caret = self.cursor_visual_row(width);
        if caret < self.scroll_row {
            self.scroll_row = caret;
        }
        if caret >= self.scroll_row + height {
            self.scroll_row = caret + 1 - height;
        }
    }

    /// Tell the editor how much room the writing surface has, so soft wrapping
    /// and scrolling line up with what the renderer draws.
    pub fn set_viewport(&self, width: usize, height: usize) {
        self.viewport_width.set(width);
        self.viewport_height.set(height);
    }

    /// The rows the renderer should draw, starting at the top of the viewport.
    ///
    /// `count` bounds the work to the visible screen instead of the whole
    /// document.
    pub fn visible_rows(&self, count: usize, width: usize) -> Vec<RenderedRow> {
        let caret_segment = self.cursor_segment_index(width);
        let mut rows = Vec::new();
        let mut visual = 0;
        for row in 0..self.lines.len() {
            let segments = self.segments(row, width);
            let last_segment = segments.len() - 1;
            let next_line_blank = self
                .lines
                .get(row + 1)
                .is_none_or(|next| next.trim().is_empty());
            for (index, (start, end)) in segments.into_iter().enumerate() {
                if visual >= self.scroll_row {
                    let caret = (row == self.cursor_row && index == caret_segment)
                        .then(|| self.cursor_col.max(start) - start);
                    rows.push(RenderedRow {
                        row,
                        start,
                        end,
                        continues: !(index == last_segment && next_line_blank),
                        caret,
                    });
                    if rows.len() >= count {
                        return rows;
                    }
                }
                visual += 1;
            }
        }
        rows
    }

    /// The characters of logical line `row` between `start` and `end`.
    pub fn segment_text(&self, row: usize, start: usize, end: usize) -> String {
        self.lines
            .get(row)
            .map(|line| {
                line.chars()
                    .skip(start)
                    .take(end.saturating_sub(start))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Wrapped rows of logical line `row`, as `(start, end)` char ranges.
    fn segments(&self, row: usize, width: usize) -> Vec<(usize, usize)> {
        self.lines
            .get(row)
            .map_or_else(|| vec![(0, 0)], |line| wrap_line(line, width))
    }

    /// Index of the wrapped row inside `cursor_row` that holds the caret.
    fn cursor_segment_index(&self, width: usize) -> usize {
        let segments = self.segments(self.cursor_row, width);
        segments
            .iter()
            .position(|(_, end)| self.cursor_col < *end)
            .unwrap_or(segments.len() - 1)
    }

    /// Display column of the caret inside its wrapped row.
    fn cursor_visual_col(&self, width: usize) -> usize {
        let segments = self.segments(self.cursor_row, width);
        let index = self.cursor_segment_index(width);
        let (start, _) = segments[index];
        self.width_between(self.cursor_row, start, self.cursor_col.max(start))
    }

    /// Rendered row index of the caret.
    fn cursor_visual_row(&self, width: usize) -> usize {
        let before: usize = (0..self.cursor_row)
            .map(|row| self.segments(row, width).len())
            .sum();
        before + self.cursor_segment_index(width)
    }

    /// Number of rendered rows in the whole document.
    fn total_visual_rows(&self, width: usize) -> usize {
        (0..self.lines.len())
            .map(|row| self.segments(row, width).len())
            .sum()
    }

    /// Display width of logical line `row` between two char indices.
    fn width_between(&self, row: usize, from: usize, to: usize) -> usize {
        self.lines.get(row).map_or(0, |line| {
            let text: String = line
                .chars()
                .skip(from)
                .take(to.saturating_sub(from))
                .collect();
            UnicodeWidthStr::width(text.as_str())
        })
    }

    /// Longest char offset from `start` whose display column stays within
    /// `desired`.
    fn column_within(&self, row: usize, start: usize, end: usize, desired: usize) -> usize {
        let Some(line) = self.lines.get(row) else {
            return 0;
        };
        let mut offset = 0;
        let mut columns = 0;
        for ch in line.chars().skip(start).take(end.saturating_sub(start)) {
            let ch_width = UnicodeWidthChar::width(ch).unwrap_or(0);
            if columns + ch_width > desired {
                break;
            }
            columns += ch_width;
            offset += 1;
        }
        offset
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn padded(line: &str, width: usize) -> String {
        justify_line(line, width, None).0
    }

    #[test]
    fn alignment_parses_known_labels_only() {
        assert_eq!(Alignment::parse("center"), Alignment::Center);
        assert_eq!(Alignment::parse("right"), Alignment::Right);
        assert_eq!(Alignment::parse("justified"), Alignment::Justify);
        assert_eq!(Alignment::parse("left"), Alignment::Left);
        assert_eq!(Alignment::parse("nonsense"), Alignment::Left);
        assert_eq!(Alignment::parse(""), Alignment::Left);
    }

    #[test]
    fn alignment_cycles_and_round_trips() {
        let mut alignment = Alignment::Left;
        for expected in [
            Alignment::Center,
            Alignment::Right,
            Alignment::Justify,
            Alignment::Left,
        ] {
            alignment = alignment.next();
            assert_eq!(alignment, expected);
        }
        for alignment in [
            Alignment::Left,
            Alignment::Center,
            Alignment::Right,
            Alignment::Justify,
        ] {
            assert_eq!(Alignment::parse(alignment.label()), alignment);
        }
    }

    #[test]
    fn justification_fills_the_width() {
        assert_eq!(
            padded("hello world", 20),
            format!("hello{}world", " ".repeat(10))
        );
        assert_eq!(padded("a b c", 8), "a   b  c");
        assert_eq!(UnicodeWidthStr::width(padded("a b c", 8).as_str()), 8);
    }

    #[test]
    fn justification_leaves_unusable_lines_alone() {
        assert_eq!(padded("hello world", 10), "hello world"); // already full
        assert_eq!(padded("word", 40), "word"); // no interior gap
        assert_eq!(padded("", 40), "");
        assert_eq!(padded("  indented", 40), "  indented");
        assert_eq!(padded("   a b", 12), "   a       b");
    }

    #[test]
    fn justification_keeps_the_caret_in_place() {
        // "ab|cd" → gap widens after the caret, so the caret moves with its text.
        assert_eq!(justify_line("ab cd", 8, Some(3)).1, Some(6));
        // Caret at the gap start stays put: padding grows to its right.
        assert_eq!(justify_line("ab cd", 8, Some(2)).1, Some(2));
        // Caret at end of line maps to the end of the widened line.
        assert_eq!(justify_line("ab cd", 8, Some(5)).1, Some(8));
        let (text, cursor) = justify_line("ab cd", 8, Some(3));
        assert_eq!(cursor.map(|c| text.chars().nth(c)), Some(Some('c')));
    }

    #[test]
    fn last_row_of_a_paragraph_stays_ragged() {
        assert!(should_justify("hello world", 20, true));
        assert!(!should_justify("hello world", 20, false));
        assert!(!should_justify("hello world", 11, true)); // already full
        assert!(!should_justify("word", 20, true)); // no interior gap
    }

    #[test]
    fn wrap_breaks_on_spaces() {
        assert_eq!(wrap_line("hello world", 5), vec![(0, 5), (6, 11)]);
        assert_eq!(wrap_line("aa bb", 3), vec![(0, 2), (3, 5)]);
        assert_eq!(wrap_line("hello", 5), vec![(0, 5)]);
        assert_eq!(wrap_line("", 5), vec![(0, 0)]);
        assert_eq!(wrap_line("hello world", 0), vec![(0, 11)]);
    }

    #[test]
    fn wrap_hard_breaks_words_wider_than_the_row() {
        assert_eq!(wrap_line("abcdef", 3), vec![(0, 3), (3, 6)]);
        assert_eq!(wrap_line("ab cd", 2), vec![(0, 2), (3, 5)]);
    }

    #[test]
    fn wrapping_never_hides_characters() {
        let line = "the quick brown fox jumps over the lazy dog";
        let rebuilt = wrap_line(line, 7)
            .iter()
            .map(|(start, end)| {
                line.chars()
                    .skip(*start)
                    .take(end - start)
                    .collect::<String>()
            })
            .collect::<Vec<_>>()
            .join(" ");
        assert_eq!(rebuilt, line);
    }

    #[test]
    fn caret_tracks_its_column_between_wrapped_rows() {
        // Wraps to "aa" / "bb" / "cc dd".
        let mut editor = Editor::new();
        editor.set_content("aa bb cc dd");
        editor.set_viewport(5, 10);
        editor.cursor_col = 4; // second "b" on the middle row
        editor.move_down();
        assert_eq!((editor.cursor_row, editor.cursor_col), (0, 7)); // second "c"
        editor.move_up();
        assert_eq!((editor.cursor_row, editor.cursor_col), (0, 4)); // back
    }

    #[test]
    fn wrapped_navigation_crosses_logical_lines() {
        let mut editor = Editor::new();
        editor.set_content("aa bb cc\ndd");
        editor.set_viewport(5, 10);
        editor.move_line_end();
        editor.move_down();
        assert_eq!((editor.cursor_row, editor.cursor_col), (1, 2));
    }

    #[test]
    fn undo_and_redo_walk_through_edits() {
        let mut editor = Editor::new();
        editor.insert_char('a');
        editor.insert_newline();
        editor.insert_char('b');
        assert_eq!(editor.get_content(), "a\nb");

        assert!(editor.undo());
        assert_eq!(editor.get_content(), "a\n");
        assert!(editor.undo());
        assert_eq!(editor.get_content(), "a");
        assert!(editor.undo());
        assert_eq!(editor.get_content(), "");
        assert!(!editor.undo()); // nothing left

        assert!(editor.redo());
        assert_eq!(editor.get_content(), "a");
        assert!(editor.redo());
        assert_eq!(editor.get_content(), "a\n");
        assert!(editor.redo());
        assert_eq!(editor.get_content(), "a\nb");
        assert!(!editor.redo());
    }

    #[test]
    fn a_burst_of_typing_is_one_undo_step() {
        let mut editor = Editor::new();
        for c in "hello".chars() {
            editor.insert_char(c);
        }
        assert!(editor.is_modified());
        assert!(editor.undo());
        assert_eq!(editor.get_content(), "");
        assert!(!editor.is_modified());
        assert!(!editor.undo());
    }

    #[test]
    fn a_burst_of_deleting_is_one_undo_step() {
        let mut editor = Editor::new();
        editor.set_content("hello");
        editor.move_line_end();
        for _ in 0..3 {
            editor.backspace();
        }
        assert_eq!(editor.get_content(), "he");
        assert!(editor.undo());
        assert_eq!(editor.get_content(), "hello");
        assert!(!editor.undo());
    }

    #[test]
    fn moving_away_starts_a_new_undo_step() {
        let mut editor = Editor::new();
        editor.insert_char('a');
        editor.move_left();
        editor.insert_char('b');
        assert_eq!(editor.get_content(), "ba");
        assert!(editor.undo());
        assert_eq!(editor.get_content(), "a");
        assert!(editor.undo());
        assert_eq!(editor.get_content(), "");
    }

    #[test]
    fn a_new_edit_drops_the_redo_stack() {
        let mut editor = Editor::new();
        editor.insert_char('a');
        assert!(editor.undo());
        editor.insert_char('b');
        assert!(!editor.redo());
        assert_eq!(editor.get_content(), "b");
    }

    #[test]
    fn deleting_at_the_boundaries_records_nothing() {
        let mut editor = Editor::new();
        editor.backspace(); // start of an empty document
        editor.delete_forward(); // end of it
        assert!(!editor.undo());
    }

    #[test]
    fn history_stays_within_its_limit() {
        let mut editor = Editor::new();
        // Alternate the kinds of edit so nothing merges into one step.
        for _ in 0..(UNDO_LIMIT + 50) {
            editor.insert_char('x');
            editor.insert_newline();
        }
        let mut steps = 0;
        while editor.undo() {
            steps += 1;
        }
        assert!(steps > 0);
        assert!(steps <= UNDO_LIMIT);
        // The oldest steps were dropped, so the document cannot be rewound to
        // its empty starting state.
        assert!(!editor.get_content().is_empty());
    }

    #[test]
    fn save_and_undo_agree_on_what_is_modified() {
        let mut editor = Editor::new();
        for c in "hello".chars() {
            editor.insert_char(c);
        }
        editor.mark_saved();
        assert!(!editor.is_modified());

        editor.insert_char('!');
        assert!(editor.is_modified());

        // Undo steps back one edit, not past the save.
        assert!(editor.undo());
        assert_eq!(editor.get_content(), "hello");
        assert!(!editor.is_modified()); // exactly what is on disk
        assert!(editor.undo());
        assert_eq!(editor.get_content(), "");
        assert!(editor.is_modified());
        assert!(editor.redo());
        assert_eq!(editor.get_content(), "hello");
        assert!(!editor.is_modified());
    }

    #[test]
    fn an_untitled_note_stays_modified_when_rewound() {
        let mut editor = Editor::new();
        editor.mark_unsaved();
        editor.insert_char('a');
        assert!(editor.undo());
        assert_eq!(editor.get_content(), "");
        // Nothing was ever written, so an empty buffer is still unsaved work.
        assert!(editor.is_modified());
    }

    #[test]
    fn undo_boundary_starts_a_new_step() {
        let mut editor = Editor::new();
        editor.insert_char('a');
        editor.insert_char('b');
        assert!(editor.undo());
        assert_eq!(editor.get_content(), "");
        editor.insert_char('c');
        // The fresh edit is its own step, so it can be undone on its own.
        assert!(editor.undo());
        assert_eq!(editor.get_content(), "");
        assert!(!editor.undo());
    }

    #[test]
    fn visible_rows_stop_at_the_screenful() {
        // Width 4 hard-breaks "three" into "thre" + "e", giving 4 rows.
        let mut editor = Editor::new();
        editor.set_content("one two three");
        assert_eq!(editor.visible_rows(3, 4).len(), 3);
        assert_eq!(editor.visible_rows(99, 4).len(), 4);
    }
}
