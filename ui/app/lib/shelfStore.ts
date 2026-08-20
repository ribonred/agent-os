import { Store } from "@tauri-apps/plugin-store";

// Shell preferences that should survive a restart. Deliberately tiny:
// what the owner has on screen (their shelves and what's on them) is
// read from disk every time, because the filesystem is the truth and a
// cached copy guarantees a stale first frame.

const STORE_FILE = "settings.json";

let storePromise: ReturnType<typeof Store.load> | null = null;

function getStore() {
  if (!storePromise) {
    storePromise = Store.load(STORE_FILE);
  }
  return storePromise;
}

export async function getChatCollapsed(): Promise<boolean> {
  try {
    const store = await getStore();
    return (await store.get<boolean>("chatCollapsed")) ?? false;
  } catch {
    // A preference is not worth failing a screen over: an unreadable
    // store just means the pane opens in its default state.
    return false;
  }
}

export async function setChatCollapsed(collapsed: boolean): Promise<void> {
  try {
    const store = await getStore();
    await store.set("chatCollapsed", collapsed);
    await store.save();
  } catch {
    // Same reasoning -- the collapse still works for this session.
  }
}

export async function getFilesCollapsed(): Promise<boolean> {
  try {
    const store = await getStore();
    return (await store.get<boolean>("filesCollapsed")) ?? false;
  } catch {
    return false;
  }
}

export async function setFilesCollapsed(collapsed: boolean): Promise<void> {
  try {
    const store = await getStore();
    await store.set("filesCollapsed", collapsed);
    await store.save();
  } catch {
    // Same reasoning -- the collapse still works for this session.
  }
}

/// How the owner reads a view. Light by default: a view is a document,
/// and it is printed onto white paper -- see design/DESIGN.md. The shell
/// around it stays dark either way.
export type ViewTheme = "light" | "dark";

export async function getViewTheme(): Promise<ViewTheme> {
  try {
    const store = await getStore();
    return (await store.get<ViewTheme>("viewTheme")) === "dark" ? "dark" : "light";
  } catch {
    return "light";
  }
}

export async function setViewTheme(theme: ViewTheme): Promise<void> {
  try {
    const store = await getStore();
    await store.set("viewTheme", theme);
    await store.save();
  } catch {
    // The page is already showing the choice; only its memory is lost.
  }
}

export const DEFAULT_CHAT_WIDTH = 460;
export const MIN_CHAT_WIDTH = 320;
export const MAX_CHAT_WIDTH = 800;

export async function getChatWidth(): Promise<number> {
  try {
    const store = await getStore();
    const width = await store.get<number>("chatWidth");
    if (
      typeof width === "number" &&
      !Number.isNaN(width) &&
      width >= MIN_CHAT_WIDTH &&
      width <= MAX_CHAT_WIDTH
    ) {
      return width;
    }
    return DEFAULT_CHAT_WIDTH;
  } catch {
    return DEFAULT_CHAT_WIDTH;
  }
}

export async function setChatWidth(width: number): Promise<void> {
  try {
    const store = await getStore();
    await store.set("chatWidth", width);
    await store.save();
  } catch {
    // A failed write leaves the current width active on screen without
    // breaking the session.
  }
}
