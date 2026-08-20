import { getFilesCollapsed, setFilesCollapsed } from "~/lib/shelfStore";

// Whether the file view is folded away to a rail so the conversation can
// have the whole window.
//
// The shell layout draws the rail but the file view's own header holds
// the control that folds it, and those are a layout and a page component
// -- app-level state rather than props threaded through a slot, the same
// reasoning as the selection context.

export function useFilesPane() {
  const collapsed = useState<boolean>("files:collapsed", () => false);
  const loaded = useState<boolean>("files:collapsedLoaded", () => false);

  /// Read once per app start. Every mounted consumer calls this, so it
  /// guards itself rather than relying on which one mounts first.
  async function restore() {
    if (loaded.value) return;
    loaded.value = true;
    collapsed.value = await getFilesCollapsed();
  }

  async function set(value: boolean) {
    collapsed.value = value;
    await setFilesCollapsed(value);
  }

  async function toggle() {
    await set(!collapsed.value);
  }

  return { collapsed, restore, set, toggle };
}
