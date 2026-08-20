import { invoke } from "@tauri-apps/api/core";
import { shelfErrorMessage } from "~/lib/shelfErrors";

// Browsing state. Nothing is persisted: the filesystem is the truth, and
// a cached listing only guarantees a stale first frame. Every directory
// is re-read when it is opened and after the agent finishes a turn --
// it may have put something down.

export type EntryKind =
  | "folder"
  /// A folder holding a page the device built. Marked apart because the
  /// row opens the page rather than the directory.
  | "view"
  | "text"
  | "table"
  | "image"
  | "document"
  | "audio"
  | "video"
  | "archive"
  | "file";

export type Entry = {
  /// Relative to home; also the id used to navigate into a directory.
  path: string;
  name: string;
  isDir: boolean;
  count: number;
  size: number;
  modified: number;
  kind: EntryKind;
};

export type Crumb = { path: string; name: string };

export type Listing = {
  path: string;
  crumbs: Crumb[];
  entries: Entry[];
};

export function useBrowser(path: Ref<string>) {
  const listing = ref<Listing | null>(null);
  const loading = ref(true);
  const error = ref<string | null>(null);

  async function load() {
    loading.value = true;
    error.value = null;
    try {
      listing.value = await invoke<Listing>("shelf_list", { path: path.value });
    } catch (e) {
      error.value = shelfErrorMessage(e);
      listing.value = null;
    } finally {
      loading.value = false;
    }
  }

  watch(path, load, { immediate: true });

  return { listing, loading, error, load };
}
