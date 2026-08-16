<script setup lang="ts">
import { Channel, invoke } from "@tauri-apps/api/core";
import { waitForAgentReady } from "~/lib/agentStatus";
import { agentErrorMessage } from "~/lib/agentErrors";
import {
  completeOnboarding,
  getOnboardingQuestionCount,
  setOnboardingQuestionCount,
} from "~/lib/setupStore";

definePageMeta({ layout: false });

type Entry =
  | { kind: "user"; content: string }
  | { kind: "assistant"; content: string }
  | { kind: "error"; content: string };

type StreamEvent =
  | { type: "token"; content: string }
  | { type: "done" }
  | { type: "error"; message: string };

const entries = ref<Entry[]>([]);
const input = ref("");
const busy = ref(false);
const streaming = ref(false);
const finished = ref(false);
const daemonError = ref<string | null>(null);
const questionCount = ref(0);
const scroller = ref<HTMLElement | null>(null);

const orbState = computed(() =>
  busy.value ? (streaming.value ? "speaking" : "thinking") : "idle",
);

async function autoscroll() {
  await nextTick();
  scroller.value?.scrollTo({ top: scroller.value.scrollHeight });
}

async function runTurn(content?: string) {
  if (busy.value || finished.value) return;
  busy.value = true;
  streaming.value = false;
  if (content) {
    entries.value.push({ kind: "user", content });
  }
  entries.value.push({ kind: "assistant", content: "" });
  const replyIndex = entries.value.length - 1;
  await autoscroll();

  const onEvent = new Channel<StreamEvent>();
  onEvent.onmessage = (event) => {
    const reply = entries.value[replyIndex];
    if (!reply) return;
    if (event.type === "token") {
      streaming.value = true;
      if (reply.kind === "assistant") reply.content += event.content;
      autoscroll();
    } else if (event.type === "error") {
      entries.value[replyIndex] = {
        kind: "error",
        content: agentErrorMessage("setup", event.message),
      };
      autoscroll();
    }
  };

  try {
    const profileCommitted = await invoke<boolean>("agent_onboarding_chat", {
      input: content ?? null,
      questionCount: questionCount.value,
      onEvent,
    });
    const reply = entries.value[replyIndex];
    if (reply && reply.kind === "assistant" && reply.content === "") {
      entries.value[replyIndex] = {
        kind: "error",
        content: "The assistant returned no response.",
      };
      return;
    }
    if (profileCommitted) {
      await completeOnboarding();
      finished.value = true;
    } else {
      questionCount.value = Math.min(15, questionCount.value + 1);
      await setOnboardingQuestionCount(questionCount.value);
    }
  } catch (error) {
    entries.value[replyIndex] = {
      kind: "error",
      content: agentErrorMessage("setup", error),
    };
  } finally {
    busy.value = false;
    streaming.value = false;
    await autoscroll();
  }
}

async function send() {
  const content = input.value.trim();
  if (!content || busy.value || finished.value) return;
  input.value = "";
  await runTurn(content);
}

onMounted(async () => {
  try {
    await waitForAgentReady();
    questionCount.value = await getOnboardingQuestionCount();
    await runTurn();
  } catch (error) {
    daemonError.value = agentErrorMessage("setup", error);
  }
});
</script>

<template>
  <main>
    <header class="enter-fade">
      <PresenceOrb :size="48" :orb-state="orbState" />
    </header>

    <section ref="scroller" class="conversation" aria-live="polite">
      <p v-if="daemonError" class="error" role="alert">{{ daemonError }}</p>
      <template v-for="(entry, index) in entries" :key="index">
        <p v-if="entry.kind === 'user'" class="user">{{ entry.content }}</p>
        <p v-else-if="entry.kind === 'assistant'" class="assistant">
          {{ entry.content }}
        </p>
        <p v-else class="error" role="alert">{{ entry.content }}</p>
      </template>
    </section>

    <form v-if="!finished" @submit.prevent="send">
      <input
        v-model="input"
        type="text"
        placeholder="Your answer…"
        :disabled="busy || daemonError !== null"
        autocomplete="off"
        autofocus
      />
    </form>
    <NuxtLink
      v-else
      class="continue"
      to="/"
      aria-label="Continue"
      title="Continue"
    >
      →
    </NuxtLink>
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
  width: 100%;
  max-width: 680px;
  padding: 0.5rem 0 1rem;
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

.assistant {
  margin: 0;
  color: var(--text-primary);
  font-size: 1rem;
  line-height: 1.65;
  white-space: pre-wrap;
}

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

input:focus-visible,
.continue:focus-visible {
  outline: 2px solid var(--accent);
  outline-offset: 2px;
}

input:disabled {
  opacity: 0.55;
}

.continue {
  width: 2.75rem;
  height: 2.75rem;
  display: grid;
  place-items: center;
  border: 1px solid color-mix(in srgb, var(--accent) 55%, transparent);
  border-radius: 50%;
  color: var(--text-primary);
  font-size: 1.25rem;
  text-decoration: none;
}

.continue:hover {
  background: var(--surface-raised);
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
