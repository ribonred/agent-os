import { invoke } from "@tauri-apps/api/core";

// Which shape the window is in, and the growing and settling the pill
// does while an exchange is happening.
//
// The geometry itself lives in the native layer -- this only says which
// mode is wanted and lets the window follow. Keeping the two apart means
// the webview never has to be trusted with moving its own window, and
// there is one place that knows what each mode means.

export type WindowMode = "full" | "minimized";

export type ResizeDirection =
  | "north"
  | "south"
  | "east"
  | "west"
  | "north-east"
  | "north-west"
  | "south-east"
  | "south-west";

export function useWindowMode() {
  const mode = useState<WindowMode>("window:mode", () => "full");
  /// Set once a reply has arrived, so the pill stays open long enough to
  /// be read rather than closing the moment the device stops talking.
  const holdingOpen = useState<boolean>("window:holdingOpen", () => false);

  async function load() {
    try {
      mode.value = (await invoke<string>("window_mode_get")) as WindowMode;
    } catch {
      // A window whose mode cannot be read is still a usable window.
      mode.value = "full";
    }
  }

  async function set(next: WindowMode) {
    if (next === "full") holdingOpen.value = false;
    try {
      mode.value = (await invoke<string>("window_mode_set", {
        mode: next,
      })) as WindowMode;
    } catch {
      // Leave the mode as it was: a frontend that claims it minimized
      // while the window did not is worse than one that did nothing.
    }
  }

  async function toggle() {
    await set(mode.value === "full" ? "minimized" : "full");
  }

  /// Grows the pill while there is something to read and settles it back
  /// when there isn't. A no-op in full mode, so callers never have to
  /// ask what shape the window is first.
  async function expand(expanded: boolean) {
    try {
      await invoke("window_pill_expand", { expanded });
    } catch {
      // Nothing to say: the pill is the wrong size, not broken.
    }
  }

  /// Neither shape has a title bar to grab, so a surface of each one is
  /// the handle: the pill's orb, and the top of the conversation pane.
  async function drag() {
    try {
      await invoke("window_drag");
    } catch {
      // A window that refused to be dragged is still where it was.
    }
  }

  /// What double-clicking a title bar does, for a window with no title
  /// bar to double-click.
  async function toggleMaximize() {
    try {
      await invoke("window_toggle_maximize");
    } catch {
      // The window is the size it was; nothing to tell the owner.
    }
  }

  /// One edge or corner, handed to the window manager for the duration
  /// of a press. An undecorated window has no resize border of its own.
  async function resizeDrag(direction: ResizeDirection) {
    try {
      await invoke("window_resize_drag", { direction });
    } catch {
      // Same: the window simply stays the size it was.
    }
  }

  return {
    mode,
    holdingOpen,
    load,
    set,
    toggle,
    expand,
    drag,
    toggleMaximize,
    resizeDrag,
  };
}
