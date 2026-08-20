// Views: pages the device builds for the owner to look at.
//
// A view is a folder in the owner's own home -- `Views/<name>/` holding
// an `index.html` -- served to the shell through a `view://` protocol
// rather than read into the webview as text. That split is the whole
// security posture of this module: the page is authored by a language
// model, and a language model's output is not a trusted input. It never
// touches the shell's own document, where the Tauri command bridge
// lives; it is loaded into a sandboxed frame with its own opaque origin
// and no way to reach back.
//
// Containment is deliberately two independent checks, the same shape the
// file view uses: `shelf::resolve` refuses anything that climbs or
// restarts at the filesystem root and re-checks the canonical path
// against the owner's home, and this module then re-checks it against
// the views directory. Either alone would hold on a good day. A symlink
// inside a view pointing at the owner's tax return is the case the
// second check exists for.

use std::fs;
use std::path::{Path, PathBuf};

use serde::Serialize;
use tauri::http::{Request, Response, StatusCode};

use crate::shelf;

/// The owner's views, relative to their home. Named plainly because it
/// sits in their file browser beside Documents and Downloads and they
/// read it every day.
pub const VIEWS_DIR: &str = "Views";

/// A folder is a view when it holds this. The marker is the page itself
/// rather than a name or a location, so a view the owner moves or
/// renames is still a view.
const INDEX: &str = "index.html";

/// What the device knows about a view that its folder cannot say.
/// Optional in every sense: a view whose metadata is missing or corrupt
/// still renders, under its folder name.
const MANIFEST: &str = "view.json";

/// What a view is allowed to do, sent with every page.
///
/// The sandbox attribute on the frame stops scripts; this stops the
/// page reaching the network at all, which the sandbox does not. That
/// matters more than it first appears: a view is markup written by a
/// model, and the model reads the owner's documents -- one of which may
/// have been written by somebody else with an interest in what is in
/// the others. `<img src="https://elsewhere/?figures=...">` needs no
/// script to run, and neither does `url()` in a style attribute.
///
/// `default-src 'none'` refuses everything and the rest adds back only
/// what a page is made of. The device may never see a network in any
/// case; this makes that a rule rather than a circumstance.
///
/// The scheme is named rather than written `'self'`. `'self'` looked
/// right and silently blocked the device's own stylesheets: WebKitGTK
/// does not match it against a custom scheme, so every view rendered
/// with no styling at all. It was invisible for a while because the
/// engine still had a cached copy of the same framework from a network
/// URL, which made a completely unstyled page look like a working one.
/// `view:` is exactly as narrow -- it is the only scheme this webview
/// can reach that is not the network.
const VIEW_CSP: &str = "default-src 'none'; \
     style-src view: 'unsafe-inline'; \
     img-src view: data:; \
     font-src view: data:; \
     base-uri 'none'; \
     form-action 'none'";

/// Assets every view shares, reachable at `view://localhost/_shared/…`.
/// One stylesheet on the device rather than a copy inside each view, so
/// a change to the design reaches views that were written months ago.
const SHARED_PREFIX: &str = "_shared";

/// Where those shared assets live on a built device. Overridable by
/// environment for a development checkout -- one mechanism, rather than
/// a dev-only branch that could behave differently from what ships.
fn shared_root() -> PathBuf {
    let root = std::env::var_os("AGENTIC_OS_SHARE")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/usr/local/share/agentic-os"))
        .join("view-assets");

    // Said out loud, because the failure is otherwise invisible: the
    // stylesheets 404, every view renders as unstyled markup, and
    // nothing anywhere reports a problem. In a development checkout
    // these live in the repo, and the Makefile points at them.
    if !root.is_dir() {
        log::warn!(
            "no shared view assets at {} -- views will render unstyled; \
             set AGENTIC_OS_SHARE to the directory holding view-assets/",
            root.display()
        );
    }
    root
}

/// Exactly what a view may be made of. Not a blocklist: anything not
/// named here is not served, so a new file type is a decision rather
/// than an oversight.
///
/// No `.js`. Views do not run -- see design/DESIGN.md. Drawings are
/// computed before the page exists and arrive as SVG.
fn content_type(path: &Path) -> Option<&'static str> {
    let ext = path
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_lowercase();

    Some(match ext.as_str() {
        "html" => "text/html; charset=utf-8",
        // Vendored alongside the device's own stylesheet so a view has
        // layout primitives the assistant already knows, without
        // reaching a network for them.
        "txt" | "license" => "text/plain; charset=utf-8",
        "css" => "text/css; charset=utf-8",
        "svg" => "image/svg+xml",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "webp" => "image/webp",
        "woff2" => "font/woff2",
        "json" => "application/json",
        _ => return None,
    })
}

/// Percent-decoding, because a folder the owner named "June takings"
/// arrives as "June%20takings". Written out rather than pulled in: this
/// is the only place the shell needs it, and a dependency that ships to
/// a customer's device should earn more than fifteen lines.
fn percent_decode(raw: &str) -> String {
    let bytes = raw.as_bytes();
    let mut out: Vec<u8> = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            let hex = std::str::from_utf8(&bytes[i + 1..i + 3]).ok();
            if let Some(byte) = hex.and_then(|h| u8::from_str_radix(h, 16).ok()) {
                out.push(byte);
                i += 3;
                continue;
            }
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

/// Resolve one request path to a file inside the views directory.
///
/// The second containment check lives here: `shelf::resolve` guarantees
/// the result is inside the owner's home, and this narrows that to the
/// views directory so a symlink planted in a view cannot serve the rest
/// of the home through a protocol the sandboxed frame can reach.
fn resolve_in_views(relative: &str) -> Option<PathBuf> {
    let joined = format!("{VIEWS_DIR}/{relative}");
    let real = shelf::resolve(&joined).ok()?;

    let root = shelf::root().ok()?.join(VIEWS_DIR);
    let real_root = root.canonicalize().ok()?;
    if !real.starts_with(&real_root) {
        return None;
    }
    real.is_file().then_some(real)
}

/// How a view is dressed. Two values and no others: the theme is written
/// straight into the page as it is served, and accepting exactly two
/// literals is what makes that safe to do at all.
///
/// Light is the default because a view is a document -- read closely,
/// and printed onto white paper. The shell around it stays dark.
/// See design/DESIGN.md.
fn theme_from_query(query: Option<&str>) -> &'static str {
    let wanted = query
        .unwrap_or("")
        .split('&')
        .find_map(|pair| pair.strip_prefix("theme="));
    match wanted {
        Some("dark") => "dark",
        _ => "light",
    }
}

/// The page, dressed the way the owner asked.
///
/// Rewriting one attribute rather than serving two stylesheets: the theme
/// is a property of how the owner is looking at the page, not of the page
/// itself, so it does not belong written into their file on disk. Their
/// file is never touched -- this rewrites the copy on its way out.
fn with_theme(html: &str, theme: &str) -> String {
    const ATTR: &str = "data-bs-theme=\"";
    if let Some(start) = html.find(ATTR) {
        let value = start + ATTR.len();
        if let Some(end) = html[value..].find('"') {
            let mut out = String::with_capacity(html.len());
            out.push_str(&html[..value]);
            out.push_str(theme);
            out.push_str(&html[value + end..]);
            return out;
        }
    }
    // A page that never declared one still gets it, so a hand-written
    // view is themed like every other.
    match html.find("<html") {
        Some(tag) => {
            let insert = tag + "<html".len();
            format!(
                "{}{}{}",
                &html[..insert],
                format_args!(" data-bs-theme=\"{theme}\""),
                &html[insert..]
            )
        }
        None => html.to_string(),
    }
}

fn not_found() -> Response<Vec<u8>> {
    Response::builder()
        .status(StatusCode::NOT_FOUND)
        .body(Vec::new())
        .unwrap_or_default()
}

/// Serve one request from the `view://` scheme.
///
/// On Linux -- the only platform this ships on -- WebKitGTK addresses a
/// custom scheme as `view://localhost/<path>`, so the host carries
/// nothing and the path is everything.
pub fn serve(request: &Request<Vec<u8>>) -> Response<Vec<u8>> {
    let raw = percent_decode(request.uri().path().trim_start_matches('/'));
    if raw.is_empty() {
        return not_found();
    }

    let path = match raw.strip_prefix(&format!("{SHARED_PREFIX}/")) {
        // Shared assets are the device's own files, not the owner's, so
        // they resolve against the install root instead. Still a single
        // flat directory with no traversal: a name, nothing more.
        Some(name) => {
            if name.is_empty() || name.contains('/') || name.contains('\\') {
                return not_found();
            }
            shared_root().join(name)
        }
        None => match resolve_in_views(&raw) {
            Some(path) => path,
            None => return not_found(),
        },
    };

    let Some(mime) = content_type(&path) else {
        return not_found();
    };
    let Ok(body) = fs::read(&path) else {
        return not_found();
    };

    // Only the page carries the attribute; the assets beside it do not.
    let body = if mime.starts_with("text/html") {
        let theme = theme_from_query(request.uri().query());
        with_theme(&String::from_utf8_lossy(&body), theme).into_bytes()
    } else {
        body
    };

    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", mime)
        // The frame is sandboxed to an opaque origin, so nothing here is
        // same-origin with the shell; these say so explicitly rather
        // than relying on that alone.
        .header("Cache-Control", "no-store")
        .header("X-Content-Type-Options", "nosniff")
        .header("Content-Security-Policy", VIEW_CSP)
        .body(body)
        .unwrap_or_else(|_| not_found())
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct View {
    /// Folder name, and the id the shell addresses it by.
    pub name: String,
    /// What to call it on screen. Falls back to the folder name.
    pub title: String,
    /// The question it was built to answer, in the owner's own words.
    pub asked: Option<String>,
    /// Where its figures came from, named the way the owner names
    /// files. Empty when the view never said.
    pub from: Vec<String>,
    /// Milliseconds since the epoch, from the page itself -- when the
    /// view last changed, not when its folder was touched.
    pub modified: u64,
}

/// Read a view's manifest. Every field is optional and a broken file is
/// not an error: metadata exists to make a view readable, and refusing
/// to show a page because its label is malformed helps nobody.
fn manifest(dir: &Path) -> Option<serde_json::Value> {
    let raw = fs::read_to_string(dir.join(MANIFEST)).ok()?;
    serde_json::from_str(&raw).ok()
}

fn modified_ms(path: &Path) -> u64 {
    fs::metadata(path)
        .and_then(|m| m.modified())
        .ok()
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

/// Whether a directory is a view. Used by the file browser so a view
/// shows as one row that opens it, rather than a folder the owner walks
/// into and finds markup in.
pub fn is_view_dir(path: &Path) -> bool {
    path.join(INDEX).is_file()
}

/// The views that exist, as one comma-separated line for the per-turn
/// overlay. `None` when there are none, so the overlay says nothing at
/// all rather than announcing an empty list.
pub fn existing_names() -> Option<String> {
    let names: Vec<String> = views_list().ok()?.into_iter().map(|v| v.name).collect();
    (!names.is_empty()).then(|| names.join(", "))
}

/// Every view the owner has, newest first.
#[tauri::command]
pub fn views_list() -> Result<Vec<View>, String> {
    let root = shelf::root()?.join(VIEWS_DIR);
    let Ok(entries) = fs::read_dir(&root) else {
        // No views directory yet is not a failure -- it is a device that
        // has not been asked for anything yet.
        return Ok(Vec::new());
    };

    let mut views: Vec<View> = entries
        .flatten()
        .filter_map(|entry| {
            let dir = entry.path();
            if !is_view_dir(&dir) {
                return None;
            }
            let name = entry.file_name().to_str()?.to_string();
            if name.starts_with('.') {
                return None;
            }

            let meta = manifest(&dir);
            let field = |key: &str| {
                meta.as_ref()
                    .and_then(|m| m.get(key))
                    .and_then(|v| v.as_str())
                    .map(str::trim)
                    .filter(|s| !s.is_empty())
                    .map(str::to_string)
            };

            Some(View {
                title: field("title").unwrap_or_else(|| name.clone()),
                asked: field("asked"),
                from: meta
                    .as_ref()
                    .and_then(|m| m.get("from"))
                    .and_then(|v| v.as_array())
                    .map(|items| {
                        items
                            .iter()
                            .filter_map(|i| i.as_str())
                            .map(str::to_string)
                            .collect()
                    })
                    .unwrap_or_default(),
                modified: modified_ms(&dir.join(INDEX)),
                name,
            })
        })
        .collect();

    // What the device made most recently is what the owner is most
    // likely to be looking for.
    views.sort_by(|a, b| b.modified.cmp(&a.modified).then(a.name.cmp(&b.name)));
    Ok(views)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get(path: &str) -> Response<Vec<u8>> {
        let request = Request::builder()
            .uri(format!("view://localhost/{path}"))
            .body(Vec::new())
            .expect("a request");
        serve(&request)
    }

    /// The view is served into a frame the model's own markup runs in.
    /// Every one of these is a way that frame could have been handed a
    /// file it has no business seeing.
    #[test]
    fn a_request_that_climbs_out_of_the_views_folder_is_refused() {
        for path in [
            "../.bashrc",
            "../../etc/passwd",
            "a/../../../etc/passwd",
            "%2e%2e/%2e%2e/etc/passwd",
        ] {
            assert_eq!(
                get(path).status(),
                StatusCode::NOT_FOUND,
                "traversal was served: {path}"
            );
        }
    }

    /// The frame's sandbox stops scripts. This stops the page reaching
    /// the network, which the sandbox does not -- and which is the way a
    /// page built from the owner's documents could carry them back out.
    #[test]
    fn a_view_is_not_allowed_to_reach_anything() {
        assert!(VIEW_CSP.contains("default-src 'none'"));
        // Everything a page is actually made of, and nothing that could
        // address a host: no connect-src, no script-src, no frame-src.
        for allowed in ["style-src view:", "img-src view: data:", "font-src view: data:"] {
            assert!(VIEW_CSP.contains(allowed), "missing: {allowed}");
        }
        // `'self'` is what this obviously wanted to say and it does not
        // work here -- see the note on the constant. Asserted so it is
        // not helpfully "corrected" back later.
        assert!(!VIEW_CSP.contains("'self'"), "'self' does not match a custom scheme");
        assert!(!VIEW_CSP.contains("http"), "a view may not name a host");
        assert!(!VIEW_CSP.contains("connect-src"));
        assert!(!VIEW_CSP.contains("*"));
        assert!(VIEW_CSP.contains("form-action 'none'"));
    }

    #[test]
    fn the_theme_is_two_literals_and_nothing_else() {
        assert_eq!(theme_from_query(Some("theme=dark")), "dark");
        assert_eq!(theme_from_query(Some("theme=light")), "light");
        // A view is a document, and paper is white.
        assert_eq!(theme_from_query(None), "light");
        assert_eq!(theme_from_query(Some("")), "light");
        // Anything else is the default, which is what makes writing this
        // straight into the page safe: two literals can be spelled, and
        // nothing a caller invents ever reaches the markup.
        for odd in [
            "theme=\" onload=alert(1) x=\"",
            "theme=DARK",
            "theme=",
            "colour=dark",
        ] {
            assert_eq!(theme_from_query(Some(odd)), "light", "{odd}");
        }
    }

    #[test]
    fn the_page_is_dressed_without_touching_the_owners_file() {
        let page = "<!doctype html>\n<html lang=\"en\" data-bs-theme=\"light\">\n<body>hi</body>";
        let dark = with_theme(page, "dark");
        assert!(dark.contains("data-bs-theme=\"dark\""));
        assert!(!dark.contains("data-bs-theme=\"light\""));
        // Everything else is exactly as it was.
        assert!(dark.contains("<html lang=\"en\""));
        assert!(dark.contains("<body>hi</body>"));
        assert_eq!(dark.len(), page.len() - 1);

        // And back again, so the switch is not one-way.
        assert!(with_theme(&dark, "light").contains("data-bs-theme=\"light\""));
    }

    #[test]
    fn a_page_that_never_declared_a_theme_still_gets_one() {
        let bare = "<!doctype html>\n<html lang=\"en\">\n<body>hi</body>";
        let out = with_theme(bare, "dark");
        assert!(out.contains("data-bs-theme=\"dark\""), "{out}");
        assert!(out.contains("lang=\"en\""));

        // Something that is not a page at all is left alone rather than
        // having markup invented around it.
        assert_eq!(with_theme("just text", "dark"), "just text");
    }

    #[test]
    fn only_the_named_file_types_are_served() {
        // The allowlist is the whole rule, so the answer for anything
        // absent from it is the same as for a file that isn't there.
        assert!(content_type(Path::new("index.html")).is_some());
        assert!(content_type(Path::new("chart.svg")).is_some());
        assert!(content_type(Path::new("view.css")).is_some());

        // Views do not run. A script is not a thing a view can contain,
        // whichever way it is spelled.
        assert!(content_type(Path::new("chart.js")).is_none());
        assert!(content_type(Path::new("chart.mjs")).is_none());
        assert!(content_type(Path::new("run.sh")).is_none());
        assert!(content_type(Path::new("index.wasm")).is_none());
        assert!(content_type(Path::new("secrets")).is_none());
    }

    #[test]
    fn shared_assets_are_a_flat_name_and_never_a_path() {
        // `_shared` resolves against the device's install root rather
        // than the owner's home, so traversal there would reach the
        // filesystem instead of merely the wrong folder.
        assert_eq!(get("_shared/../../etc/passwd").status(), StatusCode::NOT_FOUND);
        assert_eq!(get("_shared/").status(), StatusCode::NOT_FOUND);
        assert_eq!(get("_shared").status(), StatusCode::NOT_FOUND);
    }

    #[test]
    fn an_empty_request_is_not_a_directory_listing() {
        assert_eq!(get("").status(), StatusCode::NOT_FOUND);
        assert_eq!(get("/").status(), StatusCode::NOT_FOUND);
    }

    #[test]
    fn a_folder_is_a_view_only_when_it_holds_a_page() {
        let dir = std::env::temp_dir().join("agentic-os-views-test");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).expect("a temp dir");
        assert!(!is_view_dir(&dir), "an empty folder is not a view");

        fs::write(dir.join(INDEX), "<h1>hi</h1>").expect("a page");
        assert!(is_view_dir(&dir), "a folder with a page is a view");

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn percent_escapes_survive_a_folder_named_by_a_person() {
        assert_eq!(percent_decode("June%20takings"), "June takings");
        assert_eq!(percent_decode("plain"), "plain");
        // A stray percent is text, not the start of an escape.
        assert_eq!(percent_decode("100%"), "100%");
    }

    /// The owner is never shown where anything is, and neither is the
    /// frame: a listing carries folder names, never a path.
    #[test]
    fn a_listing_never_names_a_location() {
        let views = views_list().expect("a listing");
        for view in views {
            assert!(!view.name.contains('/'), "a path leaked: {}", view.name);
            assert!(!view.name.starts_with('.'));
        }
    }
}
