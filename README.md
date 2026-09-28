<img src="https://iili.io/C8z6gG2.png" width="1000">

`microwriter` is meant to feel closer to sitting in front of a typewriter than
using a modern text editor. It clears the screen, hides the chrome, and
gets out of the way of writing.

## menu

The startup menu is the home of `microwriter`; every option is one keypress away.

| Shortcut | Menu item    | Description |
|----------|--------------|-------------|
| `n`      | new note     | Pick a folder (defaults to your default folder), then start writing. |
| `o`      | open note    | Browse `.txt`, `.md`, `.rst`, and `.log` files locally. |
| `r`      | recent notes | Recently opened files grouped by *today / yesterday / this week / older*. |
| `f`      | search       | Fuzzy filename search across your default folder. |
| `s`      | settings     | Edit appearance, autosave, tabs, and other preferences. |
| `h`      | help         | Show the keymap reference. |
| `q`      | exit         | Save and quit. |

You can also jump straight to **new note** (`Ctrl+N`), **open note**
(`Ctrl+O`), and **search** (`Ctrl+F`) from anywhere. The command palette
(`Ctrl+P`) is the catch-all launcher.

Beyond the menu, `microwriter` has four more screens:

- **Editor** — the writing surface; line numbers, wrap, text alignment, and an optional status line.
- **Focus mode** — the editor with every decoration stripped (`Ctrl+P` → *focus mode*).
- **Recovery prompt** — appears on startup if the previous run left a file open.
- **Goals** — word / character / reading-time statistics, this run's writing time, and today's progress (`Ctrl+P` → *goals*).

## launch

Run Microwriter from the repository root with the launcher for your platform:

| Platform | Launcher |
|---|---|
| Windows | `launch_microwriter.bat` |
| macOS | `launch_microwriter.command` |
| Linux / other Unix systems | `launch_microwriter.sh` |

On macOS and Linux, make the Unix launchers executable once if your file manager does not run them directly:

```bash
chmod +x launch_microwriter.sh launch_microwriter.command
```

The launchers handle starting the app and keep the working directory anchored to the repository root.

## terminal commands

If you prefer the terminal, run these commands from the repository root:

```bash
cargo run
cargo check
cargo test
```

## requirements

- **Rust 1.70 or newer** (the `[package]` declares `edition = "2021"`).
- A terminal that supports the alternate screen buffer; truecolor is
  optional but recommended.

## quick start

1. Run `microwriter`.
2. Pick something from the menu with arrow keys (or `j` / `k`), or press the shortcut letter.
3. The selected action runs immediately — no confirmation dialogs.
4. Inside the editor: `Ctrl+S` saves, `Esc` returns to the menu, `Ctrl+Q`
   saves and quits.

## keybindings

### global

| Key      | Action |
|----------|--------|
| `Ctrl+N` | new note |
| `Ctrl+O` | open file browser |
| `Ctrl+F` | open search |
| `Ctrl+P` | command palette |
| `Ctrl+S` | save the current document |
| `Ctrl+Q` | save and quit |
| `Ctrl+C` | save and quit |
| `Esc`    | back to menu (in **focus mode** → editor; in **file browser** → up a directory) |

### editor

| Key                        | Action |
|----------------------------|--------|
| Arrow keys                 | move cursor |
| `Home` / `End`             | line start / line end |
| `Ctrl+Home` / `Ctrl+End`   | document start / end |
| `Ctrl+←` / `Ctrl+→`        | jump words |
| `Ctrl+↑` / `Ctrl+↓`        | scroll 3 lines without moving the caret |
| `PageUp` / `PageDown`      | page (20 lines) |
| `Enter`                    | newline |
| `Backspace` / `Delete`     | delete |
| `Tab`                      | tab, or *N* spaces depending on *tabs/spaces* |
| `Ctrl+L`                   | cycle text alignment: left → center → right → justified |
| `Ctrl+Z` / `Ctrl+Y`        | undo / redo |

### file browser

| Key                          | Action |
|------------------------------|--------|
| `↑` / `↓` (or `k` / `j`)     | move |
| `Enter`                      | open / descend |
| `Esc` / `Backspace`          | go up a directory |
| `h`                          | go to `$HOME` |
| `/`                          | start an inline filter |

### search / command palette

Both share the same shape — typing instantly filters the list.

| Key                          | Action |
|------------------------------|--------|
| type                         | instant filter |
| `↑` / `↓` (or `k` / `j`)     | move |
| `Enter`                      | confirm / open |
| `Esc`                        | cancel |

### recent notes

A flat list grouped by age — no inline filter.

| Key                          | Action |
|------------------------------|--------|
| `↑` / `↓` (or `k` / `j`)     | move between entries |
| `Enter`                      | open the selected note |
| `Esc`                        | back to menu |

### settings

| Key                          | Action |
|------------------------------|--------|
| `↑` / `↓` (or `k` / `j`)     | move between categories |
| `Enter` / `→`                | edit current category |
| `↑` / `↓` (while editing)    | choose a value |
| `Enter`                      | confirm |
| `←`                          | cancel edit |
| `Esc`                        | save and exit |

### recovery prompt

| Key                          | Action |
|------------------------------|--------|
| `↑` / `↓` (or `k` / `j`)     | toggle *yes* / *no* |
| `Enter`                      | confirm |
| `Esc`                        | dismiss for this run (prompt returns next launch) |

## configuration

A single readable `config.toml`; any field is optional and falls back to
its default.

### file locations

Resolved with the [`dirs`](https://crates.io/crates/dirs) crate.

| Platform | Settings | Storage |
|---|---|---|
| Linux   | `~/.config/microwriter/config.toml`                       | `~/.local/share/microwriter/storage.json` |
| macOS   | `~/Library/Application Support/microwriter/config.toml`  | `~/Library/Application Support/microwriter/storage.json` |
| Windows | `%APPDATA%\microwriter\config.toml`                       | `%APPDATA%\microwriter\storage.json` |

If `default_folder` is empty on the first launch, `microwriter` falls back to your
platform's `Documents` directory.

### reference

| Key                    | Default      | Allowed |
|------------------------|--------------|---------|
| `theme`                | `"dark"`     | `dark` / `light` / `paper` *(same palette as `light`)* / `amber terminal` / `green phosphor` / `nord` / `solarized dark` / `terminal default` *(synonym for `dark`)* |
| `cursor_style`         | `"block"`    | `block` / `beam` / `underline` |
| `line_numbers`         | `"off"`      | `off` / `relative` / `absolute` |
| `wrap`                 | `true`       | `true` / `false` |
| `autosave`             | `"disabled"` | literal string `disabled` / `"15 sec"` / `"30 sec"` / `"1 min"` / `"5 min"` |
| `default_folder`       | `""`         | an absolute path; falls back to `Documents` if empty (`~` is **not** expanded by `microwriter`) |
| `timestamp_filenames`  | `false`      | `true` / `false` (when on, new notes are named `YYYY-MM-DD-HH-MM.txt`) |
| `use_tabs`             | `false`      | `true` / `false` |
| `tab_spaces`           | `4`          | integer — width when `use_tabs` is `false` |
| `show_status`          | `false`      | `true` / `false` (also forced on inside **Goals**) |
| `alignment`            | `"left"`     | `left` / `center` / `right` / `justified` (display only — the file keeps its raw text) |
| `startup_behavior`     | `"menu"`     | declared in config but currently **unused** (mapped for future work) |

### example

```toml
theme = "nord"
autosave = "1 min"
default_folder = "/absolute/path/to/notes"
```

### themes

The settings UI exposes every theme — `dark`, `light`, `paper`, `amber
terminal`, `green phosphor`, `nord`, `solarized dark`, and `terminal default`
(still a synonym for `dark`). `paper` shares `light`'s palette by design. All
themes are deliberately monochrome or near-monochrome.

## supported file types

The browser and fuzzy search only index `.txt`, `.md`, `.rst`, and `.log`.
Other extensions are ignored by design — `microwriter` is for plain text.

## writing workflow

- **Save** (`Ctrl+S`) writes to `<path>.tmp` and renames it over the target
  — atomic; a crash mid-save can't corrupt the document. A small `saved`
  indicator flashes bottom-right for two seconds.
- **Undo / redo** (`Ctrl+Z` / `Ctrl+Y`) walks a bounded history: at most 500
  steps, and at most four million bytes of text across all of them. A run of
  typing or of deleting collapses into a single step, so undo removes a word
  rather than a letter — but moving the caret, pressing Enter, or accepting a
  completion starts a fresh step. The history is dropped when another file is
  opened. Each step carries a revision, so undoing back to the text you last
  saved clears the *modified* indicator and stops autosave, exactly as if you
  had just saved.
- **Autosave** is set in **Settings > autosave** (15 s / 30 s / 1 min /
  5 min). It only ticks while the buffer is modified and only inside the
  editor or focus mode.
- **Word wrap** (`wrap`, *Settings > word wrap*, or `Ctrl+P` → *toggle
  wrap*) is on by default. Long paragraphs soft-wrap at the right margin,
  breaking after a space so words stay whole; a word wider than the screen is
  hard-broken rather than hidden. Continuation rows leave the line-number
  gutter blank. With `wrap = false` a logical line stays one row and the view
  slides sideways to follow the caret.
- **Alignment** (`Ctrl+L`, *Settings > alignment*, or `Ctrl+P` → *align
  …*) cycles **left → center → right → justified** while you write. It is a
  view setting only: the saved file always keeps plain, unaligned text, and
  the choice is remembered in `config.toml`. Justified paragraphs widen the
  spaces between words so every wrapped row but the last one is flush with
  both margins.
- **Fuzzy search** uses position + consecutive-character + word-boundary
  scoring (`src/states/search.rs::fuzzy_match`) with a +10 boost for any
  file you've opened recently.
- **Recovery** records every file you open or create. If the file still
  exists next startup, `microwriter` offers to reopen it. `Esc` only hides the
  prompt for that run; pick **no** to permanently skip.
- **Goals** (`Ctrl+P` → *goals*) shows word count, character count, and a
  rough reading-time estimate (`max(1, words / 200)` minutes), followed by
  *recent days* — the last seven days of writing, newest first, labelled
  `today` / `yesterday` / `MM-DD`.
- **Session tracking** runs quietly behind the goals screen. *Session time*
  counts only the minutes a document is open — sitting on the menu is not
  writing — and each day records the words added, the minutes written, and
  the documents touched, with today's row also showing the document count.
  Thirty days are kept in `storage.json` under a local date, flushed every
  20 seconds while you write so a crash costs seconds rather than the whole
  day. No streaks, no achievements, no network sync — deliberately so.
- **Export** (`Ctrl+P` → *export html*) writes a standalone HTML page beside
  the note and named after it (`chapter.txt` → `chapter.html`). The text is
  kept verbatim and escaped, so the page opens cleanly in a browser and can
  be shared or printed to PDF from there — no rendering engine inside
  `microwriter`.

## project structure

```
src/
├── main.rs     # terminal setup, ~60 fps event loop
├── app.rs      # top-level state, mode dispatch, keymap
├── editor.rs   # UTF-8 buffer, cursor, scroll
├── ui.rs       # ratatui rendering for every mode
├── config.rs   # config.toml (serde + toml)
├── storage.rs  # storage.json — recents, last session, daily statistics
├── export.rs   # note → standalone HTML
└── states/
    ├── menu.rs
    ├── browser.rs
    ├── search.rs   # fuzzy_match + search state
    ├── palette.rs
    └── settings.rs
```

Run `cargo run` from the repository root when you prefer terminal commands over the platform launcher.

