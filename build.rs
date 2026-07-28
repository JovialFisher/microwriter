//! `build.rs`
//!
//! Embed `assets/mute_icon.ico` into the Windows `mute.exe` so the binary
//! carries the custom icon visible in Explorer, the taskbar, and shortcut
//! menus.
//!
//! Notes:
//! - macOS and Linux ship bare binaries that don't host an in-binary icon.
//!   On macOS, icons live in a `.app` bundle (`Contents/Resources/AppIcon.icns`);
//!   on Linux, they're registered with the icon-theme database. Neither is
//!   in scope for this release pipeline (we ship `tar.gz` of the raw binary).
//! - If `assets/mute_icon.ico` is absent, the build still succeeds and we
//!   emit a `cargo:warning` line so the omission is loud in CI logs.

fn main() {
    // winresource handles every cross-target case, but it's pointless to
    // touch the binary on non-Windows builds — bail out before doing any
    // work so the build script runs in microseconds on Linux/macOS.
    let target_family = std::env::var("CARGO_CFG_TARGET_FAMILY").unwrap_or_default();
    if target_family != "windows" {
        return;
    }

    let icon = std::path::Path::new("assets/mute_icon.ico");

    if !icon.exists() {
        println!(
            "cargo:warning=assets/mute_icon.ico not found; \
             the Windows binary will ship without an embedded icon. \
             Drop a .ico file at assets/mute_icon.ico to enable it."
        );
        return;
    }

    let mut res = winresource::WindowsResource::new();
    res.set_icon(
        icon.to_str()
            .expect("assets/mute_icon.ico path is not valid UTF-8"),
    );
    // Version metadata — visible in Explorer's "Properties → Details".
    res.set("ProductName", "mute");
    res.set("FileDescription", "minimal user text environment");
    res.set("OriginalFilename", "mute.exe");
    res.set("InternalName", "mute");

    if let Err(e) = res.compile() {
        // Be loud: resource-compile failures usually mean the .ico is
        // malformed or wrong version (winresource embeds via `rc.exe`).
        eprintln!("Failed to compile Windows resource (.ico): {e}");
        std::process::exit(1);
    }
}
