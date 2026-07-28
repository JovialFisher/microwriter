# `assets/`

This directory holds `mute_icon.ico` — the icon that gets embedded into the
Windows `mute.exe` binary by `build.rs` (via the `winresource` crate).

## how to use it

1. Drop a Windows-compatible `.ico` file at this exact path:
   ```
   assets/mute_icon.ico
   ```
2. The next Windows build (locally or via the `release.yml` workflow) will
   embed it into `mute.exe`. It shows up in:
   - File Explorer's icon view
   - The Windows taskbar while `mute` runs
   - Start-menu shortcuts
   - Alt+Tab thumbnails

If the file is missing, the build still succeeds and emits a `cargo:warning`
so the omission is loud in CI logs — but the binary ships with the default
executable icon. That keeps the workflow green during development; switch
to fail-fast on missing-icon if you'd rather block CI when the icon is absent.

## what Windows expects in the `.ico`

For sharp rendering at every size Windows uses, pack **multiple resolutions**
into one `.ico` file:

- **16×16** — small icons, file lists
- **32×32** — standard icons, taskbar previews
- **48×48** — large icons, Alt-Tab
- **256×256** — HiDPI displays, AppIcon-style views

A single-image `.ico` (e.g. just 256×256) works but renders blurry at the
smaller sizes. Tools that produce multi-frame `.ico` files include
ImageMagick (`convert input.png -define icon:auto-resize=256,48,32,16
output.ico`) and the built-in Windows Icon Maker.

## platform scope

| Platform | In-binary icon? |
|---|---|
| **Windows** (`x86_64-pc-windows-msvc`) | **Yes** — embedded by `build.rs` from this file. |
| **Linux**   (`x86_64-unknown-linux-gnu`) | No — bare CLI binary; icon lives in the OS icon-theme database (out of scope for the release pipeline). |
| **macOS Intel** (`x86_64-apple-darwin`) | No — bare CLI binary; icon lives in a `.app` bundle's `Contents/Resources/AppIcon.icns` (out of scope for the release pipeline). |
| **macOS Apple Silicon** (`aarch64-apple-darwin`) | No — same as macOS Intel. |

If you ever ship a packaged `.app` for macOS or an AppImage/deb for Linux,
extend this CI to emit those artifacts with their respective icon formats.
