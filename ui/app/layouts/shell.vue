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

onMounted(async () => {
  collapsed.value = await getChatCollapsed();
  await restoreFilesPane();
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
         it, instead of a re-read of their disk. -->
    <main :class="['main', { folded: filesCollapsed }]">
      <slot />
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

<style scoped>
.shell {
  display: flex;
  height: 100vh;
  overflow: hidden;
}

.main {
  flex: 1;
  min-width: 0;
  height: 100vh;
  overflow-y: auto;
}

.main.folded {
  display: none;
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
