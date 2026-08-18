<script setup lang="ts">
// The floating pill: the orb and one input, on top of whatever else the
// owner is doing.
//
// This is what makes the assistant an assistant rather than a
// destination -- someone reading an invoice in a browser can ask a
// question without leaving what they are doing. It is the same
// conversation seen through a smaller opening, not a second copy of it:
// the state comes from the same composable the full pane uses, so a
// reply that started in one shape keeps arriving in the other.

import { parseReply, streamingText } from "~/lib/chatProtocol";

const {
  entries,
  input,
  busy,
  daemonError,
  awaitingApproval,
  orbState,
  connect,
  send,
  answerApproval,
} = useConversation();
const { holdingOpen, toggle, expand, drag } = useWindowMode();

const scroller = ref<HTMLElement | null>(null);

onMounted(connect);

/// The tail of the conversation. The pill is for the exchange happening
/// now; the whole history is what the full shell is for.
const recent = computed(() => entries.value.slice(-6));

const lastAssistant = computed(() => {
  for (let i = entries.value.length - 1; i >= 0; i -= 1) {
    const turn = entries.value[i];
    if (turn?.kind === "assistant") return turn;
    if (turn?.kind === "user") return null;
  }
  return null;
});

const options = computed(() =>
  busy.value || !lastAssistant.value
    ? []
    : parseReply(lastAssistant.value.content).options,
);

function bodyText(content: string) {
  return busy.value ? streamingText(content) : parseReply(content).text;
}

// Grows while there is something to read, settles back when there
// isn't. The hold is what keeps a finished reply readable instead of
// vanishing the moment the device stops talking.
const open = computed(
  () =>
    busy.value ||
    awaitingApproval.value ||
    options.value.length > 0 ||
    holdingOpen.value ||
    daemonError.value !== null,
);

watch(
  open,
  async (isOpen) => {
    await expand(isOpen);
    if (isOpen) await autoscroll();
  },
  { immediate: true },
);

async function autoscroll() {
  await nextTick();
  scroller.value?.scrollTo({ top: scroller.value.scrollHeight });
}

watch(entries, async () => {
  if (entries.value.length > 0) holdingOpen.value = true;
  await autoscroll();
}, { deep: true });

// The chips arrive after the last entry changed, and the pill's opening
// is small enough that anything below the fold is simply not there.
watch(options, autoscroll);

async function onSubmit() {
  await send();
}
</script>

<template>
  <div :class="['pill', { open }]">
    <section v-if="open" ref="scroller" class="recent" aria-live="polite">
      <p v-if="daemonError" class="error" role="alert">{{ daemonError }}</p>
      <template v-for="(entry, index) in recent" :key="index">
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

      <OptionChips :options="options" :disabled="busy" @pick="send" />
    </section>

    <form class="bar" @submit.prevent="onSubmit">
      <!-- The orb is the handle: the pill has no title bar, and the one
           thing always on screen is the thing to grab. -->
      <button
        type="button"
        class="grip"
        aria-label="Move the assistant"
        @mousedown="drag"
      >
        <PresenceOrb :size="34" :orb-state="orbState" />
      </button>

      <ChatInput
        v-model="input"
        variant="pill"
        placeholder="Say something…"
        :disabled="busy || daemonError !== null"
        @submit="onSubmit"
      />

      <button
        type="button"
        class="grow"
        aria-label="Back to the full window"
        title="Back to the full window"
        @click="toggle()"
      >
        ⤢
      </button>
    </form>
  </div>
</template>

<style scoped>
.pill {
  display: flex;
  flex-direction: column;
  justify-content: flex-end;
  height: 100vh;
  padding: 0.5rem;
  gap: 0.5rem;
  /* The window itself is transparent, so the rounding is the shape the
     owner actually sees. */
  background: none;
}

.recent {
  flex: 1;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  gap: 0.8rem;
  padding: 0.9rem 1rem;
  background: color-mix(in srgb, var(--bg) 96%, black);
  border: 1px solid rgba(255, 255, 255, 0.07);
  border-radius: 16px;
  font-size: 0.94rem;
}

.bar {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  padding: 0.4rem 0.55rem;
  background: color-mix(in srgb, var(--bg) 96%, black);
  border: 1px solid rgba(255, 255, 255, 0.07);
  border-radius: 999px;
}

.grip {
  background: none;
  border: none;
  padding: 0;
  display: grid;
  place-items: center;
  cursor: grab;
  flex: 0 0 auto;
}

.grip:active {
  cursor: grabbing;
}

.grow {
  background: none;
  border: none;
  color: var(--text-secondary);
  font-size: 1rem;
  padding: 0.35rem 0.5rem;
  cursor: pointer;
  flex: 0 0 auto;
}

.grow:hover {
  color: var(--text-primary);
}

.grip:focus-visible,
.grow:focus-visible {
  outline: 2px solid var(--accent);
  outline-offset: 2px;
  border-radius: 8px;
}

.user {
  margin: 0;
  align-self: flex-end;
  max-width: 85%;
  background: var(--surface);
  color: var(--text-secondary);
  font-size: 0.88rem;
  line-height: 1.5;
  padding: 0.5rem 0.8rem;
  border-radius: 13px;
  white-space: pre-wrap;
}

.error {
  margin: 0;
  color: var(--danger);
  font-size: 0.87rem;
  line-height: 1.5;
}
</style>
