<script setup lang="ts">
// The guided setup conversation. Same surface as the running device's
// chat -- the composable it shares is what makes them feel continuous --
// with the two things that are genuinely setup's own: the shell counts
// the questions, and the shell decides when the interview is over.

import { parseReply, streamingText } from "~/lib/chatProtocol";
import {
  completeOnboarding,
  getOnboardingQuestionCount,
  setOnboardingQuestionCount,
} from "~/lib/setupStore";

definePageMeta({ layout: false });

const {
  entries,
  input,
  busy,
  daemonError,
  orbState,
  connect,
  runTurn,
  send,
  answerApproval,
} = useConversation("onboarding");

const finished = ref(false);
const questionCount = ref(0);
const scroller = ref<HTMLElement | null>(null);

const lastAssistant = computed(() => {
  for (let i = entries.value.length - 1; i >= 0; i -= 1) {
    const turn = entries.value[i];
    if (turn?.kind === "assistant") return turn;
    if (turn?.kind === "user") return null;
  }
  return null;
});

// Most setup questions are answerable with a yes or a no, and this is
// where a tap matters most: fifteen questions put to someone who has
// owned the device for minutes and may be answering in their second
// language.
const options = computed(() =>
  busy.value || finished.value || !lastAssistant.value
    ? []
    : parseReply(lastAssistant.value.content).options,
);

function bodyText(content: string) {
  return busy.value ? streamingText(content) : parseReply(content).text;
}

async function autoscroll() {
  await nextTick();
  scroller.value?.scrollTo({ top: scroller.value.scrollHeight });
}

watch(entries, autoscroll, { deep: true });
watch(options, autoscroll);

/// The count is the shell's, not the model's: onboarding.md bounds the
/// interview at fifteen questions and the agent is told the running
/// total as an authoritative fact rather than asked to keep it.
async function recordTurn(profileCommitted: boolean) {
  if (profileCommitted) {
    await completeOnboarding();
    finished.value = true;
    return;
  }
  questionCount.value = Math.min(15, questionCount.value + 1);
  await setOnboardingQuestionCount(questionCount.value);
}

async function onSubmit() {
  if (finished.value) return;
  await recordTurn(await send());
  await autoscroll();
}

async function onPick(option: string) {
  if (finished.value) return;
  await recordTurn(await send(option));
  await autoscroll();
}

onMounted(async () => {
  await connect();
  if (daemonError.value !== null) return;
  questionCount.value = await getOnboardingQuestionCount();
  // The agent speaks first: nobody should face an empty prompt on a
  // device they have owned for a minute.
  try {
    await recordTurn(await runTurn(null));
  } catch {
    // runTurn already put the failure in the conversation, where
    // errors are spoken rather than swallowed.
  }
  await autoscroll();
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
        <MessageBody
          v-else-if="entry.kind === 'assistant' && entry.content !== ''"
          :text="bodyText(entry.content)"
        />
        <ToolRow
          v-else-if="entry.kind === 'tool'"
          :summary="entry.summary"
          :phase="entry.phase"
        />
        <ApprovalCard
          v-else-if="entry.kind === 'approval'"
          :description="entry.description"
          :command="entry.command"
          :choices="entry.choices"
          :answer="entry.answer"
          @answer="(choice) => answerApproval(entry, choice)"
        />
        <p v-else-if="entry.kind === 'error'" class="error" role="alert">
          {{ entry.content }}
        </p>
      </template>

      <OptionChips :options="options" :disabled="busy" @pick="onPick" />
    </section>

    <form v-if="!finished" @submit.prevent="onSubmit">
      <ChatInput
        v-model="input"
        variant="setup"
        placeholder="Your answer…"
        :disabled="busy || daemonError !== null"
        autofocus
        @submit="onSubmit"
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

.continue:focus-visible {
  outline: 2px solid var(--accent);
  outline-offset: 2px;
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
