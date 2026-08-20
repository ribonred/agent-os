<script setup lang="ts">
// The two-pane shell: the conversation on the left, the owner's things on
// the right. This is a layout rather than a page for one load-bearing
// reason -- Nuxt keeps a layout instance alive across routes that share
// it, so the conversation pane does not remount when the main area
// changes and a reply keeps streaming while the owner looks around.
import { getChatCollapsed, setChatCollapsed } from "~/lib/shelfStore";

// Minimized, the whole shell is the pill: the file view has no room and
// no purpose at that size, and the conversation is the only thing the
// owner came back for. The page underneath keeps its state, so the way
// back is a resize rather than a reload.
const { mode } = useWindowMode();

const collapsed = ref(false);

// Either half can give way to the other: folded away, the file view is a
// rail the owner taps to get it back, and the conversation takes the
// whole window rather than sitting at its usual measure beside a gap.
const { collapsed: filesCollapsed, restore: restoreFilesPane, set: setFilesCollapsed } =
  useFilesPane();

// Folders and Views are tabs on the one pane, not a third column -- the
// device sits beside the owner's work, and permanently narrowing that
// work to hold a second surface costs more than it gives.
//
// The tab strip only appears once the device has actually made
// something. A unit on its first day has made nothing, and shipping
// every device with an empty tab advertises a feature instead of
// offering one.
const { views, tab, load: loadViews } = useViews();
const hasViews = computed(() => views.value.length > 0);

// Falling back rather than stranding the owner on a tab that no longer
// has anything behind it.
watch(hasViews, (any) => {
  if (!any) tab.value = "folders";
});

onMounted(async () => {
  collapsed.value = await getChatCollapsed();
  await restoreFilesPane();
  await loadViews();
  // A screen where both halves are folded away has nothing on it. If a
  // stored pair ever disagrees, the files give way -- the conversation is
  // what the device is for.
  if (collapsed.value && filesCollapsed.value) await setFilesCollapsed(false);
});

async function setCollapsed(value: boolean) {
  collapsed.value = value;
  await setChatCollapsed(value);
  // Hiding the conversation is a request for the files, so they come
  // back rather than leaving the owner facing two rails.
  if (value && filesCollapsed.value) await setFilesCollapsed(false);
}
</script>

<template>
  <PillShell v-if="mode === 'minimized'" />
  <div v-else class="shell">
    <!-- The window has no decoration, so it draws its own resize
         border. The pill has none: it sizes itself to what there is to
         read, and a hand-resized pill would fight that. -->
    <WindowEdges />
    <ConversationPane
      :collapsed="collapsed"
      :fill="filesCollapsed"
      @update:collapsed="setCollapsed"
    />
    <!-- Kept mounted while folded away rather than torn down: the owner
         gets back the directory they were in, scrolled where they left
         it, instead of a re-read of their disk. The same reasoning keeps
         both tabs mounted and merely hidden. -->
    <main :class="['main', { folded: filesCollapsed }]">
      <nav v-if="hasViews" class="tabs" aria-label="What to look at">
        <button
          type="button"
          :class="['tab', { on: tab === 'folders' }]"
          :aria-current="tab === 'folders'"
          @click="tab = 'folders'"
        >
          Folders
        </button>
        <button
          type="button"
          :class="['tab', { on: tab === 'views' }]"
          :aria-current="tab === 'views'"
          @click="tab = 'views'"
        >
          Views
        </button>
      </nav>

      <div v-show="tab === 'folders'" class="surface">
        <slot />
      </div>
      <div v-show="hasViews && tab === 'views'" class="surface">
        <ViewPane />
      </div>
    </main>
    <button
      v-if="filesCollapsed"
      type="button"
      class="files-rail"
      aria-label="Show your files"
      title="Show your files"
      :aria-expanded="false"
      @click="setFilesCollapsed(false)"
    >
      <ContentMotif kind="folder" />
    </button>
  </div>
</template>

<!-- Deliberately not scoped: it has to match a class on <body> and a
     frame inside a child component in the same rule. -->
<style>
/* While the divider is being dragged, frames stop taking the pointer.
   A view is a separate document, and it would otherwise swallow the
   drag the moment the cursor crossed into it. */
body.resizing-panes iframe {
  pointer-events: none;
}
</style>

<style scoped>
.shell {
  display: flex;
  height: 100vh;
  overflow: hidden;
}

.main {
  flex: 1;
  min-width: 0;
  /* A column, because the tab strip sits above the surface: sizing a
     child to the viewport instead would push its last row off-screen by
     exactly the strip's height. Scrolling belongs to the surface. */
  display: flex;
  flex-direction: column;
  height: 100vh;
  overflow: hidden;
}

.main.folded {
  display: none;
}

/* The tab strip is chrome, so it is quiet: two words, no box, and the
   current one reads as current by weight rather than by decoration. */
.tabs {
  display: flex;
  gap: 0.25rem;
  padding: 0.6rem 1.75rem 0;
}

.tab {
  background: none;
  border: none;
  border-radius: 8px;
  padding: 0.3rem 0.7rem;
  color: var(--text-secondary);
  font-family: var(--font-family);
  font-size: 0.88rem;
  cursor: pointer;
}

.tab:hover {
  color: var(--text-primary);
}

.tab.on {
  background: var(--surface);
  color: var(--text-primary);
}

.tab:focus-visible {
  outline: 2px solid var(--accent);
  outline-offset: -2px;
}

.surface {
  flex: 1 1 0;
  min-height: 0;
  overflow-y: auto;
}

.tabs {
  flex: 0 0 auto;
}

/* Printing a view means printing the view, not the room around it. The
   frame has no same-origin, so it cannot be told to print itself --
   this window prints, and everything that is not the page gets out of
   the way. */
@media print {
  .shell {
    display: block;
    height: auto;
    overflow: visible;
  }

  /* The conversation, the resize border and the tab strip are the room;
     none of them belong on paper. `.pane` is reachable from here because
     a component's root element carries its parent's scope. */
  .pane,
  .tabs,
  .files-rail {
    display: none !important;
  }

  .main.folded {
    display: block;
  }
}

/* The way back, and the whole width of it is the target -- the same rail
   the conversation folds down to, on the other side of the window. */
.files-rail {
  flex: 0 0 56px;
  width: 56px;
  display: grid;
  place-items: center;
  height: 100vh;
  padding: 0;
  background: color-mix(in srgb, var(--bg) 92%, black);
  border-left: 1px solid rgba(255, 255, 255, 0.05);
  border: none;
  cursor: pointer;
}

/* The mark is the child's, so the hover has to reach through the wrapper
   rather than sitting on the button's own colour. */
.files-rail:hover :deep(.motif) {
  color: var(--text-primary);
}

.files-rail:focus-visible {
  outline: 2px solid var(--accent);
  outline-offset: -2px;
}
</style>
