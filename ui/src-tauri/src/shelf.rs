// The file view: what is actually on the owner's device.
//
// This is a real file manager -- real directories, nested to any depth,
// showing what the disk shows. The agentic part is not that the files are
// curated or pre-digested; it is that the owner can point at something
// here and ask the device about it.
//
// Every filesystem operation lives in this module rather than in the
// webview. Entries are addressed by a path *relative to the owner's
// home*, and every incoming path is re-resolved and checked against that
// root before it is read, so the browser layer cannot reach outside it.
//
// The same code runs on the appliance and on a developer's machine: the
// root is the current user's home either way, so there is no dev-only
// branch that could behave differently from what ships.

use std::fs;
use std::path::{Component, Path, PathBuf};

use serde::Serialize;

/// Directories the product provides if the owner has none, so a fresh
/// device is not an empty screen. Deliberately only these two: creating
/// the full set of XDG user directories would add Desktop, Templates and
/// Public, which is clutter the owner never asked for.
const STARTER_DIRS: [&str; 2] = ["Documents", "Downloads"];

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Entry {
    /// Path relative to home, e.g. "project/private". Also the id used to
    /// navigate into a directory.
    pub path: String,
    /// The real name on disk, extension and all.
    pub name: String,
    pub is_dir: bool,
    /// How many visible things a directory holds; zero for files.
    pub count: u32,
    /// Bytes; zero for directories. The UI decides whether to say it.
    pub size: u64,
    /// Milliseconds since the epoch.
    pub modified: u64,
    /// Coarse kind, only for choosing an icon.
    pub kind: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Crumb {
    pub path: String,
    pub name: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Listing {
    /// The directory being shown, relative to home. Empty string = home.
    pub path: String,
    /// Each ancestor from home down to here, for a breadcrumb.
    pub crumbs: Vec<Crumb>,
    pub entries: Vec<Entry>,
}

/// The owner's home -- the one root, on the appliance and in development.
fn root() -> Result<PathBuf, String> {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .filter(|p| !p.as_os_str().is_empty())
        .ok_or_else(|| "no home directory".to_string())
}

/// Dotfiles are the machine's business, not the owner's.
fn is_hidden(name: &str) -> bool {
    name.starts_with('.')
}

/// Resolve a relative path against home, refusing anything that escapes.
///
/// Two independent checks, because either alone is insufficient: the
/// component scan rejects `..` and absolute paths before they are ever
/// joined, and the canonicalised prefix check catches a symlink that
/// resolves to somewhere outside the home.
fn resolve(relative: &str) -> Result<PathBuf, String> {
    let root = root()?;
    let candidate = Path::new(relative);

    for component in candidate.components() {
        match component {
            Component::Normal(_) => {}
            // A path that climbs, or restarts at the filesystem root, is
            // not a mistake to normalise away -- refuse it outright.
            _ => return Err("that isn't somewhere I can look".to_string()),
        }
    }

    let real = root
        .join(candidate)
        .canonicalize()
        .map_err(|_| "I can't find that".to_string())?;
    let real_root = root.canonicalize().unwrap_or(root);
    if !real.starts_with(&real_root) {
        return Err("that isn't somewhere I can look".to_string());
    }
    Ok(real)
}

fn modified_ms(meta: &fs::Metadata) -> u64 {
    meta.modified()
        .ok()
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

/// Coarse kind, only as fine-grained as the icon set distinguishes. The
/// extension stays visible in the name -- this exists to pick a glyph,
/// not to hide what the file is.
pub fn kind_for(name: &str) -> &'static str {
    let ext = Path::new(name)
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_lowercase();

    match ext.as_str() {
        "csv" | "xlsx" | "xls" | "tsv" | "ods" => "table",
        "jpg" | "jpeg" | "png" | "gif" | "webp" | "heic" | "bmp" | "tiff" | "svg" | "ico" => {
            "image"
        }
        "pdf" | "docx" | "doc" | "odt" | "rtf" | "pages" => "document",
        "mp3" | "wav" | "flac" | "ogg" | "m4a" | "aac" => "audio",
        "mp4" | "mkv" | "mov" | "avi" | "webm" => "video",
        "zip" | "tar" | "gz" | "bz2" | "xz" | "7z" | "rar" => "archive",
        "" => "file",
        _ => "text",
    }
}

/// How many visible things a directory holds. A directory that cannot be
/// read reports zero rather than failing the listing -- one unreadable
/// folder should not blank the whole screen.
fn visible_count(dir: &Path) -> u32 {
    let Ok(entries) = fs::read_dir(dir) else {
        return 0;
    };
    entries
        .flatten()
        .filter(|e| {
            e.file_name()
                .to_str()
                .map(|n| !is_hidden(n))
                .unwrap_or(false)
        })
        .count() as u32
}

/// Create the starter directories only when the owner has neither, so a
/// fresh device opens on something rather than nothing. Someone who
/// deleted one on purpose should not find it silently recreated.
fn ensure_starter_dirs(root: &Path) {
    if STARTER_DIRS.iter().any(|name| root.join(name).exists()) {
        return;
    }
    for name in STARTER_DIRS {
        // Best effort: a home that cannot be written to is still worth
        // showing whatever it does contain.
        let _ = fs::create_dir_all(root.join(name));
    }
}

fn crumbs_for(relative: &str) -> Vec<Crumb> {
    let mut crumbs = Vec::new();
    let mut walked = String::new();
    for part in relative.split('/').filter(|p| !p.is_empty()) {
        if !walked.is_empty() {
            walked.push('/');
        }
        walked.push_str(part);
        crumbs.push(Crumb {
            path: walked.clone(),
            name: part.to_string(),
        });
    }
    crumbs
}

/// List one directory: folders and files together, the way a file
/// manager shows them. `path` is relative to home; empty means home.
#[tauri::command]
pub fn shelf_list(path: String) -> Result<Listing, String> {
    let root = root()?;
    if path.is_empty() {
        ensure_starter_dirs(&root);
    }

    let dir = if path.is_empty() {
        root.clone()
    } else {
        resolve(&path)?
    };

    let read = fs::read_dir(&dir).map_err(|_| "I can't open that".to_string())?;

    let mut entries: Vec<Entry> = Vec::new();
    for item in read.flatten() {
        let Some(name) = item.file_name().to_str().map(|s| s.to_string()) else {
            continue;
        };
        if is_hidden(&name) {
            continue;
        }
        let Ok(meta) = item.metadata() else { continue };
        let is_dir = meta.is_dir();

        let child_path = if path.is_empty() {
            name.clone()
        } else {
            format!("{path}/{name}")
        };

        entries.push(Entry {
            path: child_path,
            is_dir,
            count: if is_dir { visible_count(&item.path()) } else { 0 },
            size: if is_dir { 0 } else { meta.len() },
            modified: modified_ms(&meta),
            kind: if is_dir { "folder" } else { kind_for(&name) }.to_string(),
            name,
        });
    }

    // Folders first, then files, each alphabetically -- what a file
    // manager does, and it keeps a directory's shape stable as its
    // contents change. Case-insensitive so names sort the way a person
    // reads them rather than the way ASCII orders them.
    entries.sort_by(|a, b| {
        b.is_dir
            .cmp(&a.is_dir)
            .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
    });

    Ok(Listing {
        crumbs: crumbs_for(&path),
        path,
        entries,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kind_picks_an_icon_without_hiding_the_extension() {
        assert_eq!(kind_for("prices.csv"), "table");
        assert_eq!(kind_for("photo.JPG"), "image");
        assert_eq!(kind_for("invoice.pdf"), "document");
        assert_eq!(kind_for("song.mp3"), "audio");
        assert_eq!(kind_for("backup.tar"), "archive");
        assert_eq!(kind_for("notes.txt"), "text");
        assert_eq!(kind_for("README"), "file");
    }

    #[test]
    fn hidden_entries_are_never_listed() {
        assert!(is_hidden(".config"));
        assert!(is_hidden(".bashrc"));
        assert!(!is_hidden("Documents"));
    }

    #[test]
    fn traversal_out_of_the_home_is_refused() {
        // Both forms matter: climbing out, and restarting at the root.
        assert!(resolve("../etc").is_err());
        assert!(resolve("project/../../etc").is_err());
        assert!(resolve("/etc/passwd").is_err());
    }

    #[test]
    fn crumbs_describe_the_path_back_to_home() {
        let crumbs = crumbs_for("project/private/agentic-os");
        let names: Vec<&str> = crumbs.iter().map(|c| c.name.as_str()).collect();
        let paths: Vec<&str> = crumbs.iter().map(|c| c.path.as_str()).collect();
        assert_eq!(names, ["project", "private", "agentic-os"]);
        assert_eq!(
            paths,
            ["project", "project/private", "project/private/agentic-os"]
        );
        assert!(crumbs_for("").is_empty());
    }

    #[test]
    fn listing_home_shows_directories_before_files_and_hides_dotfiles() {
        let listing = shelf_list(String::new()).expect("home should list");
        let mut seen_file = false;
        for entry in &listing.entries {
            if entry.is_dir {
                assert!(!seen_file, "a directory came after a file: {}", entry.name);
            } else {
                seen_file = true;
            }
            assert!(!entry.name.starts_with('.'), "a dotfile was listed");
        }
        assert_eq!(listing.path, "");
        assert!(listing.crumbs.is_empty());
    }

    #[test]
    fn a_directory_inside_a_directory_is_reachable() {
        // The whole point of this module: nesting works to any depth, and
        // the crumbs lead back to home.
        let home = shelf_list(String::new()).expect("home should list");
        let Some(dir) = home.entries.iter().find(|e| e.is_dir && e.count > 0) else {
            return; // Nothing nested on this machine; nothing to prove.
        };
        let inner = shelf_list(dir.path.clone()).expect("a subdirectory should list");
        assert_eq!(inner.path, dir.path);
        assert_eq!(
            inner.crumbs.last().map(|c| c.name.as_str()),
            Some(dir.name.as_str())
        );
    }
}

