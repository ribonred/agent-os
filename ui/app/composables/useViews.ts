import { invoke } from "@tauri-apps/api/core";
import { shelfErrorMessage } from "~/lib/shelfErrors";

// The views the device has built, and which one is open.
//
// Nothing is cached, for the same reason browsing isn't: the folder is
// the truth. The agent may have built one, rewritten one, or the owner
// may have deleted one from their file view -- so this re-reads rather
// than trusting what it saw last.
//
// The list is app-level state because the tab strip, the pane, and the
// conversation all need it; loading it once per turn rather than once
// per component is the point.

export type View = {
  /// Folder name, and the id the pane addresses it by.
  name: string;
  title: string;
  /// The question it was built to answer, in the owner's own words.
  asked: string | null;
  /// Where its figures came from, named the way the owner names files.
  from: string[];
  modified: number;
};

export function useViews() {
  const views = useState<View[]>("views:list", () => []);
  const open = useState<string | null>("views:open", () => null);
  const error = useState<string | null>("views:error", () => null);
  /// Which half of the pane is showing. Here rather than in the layout
  /// because opening a view is something the file browser does too --
  /// a view row there shows the page rather than the folder.
  const tab = useState<"folders" | "views">("pane:tab", () => "folders");

  async function load() {
    try {
      views.value = await invoke<View[]>("views_list");
      error.value = null;
      // A view the owner deleted must not leave the pane showing a page
      // that no longer exists.
      if (open.value && !views.value.some((v) => v.name === open.value)) {
        open.value = views.value[0]?.name ?? null;
      }
    } catch (e) {
      error.value = shelfErrorMessage(e);
      views.value = [];
    }
  }

  const current = computed(
    () => views.value.find((v) => v.name === open.value) ?? null,
  );

  function show(name: string) {
    open.value = name;
    tab.value = "views";
  }

  return { views, open, tab, current, error, load, show };
}
