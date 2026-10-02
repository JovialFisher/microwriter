//! Discovery of browsable volume roots — the system drive plus any external or
//! secondary volumes (USB sticks, external SSDs, network shares, …).
//!
//! Nothing here mounts anything; it only reports roots the operating system
//! already exposes so the file browser and folder picker can jump to them.

/// One browsable volume root.
#[derive(Debug, Clone)]
pub struct Drive {
    /// Friendly label shown in the drive list.
    pub name: String,
    /// Filesystem root the browser navigates to.
    pub path: String,
}

/// Every usable volume root, the system drive included. Empty when none can be
/// discovered (an unusual mount layout, or a platform with no roots).
pub fn list_drives() -> Vec<Drive> {
    #[cfg(windows)]
    {
        windows_drives()
    }
    #[cfg(unix)]
    {
        unix_drives()
    }
    #[cfg(not(any(windows, unix)))]
    {
        Vec::new()
    }
}

/// Enumerate existing drive letters and label each by the type of media it
/// holds. `GetLogicalDrives` only reports mapped letters, and `GetDriveTypeA`
/// is read from the driver without touching the media, so a card reader with
/// no card in it never blocks or prompts for a disk.
#[cfg(windows)]
fn windows_drives() -> Vec<Drive> {
    use std::ffi::CString;

    // https://learn.microsoft.com/windows/win32/api/fileapi/nf-fileapi-getlogicaldrives
    // https://learn.microsoft.com/windows/win32/api/fileapi/nf-fileapi-getdrivetypea
    extern "system" {
        fn GetLogicalDrives() -> u32;
        fn GetDriveTypeA(root: *const i8) -> u32;
    }

    let mask = unsafe { GetLogicalDrives() };
    let mut drives = Vec::new();

    // Bit 0 is A:, bit 1 is B:. Skip both so a machine with a floppy controller
    // is never asked to read a disk that is not there.
    for i in 2u32..26 {
        if mask & (1 << i) == 0 {
            continue;
        }
        let letter = (b'A' + i as u8) as char;
        let root = format!("{letter}:\\");
        let Ok(c_root) = CString::new(root.clone()) else {
            continue;
        };

        let kind = match unsafe { GetDriveTypeA(c_root.as_ptr()) } {
            2 => "removable", // USB stick, external SSD, SD card
            3 => "fixed",     // internal disk / partition
            4 => "network",
            6 => "ramdisk",
            // unknown / no root / optical — not browsable text volumes
            _ => continue,
        };

        drives.push(Drive {
            name: format!("{root} ({kind})"),
            path: root,
        });
    }

    drives
}

/// Collect the conventional places external volumes appear, then list the
/// directories directly inside each one.
///
/// - `/media` and `/run/media` (Linux, usually nested under the user name)
/// - `/Volumes` (macOS)
/// - `/mnt` (manual mounts)
#[cfg(unix)]
fn unix_drives() -> Vec<Drive> {
    use std::collections::BTreeSet;
    use std::path::PathBuf;

    let mut parents: Vec<PathBuf> = vec![
        PathBuf::from("/media"),
        PathBuf::from("/run/media"),
        PathBuf::from("/Volumes"),
        PathBuf::from("/mnt"),
    ];

    // Volumes on Linux often land in `/media/<user>/<label>` or
    // `/run/media/<user>/<label>`, so look one level deeper for each user.
    for key in ["USER", "LOGNAME"] {
        if let Some(user) = std::env::var_os(key) {
            let user = user.to_string_lossy().to_string();
            if user.is_empty() {
                continue;
            }
            parents.push(PathBuf::from("/media").join(&user));
            parents.push(PathBuf::from("/run/media").join(&user));
        }
    }

    let mut roots: BTreeSet<PathBuf> = BTreeSet::new();
    for parent in parents {
        let Ok(entries) = std::fs::read_dir(&parent) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                roots.insert(path);
            }
        }
    }

    roots
        .into_iter()
        .map(|path| {
            let name = path.to_string_lossy().to_string();
            Drive {
                name: name.clone(),
                path: name,
            }
        })
        .collect()
}
