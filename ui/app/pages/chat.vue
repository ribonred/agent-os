<script setup lang="ts">
import { Channel, invoke } from "@tauri-apps/api/core";

// The conversation surface (design/DESIGN.md): the orb is the other
// party, assistant text renders bare on the canvas, user messages are
// quiet pills, streaming is visible, errors are spoken in the flow.

type Entry =
  | { kind: "user"; content: string }
  | { kind: "assistant"; content: string }
  | { kind: "error"; content: string };

type StreamEvent =
  | { type: "token"; content: string }
  | { type: "done"; backend: string; model: string }
  | { type: "error"; message: string };

const entries = ref<Entry[]>([]);
const input = ref("");
const busy = ref(false);
const streaming = ref(false);
const daemonError = ref<string | null>(null);
const scroller = ref<HTMLElement | null>(null);

// Orb rhythm is the only status indicator: thinking between send and
// first token, speaking while tokens flow, idle otherwise.
const orbState = computed(() =>
  busy.value ? (streaming.value ? "speaking" : "thinking") : "idle",
);

onMounted(async () => {
  try {
    await invoke("agent_status");
  } catch (e) {
    daemonError.value = String(e);
  }
});

async function autoscroll() {
  await nextTick();
  scroller.value?.scrollTo({ top: scroller.value.scrollHeight });
}

async function send() {
  const content = input.value.trim();
  if (!content || busy.value) return;
  input.value = "";
  busy.value = true;
  streaming.value = false;
  entries.value.push({ kind: "user", content });
  await autoscroll();

  // Full turn history goes to the daemon -- it owns the system prompt;
  // error entries are UI-local and never sent back as context.
  const messages = entries.value
    .filter((e) => e.kind !== "error")
    .map((e) => ({
      role: e.kind === "user" ? "user" : "assistant",
      content: e.content,
    }));

  entries.value.push({ kind: "assistant", content: "" });
  const replyIndex = entries.value.length - 1;

  const onEvent = new Channel<StreamEvent>();
  onEvent.onmessage = (event) => {
    const reply = entries.value[replyIndex];
    if (!reply) return;
    if (event.type === "token") {
      streaming.value = true;
      if (reply.kind === "assistant") reply.content += event.content;
      autoscroll();
    } else if (event.type === "error") {
      entries.value[replyIndex] = { kind: "error", content: event.message };
      autoscroll();
    }
    // "done" carries backend/model -- routing is disclosed on request
    // only (constitution.md), so the UI reads it and shows nothing.
  };

  try {
    await invoke("agent_chat", { messages, onEvent });
    // An empty reply with no error event means the stream never
    // produced content -- say so rather than leaving a blank line.
    const reply = entries.value[replyIndex];
    if (reply && reply.kind === "assistant" && reply.content === "") {
      entries.value[replyIndex] = {
        kind: "error",
        content: "The assistant returned no response.",
      };
    }
  } catch (e) {
    entries.value[replyIndex] = { kind: "error", content: String(e) };
  } finally {
    busy.value = false;
    streaming.value = false;
    await autoscroll();
  }
}
</script>

<template>
  <main>
    <header class="enter-fade">
      <NuxtLink class="back" to="/" aria-label="Back to home">‹</NuxtLink>
      <PresenceOrb :size="48" :orb-state="orbState" />
    </header>

    <section ref="scroller" class="conversation">
      <p v-if="daemonError" class="error" role="alert">{{ daemonError }}</p>
      <p v-else-if="entries.length === 0" class="empty enter-fade">
        Ask me anything.
      </p>
      <template v-for="(entry, i) in entries" :key="i">
        <p v-if="entry.kind === 'user'" class="user">{{ entry.content }}</p>
        <p v-else-if="entry.kind === 'assistant'" class="assistant">
          {{ entry.content }}
        </p>
        <p v-else class="error" role="alert">{{ entry.content }}</p>
      </template>
    </section>

    <form @submit.prevent="send">
      <input
        v-model="input"
        type="text"
        placeholder="Say something…"
        :disabled="busy || daemonError !== null"
        autocomplete="off"
      />
    </form>
  </main>
</template>

<style scoped>
main {
  height: 100vh;
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: 1.5rem 0 1.75rem;
  gap: 1rem;
}

header {
  display: flex;
  align-items: center;
  justify-content: center;
  position: relative;
  width: 100%;
  max-width: 680px;
  padding: 0.5rem 0 1rem;
}

.back {
  position: absolute;
  left: 1.5rem;
  color: var(--text-secondary);
  font-size: 1.6rem;
  line-height: 1;
  text-decoration: none;
}

.back:hover {
  color: var(--text-primary);
}

.conversation {
  flex: 1;
  width: 100%;
  max-width: 680px;
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

/* The user's words are context: quiet, right-aligned pill */
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
  width: 100%;
  max-width: 680px;
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

.enter-fade {
  animation: fade-in 0.5s both;
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
