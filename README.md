<img src="https://iili.io/C8z6gG2.png" width="1000">

`microwriter` is meant to feel closer to sitting in front of a typewriter than
using a modern text editor. It clears the screen, hides the chrome, and
gets out of the way of writing.

## menu

The startup menu is the home of `microwriter`; every option is one keypress away.

| Shortcut | Menu item    | Description |
|----------|--------------|-------------|
| `n`      | new note     | Pick a folder (defaults to your default folder), then start writing. Press `d` to pick another drive. |
| `o`      | open note    | Browse `.txt`, `.md`, `.rst`, and `.log` files locally. Press `d` to jump to another drive. |
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
- **Recovery prompt** — appears on startup when the previous run ended with unsaved changes, offering to bring the text back.
- **Goals** — word / character / reading-time statistics, this run's writing time, and today's progress (`Ctrl+P` → *goals*).

## launch

Double-click the launcher for your platform. It checks whether Microwriter is
already built and, on the first run, builds it for you.

| Platform | Launcher |
|---|---|
| Windows | `launch_microwriter.bat` |
| macOS | `launch_microwriter.command` |
| Linux / other Unix systems | `launch_microwriter.sh` |

On macOS and Linux, make the Unix launchers executable once if your file manager does not run them directly:

```bash
chmod +x launch_microwriter.sh launch_microwriter.command
```

What the launcher does, in plain language:

- **Already built** — starts the app immediately; no build, no waiting. A
  finished build leaves a marker beside the app, so the launcher can tell a
  complete build from a broken one.
- **Not built yet** — explains that the one-time setup takes a minute or two,
  then builds with the compiler output redirected to `target/build.log`. While
  it works, one line shows a spinner, a real percentage, a bar, and the elapsed
  time: `cargo metadata` reports how many crates this machine needs, the
  launcher counts the `Compiling …` lines cargo has finished, and the last
  stretch reads `Finishing up` while the app is linked. The spinner is braille
  on a UTF-8 terminal and the bar is made of blocks; a console that cannot show
  those glyphs gets plain ASCII instead. If the crate count cannot be worked out
  the line shows crates done instead of a percentage, and a piped or headless
  run gets one plain line per milestone rather than a line that rewrites itself.
  Windows draws the same progress line from `scripts/build_progress.ps1`, and
  falls back to a silent build if PowerShell is unavailable.
- **Rust is missing** — points at <https://rustup.rs> and exits, instead of
  printing shell errors.
- **Setup was interrupted or failed** — a build stopped part-way (the window
  closed, Ctrl+C, or an error) can leave a broken half-built `target/` folder
  behind. The launcher notices the missing completion marker, closes its
  window, deletes the broken build folder, and opens a fresh window so the
  build starts again from scratch.
- **Build fails** — if the fresh build still does not finish, the launcher says
  so plainly and points at `target/build.log` rather than dumping compiler
  errors on screen. It restarts this way only once, so a genuine problem cannot
  loop forever.

The launchers keep the working directory anchored to the repository root, so
notes and settings always resolve to the same place.

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

### menu

The startup screen. Every item is one keypress away.

| Key                          | Action |
|------------------------------|--------|
| `↑` / `↓` (or `k` / `j`)     | move between items |
| `Enter`                      | run the selected item |
| `n`                          | new note |
| `o`                          | open note |
| `r`                          | recent notes |
| `f`                          | search |
| `s`                          | settings |
| `h`                          | help |
| `q` / `Esc`                  | save and quit |

### global

Available from every screen.

| Key      | Action |
|----------|--------|
| `F1`     | help — press again to close |
| `Ctrl+N` | new note |
| `Ctrl+O` | open file browser |
| `Ctrl+R` | recent notes |
| `Ctrl+F` | open search |
| `Ctrl+G` | writing statistics |
| `Ctrl+P` | command palette |
| `Ctrl+S` | save the current document (**editor** / **focus mode** only) |
| `Ctrl+Q` | save and quit |
| `Ctrl+C` | save and quit |

### editor

| Key                        | Action |
|----------------------------|--------|
| Arrow keys                 | move cursor |
| `Home` / `End`             | line start / line end |
| `Ctrl+Home` / `Ctrl+End`   | document start / end |
| `Ctrl+←` / `Ctrl+→`        | jump words |
| `Ctrl+↑` / `Ctrl+↓`        | scroll 3 lines without moving the caret |
| `PageUp` / `PageDown`      | page up / down |
| `Enter`                    | newline — or accept the visible suggestion |
| `Backspace` / `Delete`     | delete |
| `Tab`                      | accept the suggestion, else complete the word, else tab / *N* spaces |
| `Ctrl+Space`               | complete the word; press again to cycle through candidates |
| `→`                        | accept the visible suggestion, else move right |
| `Ctrl+L`                   | cycle text alignment: left → center → right → justified |
| `Ctrl+Z` / `Ctrl+Y`        | undo / redo |

### focus mode

The editor with the chrome hidden — every editor key works; `Esc` or `F1` leaves.

| Key           | Action |
|---------------|--------|
| `Esc` / `F1`  | leave focus mode |
| anything else | the matching editor key |

### new note (folder picker)

Choose the folder a new note is created in.

| Key                          | Action |
|------------------------------|--------|
| `↑` / `↓` (or `k` / `j`)     | move between entries |
| `Enter`                      | create the note in this folder / descend into the highlighted folder |
| `Backspace`                  | go up a directory |
| `h`                          | jump to `$HOME` |
| `d`                          | list drives — the system drive or any external / mounted volume |
| `/`                          | start an inline filter |
| `Esc`                        | back to the menu |

While filtering (after `/`): type to narrow the list, `Tab` completes to the highlighted entry, `Backspace` deletes, `Enter` leaves the filter, `Esc` clears it. While the drive list is open: `↑` / `↓` choose a drive, `Enter` opens it, `Esc` or `q` closes it.

### file browser

| Key                          | Action |
|------------------------------|--------|
| `↑` / `↓` (or `k` / `j`)     | move |
| `Enter`                      | open / descend |
| `Esc` / `Backspace`          | go up a directory (back to the menu at the root) |
| `h`                          | go to `$HOME` |
| `d`                          | list drives — jump to the system drive or any external / mounted volume |
| `/`                          | start an inline filter |

Only `.txt`, `.md`, `.rst`, and `.log` files are listed. While filtering (after `/`): type to narrow the list, `Tab` completes to the first match, `Backspace` deletes, `Enter` leaves the filter, `Esc` clears it. While the drive list is open: `↑` / `↓` choose a drive, `Enter` opens it, `Esc` or `q` closes it.

### search

Fuzzy filename search across your default folder.

| Key                          | Action |
|------------------------------|--------|
| type                         | instant filter |
| `↑` / `↓` (or `k` / `j`)     | move |
| `Tab`                        | complete the query to the top result |
| `Enter`                      | open the selected note |
| `Esc`                        | cancel |

### command palette

The catch-all launcher: the same shape as search, over the built-in command list.

| Key                          | Action |
|------------------------------|--------|
| type                         | instant filter |
| `↑` / `↓` (or `k` / `j`)     | move |
| `Tab`                        | complete the query to the top command |
| `Enter`                      | run the selected command |
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
| `↑` / `↓` (or `k` / `j`)     | move between categories (wraps around) |
| `Enter` / `→`                | edit current category |
| `↑` / `↓` (while editing)    | choose a value (wraps around) |
| `Enter`                      | confirm |
| `←`                          | cancel edit |
| `Esc`                        | save and exit |

### help

A concise keymap reference, not a place for prose. Reachable from the menu
(`h`) or from anywhere with `F1`.

| Key      | Action |
|----------|--------|
| `F1`     | close help |
| any other key | back to the menu |

### goals

| Key      | Action |
|----------|--------|
| `Esc`    | back to the menu |

### recovery prompt

Shown at startup only when the last run ended without saving — a crash, a
closed terminal, or a lost battery.

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
| `typewriter_scroll`    | `false`      | `true` / `false` (hold the caret mid-screen and scroll the text up as you write) |
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

## drives

The file browser and the new-note folder picker both detect the volumes the
operating system has mounted, not just the folder `microwriter` starts in.
Press `d` for a list and `Enter` to jump to one. On Windows this is every
existing drive letter (A: and B: are skipped so a floppy controller is never
asked for a disk), labelled by kind — `E:\ (removable)`, `D:\ (fixed)`,
`Z:\ (network)`. On Linux and macOS it looks under `/media`, `/run/media`,
`/mnt`, and `/Volumes` (including the per-user subfolders a desktop mounts a
USB stick into). Nothing is mounted and no media is spun up by listing, so an
empty card reader stays quiet; a root that turns out to be unreadable simply
shows an empty list.

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
- **Completion** shows the word you are typing as a dim *ghost* after the
  caret; `Tab` or `Enter` accepts it. Prefix lookup runs on a weighted trie
  from the [`autocomplete`](https://crates.io/crates/autocomplete) crate, fed
  from three lexicons at once: the words already in the open document (ranked
  highest), the words in your other recent notes, and a bundled vocabulary of
  common English words (ranked lowest). A brand new note therefore completes a
  word you have never typed, not just one you already used. `Ctrl+Space`
  completes the word outright, and each press after that cycles to the next
  candidate. Suggestions keep the capitalisation you began with (`Rec` →
  `Receive`) and keep an unusual spelling (`kub` → `Kubernetes`), and a very
  short prefix offers at most 40 candidates so cycling stays quick.
- **Word wrap** (`wrap`, *Settings > word wrap*, or `Ctrl+P` → *toggle
  wrap*) is on by default. Long paragraphs soft-wrap at the right margin,
  breaking after a space so words stay whole; a word wider than the screen is
  hard-broken rather than hidden. Continuation rows leave the line-number
  gutter blank. With `wrap = false` a logical line stays one row and the view
  slides sideways to follow the caret.
- **Typewriter scrolling** (*Settings > typewriter scroll*, or `Ctrl+P` →
  *toggle typewriter scroll*) holds the caret around the middle of the screen.
  As you write past that point the text scrolls up and the caret is pulled
  back to the middle, so you always keep several lines of what you just wrote
  in view instead of running along the bottom edge. Off by default; the plain
  behavior only scrolls when the caret would otherwise leave the screen.
- **Alignment** (`Ctrl+L`, *Settings > alignment*, or `Ctrl+P` → *align
  …*) cycles **left → center → right → justified** while you write. It is a
  view setting only: the saved file always keeps plain, unaligned text, and
  the choice is remembered in `config.toml`. Justified paragraphs widen the
  spaces between words so every soft-wrapped row but the last one is flush with
  both margins. A paragraph ends where you press Enter, so a line you end
  yourself always keeps its natural length instead of being stretched across
  the screen.
- **Fuzzy search** uses position + consecutive-character + word-boundary
  scoring (`src/states/search.rs::fuzzy_match`) with a +10 boost for any
  file you've opened recently.
- **Recovery** keeps a crash draft of the open buffer beside
  `storage.json`, rewritten every few seconds while there are unsaved changes
  (independent of the `autosave` setting). If the app dies before you save,
  the next launch offers to bring that text back — the file on disk may never
  have been written, so the recovered buffer is restored as *modified*. The
  draft is cleared on a normal save or quit, so a clean run never prompts.
  `Esc` only hides the prompt for that run; pick **no** to discard the draft
  permanently.
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
├── editor.rs   # UTF-8 buffer, cursor, scroll, completion
├── dictionary.rs # weighted completion trie over the bundled vocabulary
├── ui.rs       # ratatui rendering for every mode
├── config.rs   # config.toml (serde + toml)
├── storage.rs  # storage.json — recents, daily statistics; session.draft.json — crash draft
├── export.rs   # note → standalone HTML
└── states/
    ├── menu.rs
    ├── browser.rs
    ├── search.rs   # fuzzy_match + search state
    ├── palette.rs
    └── settings.rs
```

The bundled word list lives in `src/common_words.txt` — plain,
whitespace-separated text compiled into the binary, so completion works
offline and no dictionary file has to be installed or downloaded. Prefix
search and ranking are handled by the `autocomplete` crate's weighted trie
(`src/dictionary.rs`), which folds the document, recent notes, and this list
into a single index.

Run `cargo run` from the repository root when you prefer terminal commands over the platform launcher.

