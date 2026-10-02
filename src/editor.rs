use std::cell::Cell;
use std::cmp;
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
/// more of its paragraph wraps below it — the final row of a paragraph stays
/// ragged, exactly as in a printed book. A row that ends a line you typed
/// yourself is the last row of its paragraph, so it is never stretched: a
/// short line stays its natural length instead of being blown across the
/// screen.
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
    /// Whether another wrapped row of the same paragraph follows below, which
    /// makes this row a candidate for justification. Only soft wrapping can
    /// continue a paragraph, so the last row of every typed line is `false`.
    pub continues: bool,
    /// Char offset of the caret inside `start..end`, when the caret is here.
    pub caret: Option<usize>,
}

/// Most completion candidates kept for one prefix, so cycling through a very
/// short prefix stays a short walk instead of hundreds of words.
const AUTOCOMPLETE_LIMIT: usize = 40;
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
    /// Typewriter scrolling, pushed in by the renderer from `config`. Also a
    /// `Cell` for the same reason as the viewport.
    typewriter_scroll: Cell<bool>,
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
            typewriter_scroll: Cell::new(false),
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
        // Follow the caret as it types, so a new wrapped row can pull the view
        // along without waiting for the next cursor key.
        self.ensure_scroll();
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
        self.ensure_scroll();
    }

    // ─── Autocomplete ─────────────────────────────────────────

    /// Candidate completions for `prefix`, best candidate first.
    ///
    /// Every lexicon is fed into one weighted trie and looked up together, so
    /// the words already in this document outrank words from recent notes,
    /// which in turn outrank the bundled vocabulary. A brand new note still
    /// completes, because the bundled list needs nothing from the writer's
    /// history.
    fn completions_for(&self, prefix: &str) -> Vec<String> {
        if prefix.is_empty() {
            return Vec::new();
        }
        let mut completer = crate::dictionary::Completer::new();
        for word in &self.cross_file_words {
            completer.add(word, crate::dictionary::RECENT_WEIGHT);
        }
        for word in self.lines.iter().flat_map(|line| line.split_whitespace()) {
            completer.add(word, crate::dictionary::DOCUMENT_WEIGHT);
        }
        completer.complete(prefix, AUTOCOMPLETE_LIMIT)
    }

    /// Forget the current completion session.
    fn clear_completion(&mut self) {
        self.autocomplete_matches.clear();
        self.autocomplete_prefix.clear();
        self.autocomplete_index = 0;
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
        let prefix_len = prefix.chars().count();
        if let Some(matched) = self.completions_for(&prefix).into_iter().next() {
            self.ghost_suggestion = matched.chars().skip(prefix_len).collect();
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
        self.clear_completion();
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
    ///
    /// The first press completes the word with the best candidate; pressing it
    /// again walks the remaining candidates, replacing the completion each
    /// time, so a run of presses cycles through every match.
    pub fn try_autocomplete(&mut self) -> bool {
        let (word_start, word) = self.current_word_prefix();
        if word.is_empty() {
            self.clear_completion();
            return false;
        }

        // A press right after a completion is a request to cycle, not a fresh
        // search: the word now under the caret is the suggestion just applied.
        let cycling = self
            .autocomplete_matches
            .get(self.autocomplete_index)
            .is_some_and(|applied| applied.eq_ignore_ascii_case(&word));

        if cycling {
            self.autocomplete_index =
                (self.autocomplete_index + 1) % self.autocomplete_matches.len();
        } else {
            let matches = self.completions_for(&word);
            if matches.is_empty() {
                self.clear_completion();
                return false;
            }
            self.autocomplete_matches = matches;
            self.autocomplete_index = 0;
            self.autocomplete_prefix = word.clone();
        }

        let completed = self.autocomplete_matches[self.autocomplete_index].clone();
        self.replace_word(word_start, &completed);
        true
    }

    /// Replace the word ending at the caret with `completed`.
    fn replace_word(&mut self, word_start: usize, completed: &str) {
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
        line.insert_str(byte_start, completed);
        self.cursor_col = word_start + completed.chars().count();
        self.preferred_col = None;
        self.end_edit();
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
        self.clear_completion();
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
    ///
    /// Normally the view stays put until the caret would leave it. With
    /// typewriter scrolling on, the caret is instead held around the middle of
    /// the screen: writing at the bottom pulls the text up past the writer
    /// rather than letting the caret sink to the last row.
    fn ensure_scroll(&mut self) {
        let width = self.viewport_width.get();
        let height = self.viewport_height.get().max(1);
        let caret = self.cursor_visual_row(width);
        if self.typewriter_scroll.get() {
            let last_screen = self.total_visual_rows(width).saturating_sub(height);
            self.scroll_row = caret.saturating_sub(height / 2).min(last_screen);
            return;
        }
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

    /// Turn typewriter scrolling on or off; the renderer pushes this in from
    /// `config` each frame.
    pub fn set_typewriter_scroll(&self, on: bool) {
        self.typewriter_scroll.set(on);
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
            for (index, (start, end)) in segments.into_iter().enumerate() {
                if visual >= self.scroll_row {
                    let caret = (row == self.cursor_row && index == caret_segment)
                        .then(|| self.cursor_col.max(start) - start);
                    rows.push(RenderedRow {
                        row,
                        start,
                        end,
                        // A typed line ends a paragraph, so only the rows that
                        // soft wrapping pushed down continue it.
                        continues: index < last_segment,
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
    fn a_short_typed_line_is_never_stretched() {
        // The end of a soft-wrapped paragraph and a whole typed line both have
        // no row below them, so neither is a justification candidate.
        assert!(!should_justify("a short line here", 80, false));
        assert!(should_justify(
            "a short line in a wrapping paragraph",
            80,
            true
        ));
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
    fn only_wrapped_continuation_rows_are_justifiable() {
        let mut editor = Editor::new();
        editor.set_content("first line\nsecond line that wraps over several rows");
        let rows = editor.visible_rows(10, 15);
        let justifiable: Vec<bool> = rows.iter().map(|row| row.continues).collect();
        // `first line` is a whole paragraph, and `rows` ends the wrapped one.
        assert_eq!(justifiable, vec![false, true, true, true, false]);
    }

    #[test]
    fn visible_rows_stop_at_the_screenful() {
        // Width 4 hard-breaks "three" into "thre" + "e", giving 4 rows.
        let mut editor = Editor::new();
        editor.set_content("one two three");
        assert_eq!(editor.visible_rows(3, 4).len(), 3);
        assert_eq!(editor.visible_rows(99, 4).len(), 4);
    }

    #[test]
    fn completion_draws_on_built_in_words_not_just_the_document() {
        let mut editor = Editor::new();
        editor.set_content("rec");
        editor.move_line_end();
        editor.refresh_ghost();
        // Nothing in the buffer can complete "rec", so the bundled list must.
        assert!(!editor.ghost_suggestion.is_empty());
        assert!(editor
            .ghost_suggestion
            .chars()
            .all(|c| c.is_ascii_alphabetic()));
    }

    #[test]
    fn completion_prefers_a_word_already_in_the_document() {
        let mut editor = Editor::new();
        editor.set_content("serendipity\nseren");
        editor.cursor_row = 1;
        editor.cursor_col = 5;
        editor.refresh_ghost();
        assert_eq!(editor.ghost_suggestion, "dipity");
    }

    #[test]
    fn autocomplete_completes_the_word_under_the_caret() {
        let mut editor = Editor::new();
        editor.set_content("serendipity\nseren");
        editor.cursor_row = 1;
        editor.cursor_col = 5;
        assert!(editor.try_autocomplete());
        assert_eq!(editor.lines[1], "serendipity");
    }

    #[test]
    fn autocomplete_cycles_between_candidates() {
        let mut editor = Editor::new();
        editor.set_content("serenade\nserendipity\nseren");
        editor.cursor_row = 2;
        editor.cursor_col = 5;
        assert!(editor.try_autocomplete());
        let first = editor.lines[2].clone();
        assert!(editor.try_autocomplete());
        let second = editor.lines[2].clone();
        assert_ne!(first, second);
        assert!(first.starts_with("seren") && second.starts_with("seren"));
        assert_eq!(editor.autocomplete_matches.len(), 2);
    }

    #[test]
    fn completion_matches_the_prefix_capitalisation() {
        let mut editor = Editor::new();
        editor.set_content("Ther");
        editor.move_line_end();
        assert!(editor.try_autocomplete());
        assert!(editor.lines[0].starts_with("Ther"));
        assert!(editor.lines[0].chars().next().unwrap().is_uppercase());
        assert!(editor.lines[0].to_lowercase().starts_with("ther"));
    }

    #[test]
    fn completion_candidates_stay_bounded() {
        let mut editor = Editor::new();
        editor.set_content("a");
        editor.move_line_end();
        assert!(editor.try_autocomplete());
        assert!(!editor.autocomplete_matches.is_empty());
        assert!(editor.autocomplete_matches.len() <= AUTOCOMPLETE_LIMIT);
    }

    fn numbered_lines(count: usize) -> String {
        (0..count).map(|i| format!("line {}\n", i)).collect()
    }

    #[test]
    fn typewriter_scroll_holds_the_caret_mid_screen() {
        let mut editor = Editor::new();
        editor.set_content(&numbered_lines(30));
        editor.set_viewport(40, 9);
        editor.set_typewriter_scroll(true);
        editor.cursor_row = 20;
        editor.move_line_end(); // any move re-scrolls the view
        assert_eq!(editor.scroll_row, 20 - 9 / 2);
    }

    #[test]
    fn typewriter_scroll_is_off_by_default() {
        let mut editor = Editor::new();
        editor.set_content(&numbered_lines(30));
        editor.set_viewport(40, 9);
        editor.cursor_row = 20;
        editor.move_line_end();
        // Without typewriter scrolling the view only moves when it must, so the
        // caret sits on the bottom row rather than the middle.
        assert_eq!(editor.scroll_row, 20 + 1 - 9);
    }

    #[test]
    fn typing_follows_the_caret_into_new_wrapped_rows() {
        let mut editor = Editor::new();
        editor.set_viewport(4, 2);
        editor.set_typewriter_scroll(true);
        for _ in 0..20 {
            editor.insert_char('x');
        }
        // The caret wrapped far past the first screen; the view came along
        // instead of being stuck at the top.
        assert!(editor.scroll_row > 0);
    }
}
