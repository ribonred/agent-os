<script setup lang="ts">
// What the owner has selected, shown just above the input so it is
// obvious the next question is about it (design/DESIGN.md, "Context
// comes from selection, not from attaching").
//
// Not an attachment control: there is no picker and no dialog, and
// nothing can appear here that isn't already visible on the file view.
import { useContext } from "~/composables/useContext";

const { items, clear, remove } = useContext();

// Several selected things collapse to one chip: a column of them would
// push the conversation off-screen, and the owner can already see the
// full selection on the other side of the window.
const COLLAPSE_AT = 2;
</script>

<template>
  <div v-if="items.length > 0" class="context">
    <template v-if="items.length < COLLAPSE_AT">
      <div v-for="item in items" :key="item.path" class="chip">
        <ContentMotif :kind="item.isDir ? 'folder' : 'file'" />
        <span class="name">{{ item.name }}</span>
        <button
          type="button"
          class="drop"
          :aria-label="`Don't ask about ${item.name}`"
          @click="remove(item.path)"
        >
          ✕
        </button>
      </div>
    </template>

    <div v-else class="chip">
      <ContentMotif kind="file" />
      <span class="name">{{ items.length }} things selected</span>
      <button
        type="button"
        class="drop"
        aria-label="Don't ask about the selection"
        @click="clear"
      >
        ✕
      </button>
    </div>
  </div>
</template>

<style scoped>
.context {
  display: flex;
  flex-direction: column;
  gap: 0.4rem;
  padding: 0 1.5rem 0.5rem;
}

.chip {
  display: flex;
  align-items: center;
  gap: 0.55rem;
  padding: 0.45rem 0.6rem 0.45rem 0.7rem;
  background: var(--surface);
  border: 1px solid color-mix(in srgb, var(--accent) 22%, transparent);
  border-radius: 10px;
}

.name {
  flex: 1;
  min-width: 0;
  color: var(--text-secondary);
  font-size: 0.85rem;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.drop {
  flex: 0 0 auto;
  background: none;
  border: none;
  color: var(--text-secondary);
  font-size: 0.8rem;
  line-height: 1;
  padding: 0.2rem 0.3rem;
  cursor: pointer;
}

.drop:hover {
  color: var(--text-primary);
}

.drop:focus-visible {
  outline: 2px solid var(--accent);
  outline-offset: 1px;
  border-radius: 4px;
}
</style>
