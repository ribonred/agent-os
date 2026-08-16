<script setup lang="ts">
// The file view. One component for every directory including home, so
// browsing into a folder is the same screen with a different path rather
// than a separate mode.
import type { Entry } from "~/composables/useShelf";
import { useBrowser } from "~/composables/useShelf";
import { useSelection } from "~/composables/useSelection";

const { path } = defineProps<{ path: string }>();
const current = computed(() => path);

const { listing, loading, error, load } = useBrowser(current);
const entries = computed(() => listing.value?.entries ?? []);

const {
  selected,
  selectedEntries,
  focused,
  focusedEntry,
  clear,
  handleClick,
  moveFocus,
} = useSelection(entries);

// What is selected is what the conversation will ask about, so the two
// stay in step rather than needing a separate "use this" gesture.
const context = useContext();
watch(selectedEntries, (list) => context.set(list), { deep: true });

// Dismissing something from the chip has to deselect the row too, or the
// file view would keep showing a row as picked after the owner said they
// didn't mean it. The chip is the authority in this direction.
watch(
  context.paths,
  (paths) => {
    if (paths.length === selected.value.size) return;
    const kept = new Set(paths.filter((p) => selected.value.has(p)));
    if (kept.size !== selected.value.size) selected.value = kept;
  },
  { deep: true },
);

// A new directory is a fresh start: carrying a selection across a
// navigation would leave rows picked that the owner can no longer see.
watch(current, () => {
  clear();
  context.clear();
});

// The agent may have put something down, moved something, or downloaded
// a file during its turn -- re-read when it finishes rather than leaving
// the owner looking at a stale directory.
const { busy } = useConversation();
watch(busy, (isBusy, was) => {
  if (was && !isBusy) load();
});

function open(entry: Entry) {
  // Only folders go anywhere for now. Opening a file is what the
  // conversation is for, and that lands with the context chip.
  if (entry.isDir) navigateTo(`/files/${entry.path}`);
}

function onKeydown(event: KeyboardEvent) {
  switch (event.key) {
    case "ArrowDown":
      event.preventDefault();
      moveFocus(1, event.shiftKey);
      break;
    case "ArrowUp":
      event.preventDefault();
      moveFocus(-1, event.shiftKey);
      break;
    case "Enter":
      if (focusedEntry.value) {
        event.preventDefault();
        open(focusedEntry.value);
      }
      break;
    case "Escape":
      clear();
      break;
  }
}

// Where "up" goes: the parent crumb, or home.
const parentPath = computed(() => {
  const crumbs = listing.value?.crumbs ?? [];
  if (crumbs.length === 0) return null;
  return crumbs.length === 1 ? "" : (crumbs[crumbs.length - 2]?.path ?? "");
});

const isEmpty = computed(
  () => !loading.value && !error.value && entries.value.length === 0,
);
</script>

<template>
  <section class="files">
    <header class="bar">
      <NuxtLink
        v-if="parentPath !== null"
        class="back"
        :to="parentPath === '' ? '/' : `/files/${parentPath}`"
        aria-label="Back"
      >
        ‹
      </NuxtLink>

      <!-- The breadcrumb is the only place a location is spelled out, and
           it is named folders rather than a filesystem path: no leading
           slash, no home directory, nothing above where the owner can go. -->
      <nav class="crumbs" aria-label="Location">
        <NuxtLink class="crumb" to="/">Your files</NuxtLink>
        <template v-for="crumb in listing?.crumbs ?? []" :key="crumb.path">
          <span class="sep">›</span>
          <NuxtLink class="crumb" :to="`/files/${crumb.path}`">
            {{ crumb.name }}
          </NuxtLink>
        </template>
      </nav>

      <span v-if="selected.size > 1" class="count">
        {{ selected.size }} selected
      </span>
      <span v-else-if="!loading && !error" class="count">
        {{ entries.length }}
      </span>
    </header>

    <p v-if="error" class="surface-error" role="alert">
      {{ error }}
      <button type="button" @click="load">Try again</button>
    </p>

    <div v-else-if="isEmpty" class="empty enter-fade">
      <p class="lead">This folder is empty.</p>
      <p class="hint">Put something here, or ask me to.</p>
    </div>

    <!-- Clicking past the last row clears the selection, the way an empty
         area does in any file manager. -->
    <div
      v-else-if="!loading"
      class="rows enter-fade"
      role="listbox"
      aria-multiselectable="true"
      aria-label="Files"
      tabindex="0"
      @keydown="onKeydown"
      @click.self="clear"
    >
      <FileRow
        v-for="entry in entries"
        :key="entry.path"
        :entry="entry"
        :selected="selected.has(entry.path)"
        :focused="focused === entry.path"
        @select="handleClick(entry.path, $event)"
        @open="open(entry)"
      />
    </div>
  </section>
</template>

<style scoped>
.files {
  display: flex;
  flex-direction: column;
  min-height: 100vh;
}

.bar {
  display: flex;
  align-items: center;
  gap: 0.6rem;
  height: 56px;
  padding: 0 1.75rem;
  border-bottom: 1px solid rgba(255, 255, 255, 0.05);
}

.back {
  color: var(--text-secondary);
  font-size: 1.5rem;
  line-height: 1;
  text-decoration: none;
}

.back:hover,
.back:focus-visible {
  color: var(--text-primary);
}

.crumbs {
  flex: 1;
  min-width: 0;
  display: flex;
  align-items: center;
  gap: 0.4rem;
  overflow: hidden;
}

.crumb {
  color: var(--text-secondary);
  font-size: 0.95rem;
  text-decoration: none;
  white-space: nowrap;
}

/* The directory you are actually in is the one that reads as current. */
.crumb:last-of-type {
  color: var(--text-primary);
}

.crumb:hover,
.crumb:focus-visible {
  color: var(--text-primary);
}

.sep {
  color: var(--text-secondary);
  font-size: 0.8rem;
  opacity: 0.6;
}

.count {
  color: var(--text-secondary);
  font-size: 0.82rem;
  white-space: nowrap;
}

.rows {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-content: flex-start;
  padding-bottom: 2rem;
}

.rows:focus {
  outline: none;
}

.empty {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 0.5rem;
  padding: 2rem;
  text-align: center;
}

.lead {
  margin: 0;
  color: var(--text-primary);
  font-size: 1.05rem;
  font-weight: 300;
}

.hint {
  margin: 0;
  color: var(--text-secondary);
  font-size: 0.92rem;
}

/* An error about the whole surface replaces the content, in the same
   voice -- never a toast, which is a message the owner didn't read. */
.surface-error {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 0.9rem;
  margin: 0;
  color: var(--danger);
  font-size: 0.95rem;
}

.surface-error button {
  background: var(--surface);
  color: var(--text-primary);
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 10px;
  padding: 0.5rem 1rem;
  font-family: var(--font-family);
  font-size: 0.88rem;
  cursor: pointer;
}

.surface-error button:hover {
  background: var(--surface-raised);
}

.enter-fade {
  animation: fade-in 0.35s both;
}

@keyframes fade-in {
  from {
    opacity: 0;
  }
  to {
    opacity: 1;
  }
}

@media (prefers-reduced-motion: reduce) {
  .enter-fade {
    animation: none;
  }
}
</style>
