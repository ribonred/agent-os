import type { Entry } from "~/composables/useShelf";

// What the owner has selected, as the conversation sees it.
//
// The file view and the conversation pane are siblings under the shell,
// so this is app-level state rather than props threaded through the
// layout. It holds the minimum the chip needs to render and the turn
// needs to send -- names for the owner, paths for the native layer,
// which turns them back into plain language before the model sees them.

export type ContextItem = { path: string; name: string; isDir: boolean };

export function useContext() {
  const items = useState<ContextItem[]>("context:items", () => []);
  // Where they are reading, sent on every turn rather than only when
  // something is selected: "what is in here?" is a question about the
  // folder, and it is the one the owner asks without selecting anything
  // first. Empty means home, which is deliberately never named.
  const folder = useState<string>("context:folder", () => "");

  function set(entries: Entry[]) {
    items.value = entries.map((e) => ({
      path: e.path,
      name: e.name,
      isDir: e.isDir,
    }));
  }

  function clear() {
    items.value = [];
  }

  function remove(path: string) {
    items.value = items.value.filter((i) => i.path !== path);
  }

  const paths = computed(() => items.value.map((i) => i.path));

  return { items, folder, paths, set, clear, remove };
}
