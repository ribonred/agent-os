//! The two shapes the shell takes: full, and a floating pill.
//!
//! One window reconfigured, never two. The conversation is mounted once
//! for the life of the app precisely so a reply keeps arriving while the
//! owner does something else; a second webview would fork that state and
//! need syncing, and the owner would be able to see two copies of one
//! conversation disagreeing.
//!
//! The geometry lives here rather than in the webview so the frontend
//! needs no window permissions to move its own window around, and so
//! there is one place that knows what each mode means.

use serde::{Deserialize, Serialize};
use tauri::{LogicalPosition, LogicalSize, Manager, PhysicalPosition, WebviewWindow};
use tauri_plugin_store::StoreExt;

const STORE_FILE: &str = "settings.json";
const MODE_KEY: &str = "windowMode";
const PILL_X: &str = "pillX";
const PILL_Y: &str = "pillY";

/// The pill is as wide as one comfortable line of conversation and as
/// tall as the input alone; it grows only while there is something to
/// read and settles back when the exchange is done.
const PILL_WIDTH: f64 = 420.0;
const PILL_HEIGHT: f64 = 76.0;
const PILL_HEIGHT_EXPANDED: f64 = 420.0;
/// Clear of the screen edge, so it reads as floating above the desktop
/// rather than stuck to it.
const PILL_MARGIN: f64 = 24.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Mode {
    Full,
    Minimized,
}

impl Mode {
    fn as_str(self) -> &'static str {
        match self {
            Mode::Full => "full",
            Mode::Minimized => "minimized",
        }
    }

    /// Anything unrecognised is full: a device that starts as a pill the
    /// owner never asked for looks broken, whereas an unexpected full
    /// window is merely the normal state.
    fn parse(value: &str) -> Mode {
        match value {
            "minimized" => Mode::Minimized,
            _ => Mode::Full,
        }
    }
}

fn main_window(app: &tauri::AppHandle) -> Result<WebviewWindow, String> {
    app.get_webview_window("main")
        .ok_or_else(|| "the main window is gone".to_string())
}

/// Where the pill last sat, if it has ever been put somewhere.
fn stored_pill_position(app: &tauri::AppHandle) -> Option<LogicalPosition<f64>> {
    let store = app.store(STORE_FILE).ok()?;
    let x = store.get(PILL_X)?.as_f64()?;
    let y = store.get(PILL_Y)?.as_f64()?;
    Some(LogicalPosition::new(x, y))
}

/// Bottom right, one margin in from each edge: out of the way of what
/// the owner is working on, and where a floating helper is conventionally
/// looked for. Used only until they move it somewhere they prefer.
fn default_pill_position(window: &WebviewWindow) -> Option<LogicalPosition<f64>> {
    let monitor = window.current_monitor().ok().flatten()?;
    let scale = monitor.scale_factor();
    let size = monitor.size().to_logical::<f64>(scale);
    let origin: LogicalPosition<f64> = monitor.position().to_logical(scale);
    Some(LogicalPosition::new(
        origin.x + size.width - PILL_WIDTH - PILL_MARGIN,
        origin.y + size.height - PILL_HEIGHT - PILL_MARGIN * 2.0,
    ))
}

/// Remembers where the owner dragged the pill to, so it is there next
/// time rather than back in a corner they moved it out of.
fn remember_pill_position(app: &tauri::AppHandle, window: &WebviewWindow) {
    let Ok(position) = window.outer_position() else {
        return;
    };
    let scale = window.scale_factor().unwrap_or(1.0);
    let position: LogicalPosition<f64> = PhysicalPosition::new(position.x, position.y)
        .to_logical(scale);
    let Ok(store) = app.store(STORE_FILE) else {
        return;
    };
    store.set(PILL_X, position.x);
    store.set(PILL_Y, position.y);
    let _ = store.save();
}

/// Turns a window-manager refusal into something worth reading, without
/// naming the call that failed.
fn failed(what: &'static str) -> impl Fn(tauri::Error) -> String {
    move |e: tauri::Error| format!("could not {what}: {e}")
}

/// Puts the window into one mode. Called on startup with the stored mode
/// and whenever the owner switches.
pub fn apply(app: &tauri::AppHandle, mode: Mode) -> Result<(), String> {
    let window = main_window(app)?;

    match mode {
        Mode::Full => {
            // Leaving the pill: note where it was before the window
            // stops being one.
            if matches!(current(app), Mode::Minimized) {
                remember_pill_position(app, &window);
            }
            window
                .set_always_on_top(false)
                .map_err(failed("release the window"))?;
            window
                .set_visible_on_all_workspaces(false)
                .map_err(failed("release the window"))?;
            window
                .set_skip_taskbar(false)
                .map_err(failed("show the window"))?;
            window.set_resizable(true).map_err(failed("resize"))?;
            // Maximized, not fullscreen: the desktop underneath stays
            // reachable, and a fullscreen window hides the system bar the
            // owner may need to get back to something else.
            window.maximize().map_err(failed("fill the screen"))?;
        }
        Mode::Minimized => {
            window.unmaximize().map_err(failed("shrink the window"))?;
            window
                .set_size(LogicalSize::new(PILL_WIDTH, PILL_HEIGHT))
                .map_err(failed("shrink the window"))?;
            if let Some(position) =
                stored_pill_position(app).or_else(|| default_pill_position(&window))
            {
                window
                    .set_position(position)
                    .map_err(failed("place the window"))?;
            }
            window
                .set_always_on_top(true)
                .map_err(failed("keep the window in front"))?;
            // It follows the owner rather than living on one workspace:
            // the whole point is being reachable from whatever they are
            // doing.
            window
                .set_visible_on_all_workspaces(true)
                .map_err(failed("keep the window in front"))?;
            // No taskbar entry: the pill is already visible, and a second
            // way to summon something never hidden is clutter.
            window
                .set_skip_taskbar(true)
                .map_err(failed("hide the window from the taskbar"))?;
            window.set_resizable(false).map_err(failed("resize"))?;
        }
    }
    Ok(())
}

/// The stored mode, defaulting to full on a device that has never
/// chosen -- and on any failure to read, for the same reason.
pub fn current(app: &tauri::AppHandle) -> Mode {
    app.store(STORE_FILE)
        .ok()
        .and_then(|store| store.get(MODE_KEY))
        .and_then(|value| value.as_str().map(Mode::parse))
        .unwrap_or(Mode::Full)
}

#[tauri::command]
pub fn window_mode_get(app: tauri::AppHandle) -> String {
    current(&app).as_str().to_string()
}

#[tauri::command]
pub fn window_mode_set(mode: String, app: tauri::AppHandle) -> Result<String, String> {
    let mode = Mode::parse(&mode);
    apply(&app, mode)?;
    let store = app
        .store(STORE_FILE)
        .map_err(|e| format!("could not open the settings store: {e}"))?;
    store.set(MODE_KEY, mode.as_str());
    store
        .save()
        .map_err(|e| format!("could not save the settings store: {e}"))?;
    Ok(mode.as_str().to_string())
}

/// Grows the pill while there is something to read, and settles it back
/// when there isn't. A no-op in full mode, so the frontend can call it
/// whenever the conversation changes without first asking what shape the
/// window is.
#[tauri::command]
pub fn window_pill_expand(expanded: bool, app: tauri::AppHandle) -> Result<(), String> {
    if !matches!(current(&app), Mode::Minimized) {
        return Ok(());
    }
    let window = main_window(&app)?;
    let height = if expanded {
        PILL_HEIGHT_EXPANDED
    } else {
        PILL_HEIGHT
    };
    window
        .set_size(LogicalSize::new(PILL_WIDTH, height))
        .map_err(|e| format!("could not resize the window: {e}"))
}

/// Hands the drag to the window manager for the duration of a press.
/// The pill has no title bar to grab, so its whole surface is the handle.
#[tauri::command]
pub fn window_drag(app: tauri::AppHandle) -> Result<(), String> {
    main_window(&app)?
        .start_dragging()
        .map_err(|e| format!("could not move the window: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_mode_round_trips_through_its_stored_spelling() {
        assert_eq!(Mode::parse(Mode::Full.as_str()), Mode::Full);
        assert_eq!(Mode::parse(Mode::Minimized.as_str()), Mode::Minimized);
    }

    #[test]
    fn anything_unrecognised_starts_full() {
        // A device that opens as a pill the owner never asked for looks
        // broken; an unexpected full window is merely the normal state.
        assert_eq!(Mode::parse(""), Mode::Full);
        assert_eq!(Mode::parse("kiosk"), Mode::Full);
        assert_eq!(Mode::parse("MINIMIZED"), Mode::Full);
    }

    #[test]
    fn the_pill_grows_but_never_wider() {
        // The pane has a measure that suits bare prose; growth is
        // downward only, so a long reply never widens the window.
        assert!(PILL_HEIGHT_EXPANDED > PILL_HEIGHT);
        assert_eq!(PILL_WIDTH, 420.0);
    }
}
