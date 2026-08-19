<script setup lang="ts">
// One conversation in the list: what it was called, and a line of what
// it was about.
//
// Rename and delete sit behind a menu rather than on the row. This is a
// list of things that cannot be got back, and it is a list the owner
// taps constantly -- a delete control a few pixels from the tap target
// is a trap, not an affordance.

import { conversationName, type Conversation } from "~/lib/sessionList";

const { conversation, current = false, disabled = false } = defineProps<{
  conversation: Conversation;
  current?: boolean;
  disabled?: boolean;
}>();

const emit = defineEmits<{
  open: [];
  rename: [title: string];
  keep: [kept: boolean];
  remove: [];
}>();

const menuOpen = ref(false);
const renaming = ref(false);
const confirming = ref(false);
const draft = ref("");
const field = ref<HTMLInputElement | null>(null);

const name = computed(() => conversationName(conversation));

function closeMenu() {
  menuOpen.value = false;
}

async function startRename() {
  closeMenu();
  // The device's own name for it, there to edit rather than an empty
  // box: most renames are a small correction to something close.
  draft.value = conversation.title.trim();
  renaming.value = true;
  await nextTick();
  field.value?.focus();
  field.value?.select();
}

function commitRename() {
  const title = draft.value.trim();
  renaming.value = false;
  if (title === "" || title === conversation.title.trim()) return;
  emit("rename", title);
}
</script>

<template>
  <li :class="['row', { current, renaming, confirming }]">
    <form v-if="renaming" class="rename" @submit.prevent="commitRename">
      <input
        ref="field"
        v-model="draft"
        type="text"
        maxlength="80"
        aria-label="Name for this conversation"
        @blur="commitRename"
        @keydown.esc="renaming = false"
      />
    </form>

    <!-- Named, and in place of the row rather than over it. A question
         about something the owner cannot see while answering it is not a
         question they can answer. -->
    <div v-else-if="confirming" class="confirm">
      <p>Delete “{{ name }}”? Everything said in it goes with it.</p>
      <div class="answers">
        <button type="button" class="danger" @click="confirming = false; emit('remove')">
          Delete it
        </button>
        <button type="button" @click="confirming = false">Keep it</button>
      </div>
    </div>

    <template v-else>
      <button
        type="button"
        class="open"
        :disabled="disabled"
        :aria-current="current ? 'true' : undefined"
        @click="emit('open')"
      >
        <span class="name">
          <span v-if="conversation.kept" class="kept" aria-label="Kept">◆</span>
          {{ name }}
        </span>
        <span v-if="conversation.preview" class="about">
          {{ conversation.preview }}
        </span>
      </button>

      <button
        type="button"
        class="more"
        :aria-expanded="menuOpen"
        aria-label="More for this conversation"
        @click="menuOpen = !menuOpen"
      >
        ⋯
      </button>
    </template>

    <!-- Anywhere else closes it, so the menu never sits open behind
         something the owner has moved on to. -->
    <div v-if="menuOpen" class="scrim" @click="closeMenu" />

    <ul v-if="menuOpen" class="menu">
      <li>
        <button type="button" @click="startRename">Rename</button>
      </li>
      <li>
        <button
          type="button"
          @click="
            closeMenu();
            emit('keep', !conversation.kept);
          "
        >
          {{ conversation.kept ? "Stop keeping this" : "Keep this one" }}
        </button>
      </li>
      <li>
        <button
          type="button"
          class="remove"
          @click="
            closeMenu();
            confirming = true;
          "
        >
          Delete
        </button>
      </li>
    </ul>
  </li>
</template>

<style scoped>
.row {
  position: relative;
  display: flex;
  align-items: flex-start;
  gap: 0.25rem;
  border-radius: 10px;
}

.row:hover,
.row:focus-within {
  background: var(--surface);
}

/* The one being read. Marked on the edge rather than by filling the row:
   a filled row and a hovered row would be the same thing, and the owner
   needs to be able to tell which is which. */
.current::before {
  content: "";
  position: absolute;
  left: 0;
  top: 0.5rem;
  bottom: 0.5rem;
  width: 2px;
  border-radius: 2px;
  background: var(--accent);
}

.open {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 0.15rem;
  padding: 0.5rem 0.25rem 0.5rem 0.7rem;
  background: none;
  border: 0;
  text-align: left;
  cursor: pointer;
  color: inherit;
  font: inherit;
}

.open:disabled {
  cursor: default;
  opacity: 0.45;
}

.name {
  display: flex;
  align-items: baseline;
  gap: 0.35rem;
  color: var(--text-primary);
  font-size: 0.9rem;
  line-height: 1.3;
  /* One line: a wrapped title turns an even list into a ragged one and
     the second line is never the part that identifies it. */
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.kept {
  flex: 0 0 auto;
  color: var(--accent-warm);
  font-size: 0.6rem;
}

.about {
  color: var(--text-secondary);
  font-size: 0.78rem;
  line-height: 1.35;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.more {
  flex: 0 0 auto;
  margin: 0.35rem 0.25rem 0 0;
  padding: 0.15rem 0.4rem;
  background: none;
  border: 0;
  border-radius: 6px;
  color: var(--text-secondary);
  font-size: 1rem;
  line-height: 1;
  cursor: pointer;
  opacity: 0;
}

.row:hover .more,
.row:focus-within .more,
.more[aria-expanded="true"] {
  opacity: 1;
}

.more:hover {
  color: var(--text-primary);
}

.scrim {
  position: fixed;
  inset: 0;
  z-index: 1;
}

.menu {
  position: absolute;
  right: 0.4rem;
  top: 2.2rem;
  z-index: 2;
  margin: 0;
  padding: 0.25rem;
  list-style: none;
  min-width: 11rem;
  background: var(--surface-raised);
  border: 1px solid color-mix(in srgb, var(--text-secondary) 20%, transparent);
  border-radius: 10px;
  box-shadow: 0 12px 28px rgb(0 0 0 / 45%);
}

.menu button {
  width: 100%;
  padding: 0.45rem 0.6rem;
  background: none;
  border: 0;
  border-radius: 7px;
  text-align: left;
  color: var(--text-primary);
  font: inherit;
  font-size: 0.85rem;
  cursor: pointer;
}

.menu button:hover {
  background: var(--surface);
}

.menu .remove {
  color: var(--danger);
}

.rename {
  flex: 1;
  padding: 0.35rem 0.4rem 0.35rem 0.7rem;
}

.rename input {
  width: 100%;
  padding: 0.3rem 0.45rem;
  background: var(--bg);
  border: 1px solid var(--accent);
  border-radius: 7px;
  color: var(--text-primary);
  font: inherit;
  font-size: 0.9rem;
}

.rename input:focus {
  outline: none;
}

.confirm {
  flex: 1;
  min-width: 0;
  padding: 0.7rem;
  background: var(--surface-raised);
  border: 1px solid color-mix(in srgb, var(--danger) 40%, transparent);
  border-radius: 10px;
}

.confirm p {
  margin: 0 0 0.6rem;
  overflow-wrap: anywhere;
  color: var(--text-primary);
  font-size: 0.83rem;
  line-height: 1.4;
}

.answers {
  display: flex;
  gap: 0.4rem;
}

.answers button {
  padding: 0.35rem 0.7rem;
  background: var(--surface);
  border: 1px solid transparent;
  border-radius: 7px;
  color: var(--text-primary);
  font: inherit;
  font-size: 0.82rem;
  cursor: pointer;
}

.answers .danger {
  border-color: color-mix(in srgb, var(--danger) 55%, transparent);
  color: var(--danger);
}
</style>
