<script setup lang="ts">
// Mic mode, as the owner sees it: a layer over the input, and nothing
// over the conversation.
//
// The rule this component exists to keep (design/DESIGN.md, "Voice") is
// that the transcript above it is never covered. Everything the device
// says is still written down and still arrives as it is written, so the
// owner watches the answer appear while they are hearing it -- and when
// they turn mic mode off, the whole exchange is already there. Covering
// the conversation in order to show that a conversation is happening is
// the mistake this replaces.
//
// One control. It is held, not toggled: nothing is recorded while it is
// not physically down, which is a thing the owner can see rather than a
// promise they have to take.

const props = defineProps<{ variant?: "pane" | "pill" }>();
const variant = computed(() => props.variant ?? "pane");

// The one owner of reading the reply out. It lives exactly as long as
// mic mode is on, which is exactly as long as anything should be spoken.
useVoiceNarration();

const {
  state,
  caption,
  recording,
  failure,
  configured,
  beginTalk,
  endTalk,
  cancelTalk,
  setMicMode,
} = useVoice();

const orbState = computed(() =>
  state.value === "unavailable" ? "idle" : state.value,
);

/// Pointer capture keeps the release on this element even if the hand
/// slides off it mid-sentence -- without it, letting go anywhere else
/// leaves the microphone open and the owner talking to a device that has
/// stopped listening.
function down(event: PointerEvent) {
  if (state.value === "unavailable") return;
  (event.currentTarget as HTMLElement).setPointerCapture?.(event.pointerId);
  void beginTalk();
}

function up() {
  void endTalk();
}

/// The window lost focus, or the pointer was cancelled by the system.
/// The recording is abandoned rather than sent: whatever was happening,
/// the owner was not finishing a question.
function cancel() {
  cancelTalk();
}

/// Space is the same control for someone who is already at the keyboard,
/// and the only way to reach this without a pointer at all. Repeats are
/// ignored -- a held key fires over and over, and each one would start a
/// new recording on top of the last.
function keydown(event: KeyboardEvent) {
  if (event.code !== "Space" || event.repeat) return;
  event.preventDefault();
  void beginTalk();
}

function keyup(event: KeyboardEvent) {
  if (event.code !== "Space") return;
  event.preventDefault();
  void endTalk();
}

// The microphone belongs to this gesture, so it is released when the
// control goes away -- mic mode turned off, the window changing shape,
// the page navigating. Never left open because a component vanished
// mid-hold.
onBeforeUnmount(cancelTalk);
</script>

<template>
  <div :class="['voice', variant]">
    <div class="row">
      <PresenceOrb :size="variant === 'pill' ? 34 : 44" :orb-state="orbState" />

      <button
        type="button"
        :class="['talk', { on: recording }]"
        :disabled="state === 'unavailable'"
        :aria-pressed="recording"
        aria-label="Hold to talk"
        @pointerdown="down"
        @pointerup="up"
        @pointercancel="cancel"
        @lostpointercapture="up"
        @keydown="keydown"
        @keyup="keyup"
        @blur="cancel"
      >
        <span class="caption">{{ caption }}</span>
      </button>

      <button
        type="button"
        class="close"
        aria-label="Stop listening and type instead"
        title="Type instead"
        @click="setMicMode(false)"
      >
        ⌨
      </button>
    </div>

    <!-- Spoken where the failure is, never as a toast, and always
         leaving the owner the way round: they can still type. -->
    <p v-if="failure" class="failure" role="alert">{{ failure }}</p>
    <p v-else-if="configured === false" class="failure" role="alert">
      I can't talk yet. You'll find the setting under Settings.
    </p>
  </div>
</template>

<style scoped>
.voice {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.voice.pane {
  padding: 0.75rem 1.25rem 1.25rem;
}

.voice.pill {
  padding: 0.35rem 0.5rem 0.5rem;
}

.row {
  display: flex;
  align-items: center;
  gap: 0.75rem;
}

/* The whole width is the target. Someone reaching for this is holding
   something else in their other hand, and a small button is one they
   have to look at to hit. */
.talk {
  flex: 1 1 0;
  min-width: 0;
  display: grid;
  place-items: center;
  padding: 0.85rem 1rem;
  border-radius: 999px;
  border: 1px solid color-mix(in srgb, var(--accent) 35%, transparent);
  background: color-mix(in srgb, var(--accent) 8%, var(--surface));
  color: var(--text-primary);
  font-family: var(--font-family);
  font-size: 0.95rem;
  cursor: pointer;
  /* A press must never select the caption instead of pressing the
     button, and a long hold on a touch screen must not raise the
     system's own text menu over it. */
  user-select: none;
  -webkit-user-select: none;
  touch-action: none;
  transition: background 0.18s ease, border-color 0.18s ease;
}

.voice.pill .talk {
  padding: 0.55rem 0.8rem;
  font-size: 0.88rem;
}

.talk:hover:not(:disabled) {
  background: color-mix(in srgb, var(--accent) 14%, var(--surface));
}

.talk.on {
  background: color-mix(in srgb, var(--accent) 26%, var(--surface));
  border-color: color-mix(in srgb, var(--accent) 70%, transparent);
}

.talk:disabled {
  opacity: 0.55;
  cursor: default;
}

.talk:focus-visible {
  outline: 2px solid var(--accent);
  outline-offset: 2px;
}

.caption {
  pointer-events: none;
}

.close {
  flex: 0 0 auto;
  background: none;
  border: none;
  color: var(--text-secondary);
  font-size: 1.1rem;
  line-height: 1;
  padding: 0.4rem;
  cursor: pointer;
}

.close:hover {
  color: var(--text-primary);
}

.close:focus-visible {
  outline: 2px solid var(--accent);
  outline-offset: 2px;
  border-radius: 8px;
}

.failure {
  margin: 0;
  color: var(--danger);
  font-size: 0.85rem;
  line-height: 1.5;
}

@media (prefers-reduced-motion: reduce) {
  .talk {
    transition: none;
  }
}
</style>
