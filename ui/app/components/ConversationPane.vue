<script setup lang="ts">
// The conversation surface as a pane (design/DESIGN.md): the orb is the
// other party, assistant text renders bare on the canvas, owner messages
// are quiet pills, streaming is visible, errors are spoken in the flow.
//
// Collapsed, this narrows to a rail holding just the orb -- the orb is
// never removed from the screen, because a screen with no orb reads as a
// device that is switched off.

const { collapsed = false } = defineProps<{ collapsed?: boolean }>();
const emit = defineEmits<{ "update:collapsed": [value: boolean] }>();

const { entries, input, busy, daemonError, orbState, connect, send } =
  useConversation();
const scroller = ref<HTMLElement | null>(null);

onMounted(connect);

async function autoscroll() {
  await nextTick();
  scroller.value?.scrollTo({ top: scroller.value.scrollHeight });
}

// The composable owns the entries; the pane owns the scroller, so it
// follows the stream from here rather than reaching back the other way.
watch(entries, autoscroll, { deep: true });

async function onSubmit() {
  await send();
  await autoscroll();
}
</script>

<template>
  <aside :class="['pane', { collapsed }]">
    <template v-if="collapsed">
      <button
        type="button"
        class="rail"
        aria-label="Show the conversation"
        :aria-expanded="false"
        @click="emit('update:collapsed', false)"
      >
        <PresenceOrb :size="40" :orb-state="orbState" />
      </button>
    </template>

    <template v-else>
      <header>
        <PresenceOrb :size="56" :orb-state="orbState" />
        <button
          type="button"
          class="collapse"
          aria-label="Hide the conversation"
          :aria-expanded="true"
          @click="emit('update:collapsed', true)"
        >
          ‹
        </button>
      </header>

      <section ref="scroller" class="conversation" aria-live="polite">
        <p v-if="daemonError" class="error" role="alert">{{ daemonError }}</p>
        <p v-else-if="entries.length === 0" class="empty">Ask me anything.</p>
        <template v-for="(entry, index) in entries" :key="index">
          <p v-if="entry.kind === 'user'" class="user">{{ entry.content }}</p>
          <p v-else-if="entry.kind === 'assistant'" class="assistant">
            {{ entry.content }}
          </p>
          <p v-else class="error" role="alert">{{ entry.content }}</p>
        </template>
      </section>

      <form @submit.prevent="onSubmit">
        <input
          v-model="input"
          type="text"
          placeholder="Say something…"
          :disabled="busy || daemonError !== null"
          autocomplete="off"
        />
      </form>
    </template>
  </aside>
</template>

<style scoped>
.pane {
  display: flex;
  flex-direction: column;
  height: 100vh;
  flex: 0 0 460px;
  width: 460px;
  padding: 1.25rem 0 1.5rem;
  gap: 0.75rem;
  border-right: 1px solid rgba(255, 255, 255, 0.05);
  background: color-mix(in srgb, var(--bg) 92%, black);
  transition: flex-basis 0.28s ease, width 0.28s ease;
}

.pane.collapsed {
  flex-basis: 72px;
  width: 72px;
  padding: 0;
}

/* Collapsed: the orb alone, centered, and the whole rail restores it */
.rail {
  flex: 1;
  display: grid;
  place-items: center;
  width: 100%;
  background: none;
  border: none;
  cursor: pointer;
  padding: 0;
}

.rail:focus-visible,
.collapse:focus-visible {
  outline: 2px solid var(--accent);
  outline-offset: -2px;
}

header {
  position: relative;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 0.5rem 0 0.75rem;
}

.collapse {
  position: absolute;
  right: 1rem;
  background: none;
  border: none;
  color: var(--text-secondary);
  font-size: 1.5rem;
  line-height: 1;
  padding: 0.25rem 0.5rem;
  cursor: pointer;
}

.collapse:hover {
  color: var(--text-primary);
}

.conversation {
  flex: 1;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  gap: 1.1rem;
  padding: 0.5rem 1.5rem;
}

.empty {
  margin: auto;
  color: var(--text-secondary);
  font-size: 1.05rem;
  font-weight: 300;
  letter-spacing: 0.02em;
}

/* The device speaking: bare text on the canvas, full measure */
.assistant {
  margin: 0;
  color: var(--text-primary);
  font-size: 1rem;
  line-height: 1.65;
  white-space: pre-wrap;
}

/* The owner's words are context: quiet, right-aligned pill */
.user {
  margin: 0;
  align-self: flex-end;
  max-width: 80%;
  background: var(--surface);
  color: var(--text-secondary);
  font-size: 0.92rem;
  line-height: 1.5;
  padding: 0.6rem 0.95rem;
  border-radius: 14px;
  white-space: pre-wrap;
}

.error {
  margin: 0;
  color: var(--danger);
  font-size: 0.9rem;
  line-height: 1.5;
}

form {
  padding: 0 1.5rem;
}

input {
  width: 100%;
  background: var(--surface);
  color: var(--text-primary);
  border: 1px solid rgba(255, 255, 255, 0.06);
  border-radius: 12px;
  padding: 0.9rem 1.1rem;
  font-family: var(--font-family);
  font-size: 0.98rem;
}

input:focus-visible {
  outline: 2px solid var(--accent);
  outline-offset: 1px;
}

input:disabled {
  opacity: 0.55;
}

/* The pane has an optimal measure for bare assistant text; a percentage
   is comfortable at one size and wrong at every other. */
@media (max-width: 1599px) {
  .pane {
    flex-basis: 400px;
    width: 400px;
  }
}

@media (max-width: 1279px) {
  .pane {
    flex-basis: 360px;
    width: 360px;
  }
}

@media (prefers-reduced-motion: reduce) {
  .pane {
    transition: none;
  }
}
</style>
