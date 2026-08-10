<script setup lang="ts">
// One row: a folder to open, or a file to point the device at. Names are
// shown exactly as they are on disk, extension included -- this is the
// owner's own filesystem, not a curated view of it.
//
// Selection lives in the browser above, not here: a row reports what the
// owner did to it and renders what it is told, so the rules about what
// a modifier means live in one place.
import type { Entry } from "~/composables/useShelf";
import { fileSize, itemCount, relativeDate } from "~/lib/shelfErrors";

const { entry, selected = false, focused = false } = defineProps<{
  entry: Entry;
  selected?: boolean;
  focused?: boolean;
}>();

const emit = defineEmits<{
  select: [event: MouseEvent];
  open: [];
}>();

// Folders say what they hold; files say how big they are.
const detail = computed(() =>
  entry.isDir ? itemCount(entry.count) : fileSize(entry.size),
);
</script>

<template>
  <div
    class="row"
    :class="{ selected, focused }"
    role="option"
    :aria-selected="selected"
    :tabindex="focused ? 0 : -1"
    @click="emit('select', $event)"
    @dblclick="emit('open')"
  >
    <ContentMotif :kind="entry.kind" />
    <span class="name">{{ entry.name }}</span>
    <span class="detail">{{ detail }}</span>
    <span class="when">{{ relativeDate(entry.modified) }}</span>
  </div>
</template>

<style scoped>
.row {
  display: flex;
  align-items: center;
  gap: 1rem;
  height: 56px;
  padding: 0 1.25rem;
  border-bottom: 1px solid rgba(255, 255, 255, 0.04);
  cursor: default;
  /* A double-click that selects the row's text instead of opening the
     folder is the classic file-manager annoyance. */
  user-select: none;
}

.row:hover {
  background: color-mix(in srgb, var(--surface) 60%, transparent);
}

.selected {
  background: color-mix(in srgb, var(--accent) 14%, transparent);
}

.selected:hover {
  background: color-mix(in srgb, var(--accent) 18%, transparent);
}

/* Keyboard focus is shown as a ring rather than by changing the fill, so
   the owner can tell "where I am" apart from "what I picked" -- with
   multi-select those are genuinely different things. */
.focused {
  outline: 2px solid var(--accent);
  outline-offset: -2px;
}

.row:focus-visible {
  outline: 2px solid var(--accent);
  outline-offset: -2px;
}

.name {
  flex: 1;
  min-width: 0;
  color: var(--text-primary);
  font-size: 0.95rem;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.detail,
.when {
  color: var(--text-secondary);
  font-size: 0.82rem;
  white-space: nowrap;
}

.detail {
  width: 6rem;
  text-align: right;
}

.when {
  width: 7rem;
  text-align: right;
}
</style>
