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
// The full window's own geometry, remembered for the same reason the
// pill's is: coming back should return the owner to the window they
// left, not to whatever the last mode happened to leave behind.
const FULL_X: &str = "fullX";
const FULL_Y: &str = "fullY";
const FULL_W: &str = "fullWidth";
const FULL_H: &str = "fullHeight";
const FULL_MAXIMIZED: &str = "fullMaximized";

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

const MIN_FULL_WIDTH: f64 = 640.0;
const MIN_FULL_HEIGHT: f64 = 480.0;

fn is_plausible_full_size(size: &LogicalSize<f64>) -> bool {
    size.width >= MIN_FULL_WIDTH && size.height >= MIN_FULL_HEIGHT
}

/// Remembers the full window before it becomes a pill.
///
/// Without this there is nothing to go back to: the pill sets a 420x76
/// size, and that is then the only size the window has ever been told
/// about. Restoring "full" would leave a pill-sized window wearing the
/// full layout, which is what a broken expand looks like.
fn remember_full_geometry(app: &tauri::AppHandle, window: &WebviewWindow) {
    let Ok(store) = app.store(STORE_FILE) else {
        return;
    };
    let maximized = window.is_maximized().unwrap_or(true);
    store.set(FULL_MAXIMIZED, maximized);
    // A maximized window's own size is the screen's, which is not a size
    // worth restoring to -- what matters is that it was maximized.
    if !maximized {
        let scale = window.scale_factor().unwrap_or(1.0);
        if let Ok(size) = window.outer_size() {
            let size = size.to_logical::<f64>(scale);
            if is_plausible_full_size(&size) {
                store.set(FULL_W, size.width);
                store.set(FULL_H, size.height);
            } else {
                // The window was already pill-sized when this ran, so
                // there is no full geometry worth keeping. Forget it and
                // fill the screen next time rather than recording a size
                // the owner never chose.
                store.delete(FULL_W);
                store.delete(FULL_H);
                store.set(FULL_MAXIMIZED, true);
            }
        }
        if let Ok(position) = window.outer_position() {
            let position: LogicalPosition<f64> =
                PhysicalPosition::new(position.x, position.y).to_logical(scale);
            store.set(FULL_X, position.x);
            store.set(FULL_Y, position.y);
        }
    }
    let _ = store.save();
}

/// The size and position the full window had, when it was not maximized.
fn stored_full_geometry(
    app: &tauri::AppHandle,
) -> (bool, Option<LogicalSize<f64>>, Option<LogicalPosition<f64>>) {
    let Ok(store) = app.store(STORE_FILE) else {
        // Never sized by the owner: fill the screen, which is what a
        // device that has just been switched on should do.
        return (true, None, None);
    };
    let maximized = store
        .get(FULL_MAXIMIZED)
        .and_then(|value| value.as_bool())
        .unwrap_or(true);
    let size = store
        .get(FULL_W)
        .and_then(|w| w.as_f64())
        .zip(store.get(FULL_H).and_then(|h| h.as_f64()))
        .map(|(w, h)| LogicalSize::new(w, h))
        // A store written before this was guarded can still hold the
        // pill's geometry. Dropping it here means the window fills the
        // screen once and records something sane, rather than the owner
        // resizing by hand on every start forever.
        .filter(is_plausible_full_size);
    let position = store
        .get(FULL_X)
        .and_then(|x| x.as_f64())
        .zip(store.get(FULL_Y).and_then(|y| y.as_f64()))
        .map(|(x, y)| LogicalPosition::new(x, y));
    (maximized, size, position)
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

/// Puts the pill at its size and place.
fn place_pill(app: &tauri::AppHandle, window: &WebviewWindow) -> Result<(), String> {
    window
        .set_size(LogicalSize::new(PILL_WIDTH, PILL_HEIGHT))
        .map_err(failed("shrink the window"))?;
    if let Some(position) = stored_pill_position(app).or_else(|| default_pill_position(window)) {
        window
            .set_position(position)
            .map_err(failed("place the window"))?;
    }
    Ok(())
}

/// Re-asks for the pill's geometry while the window manager finishes
/// unmaximizing. Stops as soon as the window is the size it was asked
/// for, or as soon as the owner has switched back to the full window --
/// a stray resize arriving after that would shrink a window they are
/// using.
fn settle_pill(app: tauri::AppHandle, was_maximized: bool) {
    // Nothing to wait for when the window was already an ordinary size:
    // the geometry set above took effect immediately.
    if !was_maximized {
        return;
    }
    tauri::async_runtime::spawn(async move {
        for _ in 0..12 {
            tokio::time::sleep(std::time::Duration::from_millis(60)).await;
            // The owner switched back while this was waiting; a resize
            // landing now would shrink a window they are using.
            if !matches!(current(&app), Mode::Minimized) {
                return;
            }
            let Ok(window) = main_window(&app) else {
                return;
            };
            let scale = window.scale_factor().unwrap_or(1.0);
            let settled = window
                .outer_size()
                .map(|size| (size.to_logical::<f64>(scale).width - PILL_WIDTH).abs() < 2.0)
                .unwrap_or(false);
            if settled {
                return;
            }
            let _ = place_pill(&app, &window);
        }
    });
}

/// Turns a window-manager refusal into something worth reading, without
/// naming the call that failed.
fn failed(what: &'static str) -> impl Fn(tauri::Error) -> String {
    move |e: tauri::Error| format!("could not {what}: {e}")
}

/// Puts the window into one mode.
///
/// `from` is the mode being left, passed in rather than read back from
/// the store: the store is written after the switch, so a settle task
/// reading it mid-switch sees the old value and gives up immediately.
pub fn apply(app: &tauri::AppHandle, from: Mode, mode: Mode) -> Result<(), String> {
    let window = main_window(app)?;

    match mode {
        Mode::Full => {
            // Leaving the pill: note where it was before the window
            // stops being one.
            if matches!(from, Mode::Minimized) {
                remember_pill_position(app, &window);
            }
            // Resizable FIRST, and on its own. While a window is not
            // resizable GTK pins its minimum and maximum size to the
            // current one, and the window manager then refuses both a
            // resize and a maximize -- so a size or maximize requested
            // in the same breath as this is simply dropped, and the
            // window stays pill-sized wearing the full layout.
            window.set_resizable(true).map_err(failed("resize"))?;
            window
                .set_always_on_top(false)
                .map_err(failed("release the window"))?;
            window
                .set_visible_on_all_workspaces(false)
                .map_err(failed("release the window"))?;
            window
                .set_skip_taskbar(false)
                .map_err(failed("show the window"))?;

            // Clear any stale maximized state before restoring a size:
            // the size a maximized window would "restore" to is still
            // the pill's, so unmaximizing later would shrink it again.
            window.unmaximize().map_err(failed("restore the window"))?;
            let (maximized, size, position) = stored_full_geometry(app);
            if let Some(size) = size {
                window.set_size(size).map_err(failed("restore the window"))?;
            }
            if let Some(position) = position {
                window
                    .set_position(position)
                    .map_err(failed("restore the window"))?;
            }
            if maximized || size.is_none() {
                // Maximized, not fullscreen: the desktop underneath stays
                // reachable, and a fullscreen window hides the system bar
                // the owner may need to get back to something else.
                window.maximize().map_err(failed("fill the screen"))?;
            }
        }
        Mode::Minimized => {
            // Before the window stops being full, so there is something
            // to come back to.
            if matches!(from, Mode::Full) {
                remember_full_geometry(app, &window);
            }
            window.set_resizable(true).map_err(failed("resize"))?;
            let was_maximized = window.is_maximized().unwrap_or(false);
            window.unmaximize().map_err(failed("shrink the window"))?;
            place_pill(app, &window)?;
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

            // The window is deliberately left resizable even though the
            // pill offers no way to resize it. Marking it fixed makes GTK
            // drop the size it was just given and adopt the webview's own
            // natural height instead -- measured at 200px against a 76px
            // request, so the pill would arrive nearly three times too
            // tall.

            // Unmaximizing is a request to the window manager, not an
            // instruction: it has not happened yet when the size above is
            // set, so the manager overwrites that size with the geometry
            // it is still holding and the pill never shrinks at all.
            // Measured: a maximized window asked to become a pill stayed
            // at full size. Ask again as it settles.
            settle_pill(app.clone(), was_maximized);
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
    let from = current(&app);
    // Recorded before the window moves, not after: the switch finishes
    // asynchronously, and anything still working on it needs to know
    // where the window is going rather than where it has been.
    let store = app
        .store(STORE_FILE)
        .map_err(|e| format!("could not open the settings store: {e}"))?;
    store.set(MODE_KEY, mode.as_str());
    store
        .save()
        .map_err(|e| format!("could not save the settings store: {e}"))?;
    apply(&app, from, mode)?;
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
/// Neither shape has a title bar to grab, so a surface of each one is
/// the handle: the pill's orb, and the top of the conversation pane.
#[tauri::command]
pub fn window_drag(app: tauri::AppHandle) -> Result<(), String> {
    main_window(&app)?
        .start_dragging()
        .map_err(|e| format!("could not move the window: {e}"))
}

/// Fills the screen, or gives it back. What double-clicking a title bar
/// does, for a window that has no title bar to double-click.
#[tauri::command]
pub fn window_toggle_maximize(app: tauri::AppHandle) -> Result<(), String> {
    let window = main_window(&app)?;
    window
        .is_maximized()
        .and_then(|maximized| {
            if maximized {
                window.unmaximize()
            } else {
                window.maximize()
            }
        })
        .map_err(|e| format!("could not resize the window: {e}"))
}

/// Hands one edge or corner to the window manager for the duration of a
/// press.
///
/// Undecorated windows get no resize border from the window manager --
/// that border is part of the decoration -- so without this the owner
/// can never change the window's size at all. The webview draws its own
/// thin edges and points them here.
#[tauri::command]
pub fn window_resize_drag(direction: String, app: tauri::AppHandle) -> Result<(), String> {
    use tauri_runtime::ResizeDirection;
    let direction = match direction.as_str() {
        "north" => ResizeDirection::North,
        "south" => ResizeDirection::South,
        "east" => ResizeDirection::East,
        "west" => ResizeDirection::West,
        "north-east" => ResizeDirection::NorthEast,
        "north-west" => ResizeDirection::NorthWest,
        "south-east" => ResizeDirection::SouthEast,
        "south-west" => ResizeDirection::SouthWest,
        // A direction this build does not know is not a direction to
        // guess at: dragging the wrong edge is worse than not dragging.
        other => return Err(format!("unknown resize direction: {other}")),
    };
    main_window(&app)?
        .as_ref()
        .window()
        .start_resize_dragging(direction)
        .map_err(|e| format!("could not resize the window: {e}"))
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

    /// A real device reached a state where the stored "full" size was
    /// the pill's own 420x76, so every start opened a window the owner
    /// had to expand by hand -- and each start recorded the bad size
    /// again. Both directions are guarded, and both are asserted.
    #[test]
    fn the_pills_geometry_is_never_mistaken_for_the_full_windows() {
        assert!(!is_plausible_full_size(&LogicalSize::new(
            PILL_WIDTH,
            PILL_HEIGHT
        )));
        // The shape that was actually found on disk.
        assert!(!is_plausible_full_size(&LogicalSize::new(420.0, 76.0)));
        // Narrow but real windows are still the owner's business.
        assert!(is_plausible_full_size(&LogicalSize::new(1024.0, 768.0)));
        assert!(is_plausible_full_size(&LogicalSize::new(
            MIN_FULL_WIDTH,
            MIN_FULL_HEIGHT
        )));
        // A window wide enough but pill-height is still not a window.
        assert!(!is_plausible_full_size(&LogicalSize::new(1600.0, 76.0)));
    }

    #[test]
    fn the_pill_grows_but_never_wider() {
        // The pane has a measure that suits bare prose; growth is
        // downward only, so a long reply never widens the window.
        assert!(PILL_HEIGHT_EXPANDED > PILL_HEIGHT);
        assert_eq!(PILL_WIDTH, 420.0);
    }
}
