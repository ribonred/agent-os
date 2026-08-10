import type { Entry } from "~/composables/useShelf";

// Selection for a list of rows, with the gestures people already have in
// their hands from every other file manager:
//
//   click            select just this one
//   ctrl/cmd+click   add or remove this one
//   shift+click      select the range from the anchor to here
//   double-click     open (handled by the caller -- opening is not
//                    selection, and a folder and a file open differently)
//
// Kept apart from the browser so the rules live in one readable place
// rather than spread through click handlers.

export function useSelection(entries: Ref<Entry[]>) {
  // Paths rather than indices: a refresh can reorder or drop rows, and a
  // selection that silently moves to a different file is worse than one
  // that disappears.
  const selected = ref<Set<string>>(new Set());

  // Where a shift-range measures from, and where the keyboard is.
  const anchor = ref<string | null>(null);
  const focused = ref<string | null>(null);

  const selectedEntries = computed(() =>
    entries.value.filter((e) => selected.value.has(e.path)),
  );

  function clear() {
    selected.value = new Set();
    anchor.value = null;
    focused.value = null;
  }

  function selectOnly(path: string) {
    selected.value = new Set([path]);
    anchor.value = path;
    focused.value = path;
  }

  function toggle(path: string) {
    const next = new Set(selected.value);
    if (next.has(path)) {
      next.delete(path);
    } else {
      next.add(path);
    }
    selected.value = next;
    // The anchor follows the last row the owner actually touched, so a
    // shift-range afterwards measures from where they think it does.
    anchor.value = path;
    focused.value = path;
  }

  function selectRange(path: string) {
    const from = entries.value.findIndex((e) => e.path === anchor.value);
    const to = entries.value.findIndex((e) => e.path === path);
    if (to === -1) return;
    if (from === -1) {
      selectOnly(path);
      return;
    }
    const [start, end] = from <= to ? [from, to] : [to, from];
    const next = new Set<string>();
    for (let i = start; i <= end; i += 1) {
      const entry = entries.value[i];
      if (entry) next.add(entry.path);
    }
    selected.value = next;
    // The anchor stays put: dragging a range back and forth with shift
    // should keep measuring from the same origin.
    focused.value = path;
  }

  /// One entry point for a click, so the modifier rules are not
  /// re-decided at each call site.
  function handleClick(path: string, event: MouseEvent) {
    if (event.shiftKey) {
      selectRange(path);
    } else if (event.ctrlKey || event.metaKey) {
      toggle(path);
    } else {
      selectOnly(path);
    }
  }

  /// Move the keyboard focus, optionally extending the selection the way
  /// shift+arrow does elsewhere.
  function moveFocus(delta: number, extend = false) {
    if (entries.value.length === 0) return;
    const current = entries.value.findIndex((e) => e.path === focused.value);
    const next = Math.max(
      0,
      Math.min(entries.value.length - 1, current === -1 ? 0 : current + delta),
    );
    const entry = entries.value[next];
    if (!entry) return;
    if (extend) {
      if (anchor.value === null) anchor.value = focused.value ?? entry.path;
      selectRange(entry.path);
    } else {
      selectOnly(entry.path);
    }
  }

  const focusedEntry = computed(
    () => entries.value.find((e) => e.path === focused.value) ?? null,
  );

  // A listing that no longer contains a selected path has had that file
  // moved, renamed or deleted -- drop it rather than keeping a selection
  // that points at nothing.
  watch(entries, (list) => {
    const present = new Set(list.map((e) => e.path));
    const kept = [...selected.value].filter((p) => present.has(p));
    if (kept.length !== selected.value.size) {
      selected.value = new Set(kept);
    }
    if (focused.value && !present.has(focused.value)) focused.value = null;
    if (anchor.value && !present.has(anchor.value)) anchor.value = null;
  });

  return {
    selected,
    selectedEntries,
    focused,
    focusedEntry,
    clear,
    selectOnly,
    handleClick,
    moveFocus,
  };
}
