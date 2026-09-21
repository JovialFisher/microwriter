<img src="https://iili.io/C8z6gG2.png" width="1000">

`microwriter` is meant to feel closer to sitting in front of a typewriter than
using a modern text editor. It clears the screen, hides the chrome, and
gets out of the way of writing.

## menu

The startup menu is the home of `microwriter`; every option is one keypress away.

| Shortcut | Menu item    | Description |
|----------|--------------|-------------|
| `n`      | new note     | Create a new text file in your default folder and start writing. |
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

- **Editor** — the writing surface; line numbers, wrap, and an optional status line.
- **Focus mode** — the editor with every decoration stripped (`Ctrl+P` → *focus mode*).
- **Recovery prompt** — appears on startup if the previous run left a file open.
- **Goals** — word / character / reading-time statistics (`Ctrl+P` → *goals*).

## install

`microwriter` is a single Rust binary. Pick whichever path suits you — both
produce the same `microwriter` / `microwriter.exe` executable.

### prebuilt binaries (easiest)

Open the [Releases](../../releases) page, grab the archive that matches
your platform, and unpack it. The `continuous build (…)` entry at the top
reflects the latest commit on `main` and is rebuilt on every push; pick a
`vX.Y.Z` tag entry for a frozen build.

| Platform                | Archive                                                                         | Run it |
|-------------------------|---------------------------------------------------------------------------------|---|
| Linux (x86_64)          | `microwriter-linux-x86_64-<date>-<shortsha>.tar.gz`                                   | `tar -xzf … && ./microwriter-linux-x86_64-<date>-<shortsha>/microwriter` |
| macOS Intel             | `microwriter-macos-x86_64-<date>-<shortsha>.tar.gz`                                    | `tar -xzf … && ./microwriter-macos-x86_64-<date>-<shortsha>/microwriter` |
| macOS Apple Silicon     | `microwriter-macos-aarch64-<date>-<shortsha>.tar.gz`                                   | `tar -xzf … && ./microwriter-macos-aarch64-<date>-<shortsha>/microwriter` |
| Windows (x86_64)        | `microwriter-windows-x86_64-<date>-<shortsha>.zip`                                     | unzip, then double-click `microwriter.exe` |

To verify a download, fetch `checksums.txt` from the same release and run:

```bash
# Linux / macOS
tar -xzf microwriter-linux-x86_64-<date>-<shortsha>.tar.gz
sha256sum -c checksums.txt
# Windows (PowerShell)
Expand-Archive .\microwriter-windows-x86_64-<date>-<shortsha>.zip
Get-FileHash .\microwriter-windows-x86_64-<date>-<shortsha>\microwriter.exe
```

Or move the unpacked binary onto your `PATH` — see [below](#putting-the-binary-on-path).

> **SmartScreen / Gatekeeper note.** When the maintainer has wired up the
> signing secrets (see [maintainers](#maintainers)), the Windows and macOS
> binaries are signed *and* the macOS one is notarized — double-clicking
> runs them without prompts. Without signing configured, Windows shows a
> SmartScreen warning and macOS blocks the binary on first launch; either
> upgrade to a tagged release (which the maintainer signs by hand) or build
> from source.

### quick install (from the git repo)

```bash
cargo install --git <repository-url> --locked --release
```

Builds and drops `microwriter` into `~/.cargo/bin/` (or
`%USERPROFILE%\.cargo\bin\` on Windows), which is already on `PATH` for most
Rust installations.

### build from source

```bash
git clone <repository-url>
cd microwriter
cargo build --release
```

The binary lands at `target/release/microwriter` (`microwriter.exe` on Windows).

### double-click launchers

After cloning or building the project, use the launcher for your platform from the repository root:

| Platform | Launcher |
|---|---|
| Windows | `launch_microwriter.bat` |
| macOS | `launch_microwriter.command` |
| Linux / other Unix systems | `launch_microwriter.sh` |

The launchers use an existing release or debug binary when available, otherwise they build and run the release binary with Cargo. On macOS/Linux, make the Unix launcher executable once if your file manager does not run it directly:

```bash
chmod +x launch_microwriter.sh launch_microwriter.command
```

### requirements

- **Rust 1.70 or newer** (the `[package]` declares `edition = "2021"`).
- A terminal that supports the alternate screen buffer; truecolor is
  optional but recommended.

### putting the binary on `PATH`

If you used `cargo install`, nothing to do. If you built from source:

| Platform | Move it to |
|---|---|
| Linux / macOS | `~/.local/bin/` (XDG default) or `/usr/local/bin/` for system-wide |
| Windows       | `%USERPROFILE%\.cargo\bin\` |

```bash
# Linux / macOS — single-user
install -m 0755 target/release/microwriter ~/.local/bin/

# Linux — system-wide
sudo install -m 0755 target/release/microwriter /usr/local/bin/

# Windows (PowerShell)
Move-Item .\target\release\microwriter.exe $env:USERPROFILE\.cargo\bin\
```

### uninstalling

Delete the binary and the user data directories listed under
[configuration](#configuration). `microwriter` writes no files outside of those paths.

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
| `theme`                | `"dark"`     | `dark` / `light` / `paper` *(same palette as `light`)* / `amber terminal` / `green phosphor` / `nord` / `solarized dark` |
| `cursor_style`         | `"block"`    | `block` / `beam` / `underline` |
| `line_numbers`         | `"off"`      | `off` / `relative` / `absolute` |
| `wrap`                 | `true`       | `true` / `false` |
| `autosave`             | `"disabled"` | literal string `disabled` / `"15 sec"` / `"30 sec"` / `"1 min"` / `"5 min"` |
| `default_folder`       | `""`         | an absolute path; falls back to `Documents` if empty (`~` is **not** expanded by `microwriter`) |
| `timestamp_filenames`  | `false`      | `true` / `false` (when on, new notes are named `YYYY-MM-DD-HH-MM.txt`) |
| `use_tabs`             | `false`      | `true` / `false` |
| `tab_spaces`           | `4`          | integer — width when `use_tabs` is `false` |
| `show_status`          | `false`      | `true` / `false` (also forced on inside **Goals**) |
| `startup_behavior`     | `"menu"`     | declared in config but currently **unused** (mapped for future work) |

### example

```toml
theme = "nord"
autosave = "1 min"
default_folder = "/absolute/path/to/notes"
```

### themes

The settings UI exposes `dark`, `light`, and `terminal default` (currently a
synonym for `dark`). The other four themes work in the engine but must be
set by editing `config.toml`. All themes are deliberately monochrome or
near-monochrome.

## supported file types

The browser and fuzzy search only index `.txt`, `.md`, `.rst`, and `.log`.
Other extensions are ignored by design — `microwriter` is for plain text.

## writing workflow

- **Save** (`Ctrl+S`) writes to `<path>.tmp` and renames it over the target
  — atomic; a crash mid-save can't corrupt the document. A small `saved`
  indicator flashes bottom-right for two seconds.
- **Autosave** is set in **Settings > autosave** (15 s / 30 s / 1 min /
  5 min). It only ticks while the buffer is modified and only inside the
  editor or focus mode.
- **Fuzzy search** uses position + consecutive-character + word-boundary
  scoring (`src/states/search.rs::fuzzy_match`) with a +10 boost for any
  file you've opened recently.
- **Recovery** records every file you open or create. If the file still
  exists next startup, `microwriter` offers to reopen it. `Esc` only hides the
  prompt for that run; pick **no** to permanently skip.
- **Goals** (`Ctrl+P` → *goals*) shows word count, character count, and a
  rough reading-time estimate (`max(1, words / 200)` minutes). No streaks,
  no achievements, no network sync — deliberately so.

## maintainers

### code-signing and notarization

The release workflow `.github/workflows/release.yml` will code-sign Windows
binaries and codesign + notarize macOS binaries **when the relevant GitHub
secrets are set**. If any are missing, those steps are skipped and the
workflow emits a `::warning::` annotation; unsigned binaries still publish,
so CI stays green from day one.

#### required GitHub secrets

Set these under `Settings > Secrets and variables > Actions`:

| Secret                | What it is |
|-----------------------|---|
| `WINDOWS_CERT_B64`    | Base64 of your Authenticode `.pfx`. Linux/GNU: `base64 -w0 microwriter.pfx`. macOS/BSD: `base64 -i microwriter.pfx | tr -d "\n"`. |
| `WINDOWS_CERT_PASS`   | Password for that `.pfx`. |
| `APPLE_CERT_B64`      | Base64 of your **Developer ID Application** `.p12`. Linux/GNU: `base64 -w0 microwriter.p12`. macOS/BSD: `base64 -i microwriter.p12 | tr -d "\n"`. Plain "Apple Development" certs do *not* sign for Gatekeeper. |
| `APPLE_CERT_PASS`     | Password for that `.p12`. |
| `APPLE_ID`            | Apple ID email tied to the Developer Program enrollment. |
| `APPLE_APP_PASS`      | **App-specific password** from appleid.apple.com — *not* your main Apple ID password. |
| `APPLE_TEAM_ID`       | The 10-character Team ID from the Apple Developer portal. |

#### honest caveats

- **OV code-signing certs** don't bypass SmartScreen instantly; reputation
  builds over weeks of downloads. **EV certs** do, but require a hardware
  token, which is awkward in CI.
- `xcrun notarytool submit --wait` adds **2–10 minutes** per macOS build to
  the CI run (Apple's queueing is variable).
- macOS notarization is performed on the packaged `.tar.gz` archive. Gatekeeper
  verifies the ticket on first launch (online). To make offline verification
  work after extraction, run `xcrun stapler staple /path/to/microwriter` manually.

## project structure

```
src/
├── main.rs     # terminal setup, ~60 fps event loop
├── app.rs      # top-level state, mode dispatch, keymap
├── editor.rs   # UTF-8 buffer, cursor, scroll
├── ui.rs       # ratatui rendering for every mode
├── config.rs   # config.toml (serde + toml)
├── storage.rs  # storage.json — recents + last session
└── states/
    ├── menu.rs
    ├── browser.rs
    ├── search.rs   # fuzzy_match + search state
    ├── palette.rs
    └── settings.rs
```

Build with `cargo build`, run with `cargo run`, release with
`cargo build --release` (lto, codegen-units = 1, strip), clean with
`cargo clean`.

