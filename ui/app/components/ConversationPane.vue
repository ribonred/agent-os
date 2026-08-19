<script setup lang="ts">
// The conversation surface as a pane (design/DESIGN.md): the orb is the
// other party, assistant text renders bare on the canvas, owner messages
// are quiet pills, streaming is visible, errors are spoken in the flow.
//
// Collapsed, this narrows to a rail holding just the orb -- the orb is
// never removed from the screen, because a screen with no orb reads as a
// device that is switched off.

import { parseReply, streamingText } from "~/lib/chatProtocol";

const { collapsed = false } = defineProps<{ collapsed?: boolean }>();
const emit = defineEmits<{ "update:collapsed": [value: boolean] }>();

const {
  entries,
  input,
  busy,
  daemonError,
  orbState,
  sessionId,
  connect,
  send,
  answerApproval,
  stop,
  restore,
  openSession,
  newConversation,
} = useConversation();

// The list of earlier conversations covers this pane while it is open.
const history = ref(false);

async function onOpenSession(id: string) {
  await openSession(id);
  history.value = false;
  await autoscroll();
}

async function onNewConversation() {
  history.value = false;
  await newConversation();
}
const { set: setWindowMode, drag, toggleMaximize } = useWindowMode();

// The top of the pane is this window's title bar, because the window
// does not have one: dragging it moves the window, and double-clicking
// fills the screen or gives it back. Both are ignored on the controls
// themselves, so the buttons still behave like buttons.
function onChromeMouseDown(event: MouseEvent) {
  if ((event.target as HTMLElement)?.closest("button, a")) return;
  if (event.detail > 1) return;
  drag();
}

function onChromeDoubleClick(event: MouseEvent) {
  if ((event.target as HTMLElement)?.closest("button, a")) return;
  toggleMaximize();
}
const scroller = ref<HTMLElement | null>(null);

// The placeholder says what pressing Enter will do, so a selected file
// doesn't sit there ambiguously beside a generic invitation.
const { items: contextItems } = useContext();
const hasContext = computed(() => contextItems.value.length > 0);

onMounted(async () => {
  await connect();
  // Back where they left it, then scrolled to the end of it -- the
  // owner returns to the last thing that was said, not to the first.
  await restore();
  await autoscroll();
});

/// The answers offered with the reply the owner is looking at -- only
/// the last one, and only once it has finished arriving. A half-written
/// question flickering into buttons is worse than waiting a beat.
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

/// Mid-stream the trailer is hidden as it types itself; once the turn is
/// done it is gone entirely and its contents are chips instead.
function bodyText(content: string) {
  return busy.value ? streamingText(content) : parseReply(content).text;
}

async function autoscroll() {
  await nextTick();
  scroller.value?.scrollTo({ top: scroller.value.scrollHeight });
}

// The composable owns the entries; the pane owns the scroller, so it
// follows the stream from here rather than reaching back the other way.
watch(entries, autoscroll, { deep: true });

// The answers to tap only appear once the turn is complete, which is
// after the last entry changed -- without this they render below the
// fold and the owner never sees they had a choice.
watch(options, autoscroll);

async function onSubmit() {
  await send();
  await autoscroll();
}

async function onPick(option: string) {
  await send(option);
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
      <header @mousedown="onChromeMouseDown" @dblclick="onChromeDoubleClick">
        <PresenceOrb :size="56" :orb-state="orbState" />
        <div class="controls">
          <!-- Starting a new subject and going back to an old one, side
               by side: they are the same decision seen from either end,
               and the owner reaches for whichever the moment calls for. -->
          <button
            type="button"
            class="icon"
            aria-label="Start a new conversation"
            title="Start a new conversation"
            :disabled="busy"
            @click="onNewConversation"
          >
            ✎
          </button>
          <button
            type="button"
            class="icon"
            aria-label="Earlier conversations"
            title="Earlier conversations"
            :aria-expanded="history"
            @click="history = !history"
          >
            ☰
          </button>
          <!-- Reachable but quiet, like the desktop underneath: there is
               no settings system here, only the few things the owner
               decides rather than asks for. -->
          <NuxtLink to="/settings" class="icon" aria-label="Settings">
            ⚙
          </NuxtLink>
          <button
            type="button"
            class="icon"
            aria-label="Keep the assistant to hand"
            title="Keep the assistant to hand"
            @click="setWindowMode('minimized')"
          >
            ⤡
          </button>
          <button
            type="button"
            class="icon"
            aria-label="Hide the conversation"
            :aria-expanded="true"
            @click="emit('update:collapsed', true)"
          >
            ‹
          </button>
        </div>
      </header>

      <section ref="scroller" class="conversation" aria-live="polite">
        <p v-if="daemonError" class="error" role="alert">{{ daemonError }}</p>
        <p v-else-if="entries.length === 0" class="empty">Ask me anything.</p>
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

      <ContextChip />

      <form @submit.prevent="onSubmit">
        <ChatInput
          v-model="input"
          variant="pane"
          :placeholder="hasContext ? 'Ask about this…' : 'Say something…'"
          :disabled="busy || daemonError !== null"
          @submit="onSubmit"
        />
        <!-- Only while there is something to interrupt: a control that
             does nothing most of the time is one the owner learns to
             ignore. -->
        <button v-if="busy" type="button" class="stop" @click="stop">
          Stop
        </button>
      </form>

      <!-- While a reply is arriving the list still opens and reads, but
           the rows do not respond: the reply has somewhere to land, and
           a question the device asked has somewhere to be answered. -->
      <HistoryDrawer
        v-if="history"
        :current="sessionId"
        :disabled="busy"
        @close="history = false"
        @open="onOpenSession"
      />
    </template>
  </aside>
</template>

<style scoped>
.pane {
  /* The list of earlier conversations covers this pane rather than
     opening beside it, so it is positioned against this. */
  position: relative;
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
.icon:focus-visible {
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

.controls {
  position: absolute;
  right: 0.75rem;
  display: flex;
  align-items: center;
  gap: 0.15rem;
}

.icon {
  background: none;
  border: none;
  text-decoration: none;
  color: var(--text-secondary);
  font-size: 1.15rem;
  line-height: 1;
  padding: 0.25rem 0.4rem;
  cursor: pointer;
}

.icon:hover:not(:disabled) {
  color: var(--text-primary);
}

/* Starting a new conversation mid-reply would leave the reply arriving
   into a pane that is no longer showing it. */
.icon:disabled {
  opacity: 0.35;
  cursor: default;
}

.conversation {
  flex: 1 1 0;
  min-height: 0;
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
  flex: 0 0 auto;
  min-width: 0;
  padding: 0 1.5rem;
  display: flex;
  align-items: flex-end;
  gap: 0.5rem;
}

.stop {
  flex: 0 0 auto;
  background: var(--surface);
  color: var(--text-secondary);
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 12px;
  padding: 0.75rem 0.9rem;
  font-family: var(--font-family);
  font-size: 0.88rem;
  cursor: pointer;
}

.stop:hover {
  color: var(--text-primary);
}

.stop:focus-visible {
  outline: 2px solid var(--accent);
  outline-offset: 2px;
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
