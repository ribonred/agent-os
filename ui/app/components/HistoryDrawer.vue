<script setup lang="ts">
// The conversations the owner can go back to.
//
// It covers the conversation column rather than opening beside it: the
// device is the assistant next to the owner's work, and a list of past
// chats is not worth permanently narrowing that work to hold. Covering
// also means this reads the same at every window size instead of being
// a layout that only works maximized.

import { invoke } from "@tauri-apps/api/core";
import { agentErrorMessage } from "~/lib/agentErrors";
import {
  filterConversations,
  groupConversations,
  type Conversation,
} from "~/lib/sessionList";

const { current = null, disabled = false } = defineProps<{
  current?: string | null;
  disabled?: boolean;
}>();

const emit = defineEmits<{ close: []; open: [id: string] }>();

const { listSessions } = useConversation();

const conversations = ref<Conversation[]>([]);
const hasMore = ref(false);
const loading = ref(true);
const failure = ref<string | null>(null);
const query = ref("");

const groups = computed(() =>
  groupConversations(filterConversations(conversations.value, query.value)),
);

/// Nothing to go back to yet, as opposed to nothing matching what was
/// typed -- the same empty list, two different things to say about it.
const nothingYet = computed(
  () => !loading.value && failure.value === null && conversations.value.length === 0,
);

async function load(offset = 0) {
  loading.value = true;
  failure.value = null;
  try {
    const page = await listSessions(offset);
    conversations.value =
      offset === 0 ? page.sessions : [...conversations.value, ...page.sessions];
    hasMore.value = page.hasMore;
  } catch (error) {
    failure.value = agentErrorMessage("chat", error);
  } finally {
    loading.value = false;
  }
}

// Loaded on open rather than kept in memory: the device writes a better
// name for a conversation a second or two after it starts, so a list
// held from last time would show the owner a name that is no longer
// what the row is called.
onMounted(() => load());

/// The device is the authority on names and ordering, so anything that
/// changes either is followed by asking it again rather than by editing
/// the copy on screen and hoping the two agree.
async function change(action: Promise<unknown>) {
  try {
    await action;
    await load();
  } catch (error) {
    failure.value = agentErrorMessage("chat", error);
  }
}

function rename(id: string, title: string) {
  change(invoke("sessions_rename", { sessionId: id, title }));
}

function keep(id: string, kept: boolean) {
  change(invoke("sessions_keep", { sessionId: id, kept }));
}

function remove(id: string) {
  change(invoke("sessions_delete", { sessionId: id }));
}

</script>

<template>
  <div class="drawer">
    <header>
      <h2>Earlier conversations</h2>
      <button type="button" class="close" aria-label="Back to the conversation" @click="emit('close')">
        ✕
      </button>
    </header>

    <!-- Only appears once there is enough here to be worth narrowing.
         A search box over four rows is a control that costs more
         attention than it saves. -->
    <input
      v-if="conversations.length > 8"
      v-model="query"
      type="search"
      class="find"
      placeholder="Find a conversation"
      aria-label="Find a conversation"
    />

    <div class="scroller">
      <p v-if="failure" class="error" role="alert">{{ failure }}</p>
      <p v-else-if="loading && conversations.length === 0" class="quiet">
        One moment.
      </p>
      <p v-else-if="nothingYet" class="quiet">
        This is the only conversation so far. The ones you have later will
        be here.
      </p>
      <p v-else-if="groups.length === 0" class="quiet">
        Nothing here by that name.
      </p>

      <section v-for="group in groups" :key="group.label">
        <h3>{{ group.label }}</h3>
        <ul>
          <ConversationRow
            v-for="conversation in group.conversations"
            :key="conversation.id"
            :conversation="conversation"
            :current="conversation.id === current"
            :disabled="disabled"
            @open="emit('open', conversation.id)"
            @rename="(title) => rename(conversation.id, title)"
            @keep="(kept) => keep(conversation.id, kept)"
            @remove="remove(conversation.id)"
          />
        </ul>
      </section>

      <button
        v-if="hasMore && query === ''"
        type="button"
        class="more"
        :disabled="loading"
        @click="load(conversations.length)"
      >
        Show older ones
      </button>
    </div>

    <!-- Said once, at the bottom, where someone who has just tried to
         tap a row will look for why nothing happened. -->
    <p v-if="disabled" class="held">
      One moment -- you can switch once this reply has finished.
    </p>
  </div>
</template>

<style scoped>
.drawer {
  position: absolute;
  inset: 0;
  z-index: 5;
  display: flex;
  flex-direction: column;
  background: var(--bg);
}

header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 0.5rem;
  padding: 1.1rem 1rem 0.6rem;
}

h2 {
  margin: 0;
  color: var(--text-primary);
  font-size: 0.95rem;
  font-weight: 500;
}

.close {
  padding: 0.2rem 0.45rem;
  background: none;
  border: 0;
  border-radius: 6px;
  color: var(--text-secondary);
  font-size: 0.9rem;
  line-height: 1;
  cursor: pointer;
}

.close:hover {
  color: var(--text-primary);
}

.find {
  margin: 0 1rem 0.5rem;
  padding: 0.45rem 0.6rem;
  background: var(--surface);
  border: 1px solid transparent;
  border-radius: 9px;
  color: var(--text-primary);
  font: inherit;
  font-size: 0.85rem;
}

.find:focus {
  outline: none;
  border-color: var(--accent);
}

.scroller {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
  padding: 0 0.6rem 1rem;
}

section {
  margin-bottom: 0.9rem;
}

h3 {
  margin: 0.5rem 0 0.3rem;
  padding: 0 0.7rem;
  color: var(--text-secondary);
  font-size: 0.72rem;
  font-weight: 500;
  letter-spacing: 0.04em;
  text-transform: uppercase;
}

ul {
  margin: 0;
  padding: 0;
  list-style: none;
}

.quiet {
  margin: 1.2rem 1rem;
  color: var(--text-secondary);
  font-size: 0.85rem;
  line-height: 1.5;
}

.error {
  margin: 1.2rem 1rem;
  color: var(--danger);
  font-size: 0.85rem;
  line-height: 1.5;
}

.more {
  display: block;
  width: calc(100% - 1.4rem);
  margin: 0 0.7rem;
  padding: 0.5rem;
  background: none;
  border: 0;
  border-radius: 8px;
  color: var(--text-secondary);
  font: inherit;
  font-size: 0.82rem;
  cursor: pointer;
}

.more:hover:not(:disabled) {
  background: var(--surface);
  color: var(--text-primary);
}

.held {
  margin: 0;
  padding: 0.6rem 1rem;
  border-top: 1px solid color-mix(in srgb, var(--text-secondary) 15%, transparent);
  color: var(--text-secondary);
  font-size: 0.8rem;
}
</style>
