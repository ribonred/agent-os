import { invoke } from "@tauri-apps/api/core";
import { nextSpeech, speakableText } from "~/lib/speechChunks";
import { CaptureError, VoiceRecording, type CaptureFailure } from "~/lib/voiceCapture";
import { voiceErrorMessage } from "~/lib/agentErrors";
// Aliased: this composable exports its own `setMicMode`, which is the
// one callers want -- it also stops a recording and any speech in
// flight, where the stored one only remembers the choice.
import {
  getMicMode as getStoredMicMode,
  setMicMode as setStoredMicMode,
} from "~/lib/shelfStore";

// Speaking to the device, and being answered out loud.
//
// A layer over the conversation rather than a second one: the turn it
// sends is the same turn the composer sends, lands in the same
// transcript, and streams back into the same pane. What differs is only
// how it got in and how it comes back out.
//
// The reply is read by *watching* the conversation rather than by being
// handed each token, which is the whole reason the streaming path
// upstairs needed no changes at all. Everything spoken here is the text
// the owner can also see -- there is no second, spoken version of a
// reply that could disagree with the written one.

export type VoiceState =
  | "idle"
  | "listening"
  | "thinking"
  | "speaking"
  | "unavailable";

/// How long a hold has to last to be a question rather than a slip. A
/// tap that opens and closes the microphone in the same frame produces a
/// recording of nothing, which the provider charges for and answers with
/// silence.
const MIN_HOLD_MS = 320;

// There is one microphone and one speaker on this device, so there is
// one of each of these -- not one per component that asks about voice.
// Several places call `useVoice()` (the pane, the pill, the layer, the
// settings screen), and giving each of them its own queue would have the
// device reading the same reply to itself twice, over the top of itself.
// Reactive state is shared through `useState`; these are the live
// objects behind it.
let held: VoiceRecording | null = null;
let heldSince = 0;
/// Whether the control is down *now*, as opposed to whether a recording
/// has finished starting. Opening a microphone takes long enough for a
/// short press to be over before it succeeds, and without this the
/// stream that arrives afterwards has nobody left to stop it -- a device
/// quietly recording the room with its own button already released.
let wantsToTalk = false;
let context: AudioContext | null = null;
let playing: AudioBufferSourceNode | null = null;
const queue: string[] = [];
let draining = false;
/// Bumped whenever speech is abandoned. Anything in flight compares
/// against it before it plays, so a clip that was already being fetched
/// when the owner interrupted is dropped instead of arriving a second
/// later over the top of them.
let generation = 0;
/// How far into the reply being read has been handed to the speaker.
let cursor = 0;
let spokenTurn = -1;

export function useVoice() {
  const micMode = useState<boolean>("voice:micMode", () => false);
  /// Null until asked. Kept separate from `micMode` so the control can be
  /// hidden entirely on a device with no speech key rather than offered
  /// and then failing when pressed.
  const configured = useState<boolean | null>("voice:configured", () => null);
  const recording = useState<boolean>("voice:recording", () => false);
  const speaking = useState<boolean>("voice:speaking", () => false);
  const transcribing = useState<boolean>("voice:transcribing", () => false);
  /// Spoken in the layer, never as a toast, and always in the owner's
  /// terms. Cleared by the next successful thing that happens.
  const failure = useState<string | null>("voice:failure", () => null);

  const { busy, send } = useConversation();


  const state = computed<VoiceState>(() => {
    if (configured.value === false) return "unavailable";
    if (recording.value) return "listening";
    if (transcribing.value || (busy.value && !speaking.value)) return "thinking";
    if (speaking.value) return "speaking";
    return "idle";
  });

  /// What the layer says it is doing. One line, in the owner's words --
  /// never a level meter, which is an instrument for someone tuning a
  /// recording rather than for someone asking a question.
  const caption = computed(() => {
    switch (state.value) {
      case "listening":
        return "Listening…";
      case "thinking":
        return "One moment…";
      case "speaking":
        return "Speaking";
      case "unavailable":
        return "I can't speak yet";
      default:
        return "Hold to talk";
    }
  });

  async function checkConfigured() {
    if (configured.value !== null) return;
    try {
      configured.value = (await invoke<string>("voice_key_status")) !== "none";
    } catch {
      // The command failing is not the same as no key, but from the
      // owner's side it has the same consequence, and the layer says so
      // rather than offering a control that cannot work.
      configured.value = false;
    }
  }

  // -- speaking ---------------------------------------------------------

  function audio(): AudioContext {
    if (!context) context = new AudioContext();
    return context;
  }

  /// Stops talking, now, and forgets anything queued.
  ///
  /// Called when the owner starts speaking over the device and when a
  /// turn is stopped. Interrupting has to be instant to be worth having:
  /// a device that finishes its sentence first is one the owner learns
  /// to wait for.
  function hush() {
    generation += 1;
    queue.length = 0;
    if (playing) {
      try {
        playing.stop();
      } catch {
        // Already finished on its own.
      }
      playing = null;
    }
    speaking.value = false;
  }

  async function play(bytes: ArrayBuffer, mine: number): Promise<void> {
    const ctx = audio();
    // WebKitGTK resumes suspended contexts lazily; without this the
    // first clip after a quiet period plays into a stopped clock.
    if (ctx.state === "suspended") await ctx.resume();
    const buffer = await ctx.decodeAudioData(bytes);
    if (mine !== generation) return;

    await new Promise<void>((resolve) => {
      const source = ctx.createBufferSource();
      source.buffer = buffer;
      source.connect(ctx.destination);
      source.onended = () => {
        if (playing === source) playing = null;
        resolve();
      };
      playing = source;
      source.start();
    });
  }

  async function drain() {
    if (draining) return;
    draining = true;
    const mine = generation;
    speaking.value = true;
    try {
      while (queue.length > 0 && mine === generation) {
        const sentence = queue.shift() as string;
        const bytes = await invoke<ArrayBuffer>("voice_speak", { text: sentence });
        if (mine !== generation) break;
        await play(bytes, mine);
      }
    } catch (error) {
      console.error("could not speak", error);
      failure.value = voiceErrorMessage(error);
      queue.length = 0;
    } finally {
      draining = false;
      if (mine === generation) speaking.value = false;
      // Anything queued while this loop was winding down has no drainer:
      // the loop it would have joined had already decided to stop. Left
      // alone, the first reply after an interruption is silent.
      if (queue.length > 0) void drain();
    }
  }

  function say(sentences: string[]) {
    if (sentences.length === 0) return;
    queue.push(...sentences);
    void drain();
  }

  // -- listening --------------------------------------------------------

  function captureMessage(reason: CaptureFailure): string {
    switch (reason) {
      case "no-device":
        // The ordinary case on a unit whose headset is not plugged in,
        // not a fault: said as a fact with the fix in it.
        return "I can't hear anything plugged in. Connect a microphone and try again.";
      case "refused":
        return "Something on this device is holding the microphone. This needs someone to look at it.";
      case "unsupported":
        return "This device can't record sound. You can still type to me.";
      default:
        return "I couldn't start listening just now.";
    }
  }

  /// The control went down.
  async function beginTalk() {
    if (wantsToTalk || held) return;
    wantsToTalk = true;
    await checkConfigured();
    if (!configured.value) {
      wantsToTalk = false;
      failure.value = "I need a speech connection before I can listen. You'll find it in Settings.";
      return;
    }
    // Talking over the device stops it talking. This is the barge-in,
    // and it happens on the press rather than on the release so the room
    // goes quiet the instant the owner decides to speak.
    hush();
    failure.value = null;
    try {
      const started = await VoiceRecording.start();
      // Let go while the microphone was still opening. Nothing was
      // asked, so nothing is kept -- and the stream is closed rather
      // than left running behind a control that is already up.
      if (!wantsToTalk) {
        started.discard();
        return;
      }
      held = started;
      heldSince = Date.now();
      recording.value = true;
    } catch (error) {
      wantsToTalk = false;
      held = null;
      failure.value =
        error instanceof CaptureError
          ? captureMessage(error.reason)
          : captureMessage("failed");
    }
  }

  /// The control came up: transcribe what was said and send it as an
  /// ordinary turn.
  async function endTalk() {
    wantsToTalk = false;
    const recorder = held;
    held = null;
    if (!recorder) return;
    recording.value = false;

    // Too brief to be speech -- a mis-tap, or a hand resting on the
    // control. Thrown away without a word: the owner did not ask
    // anything, so there is nothing to report.
    if (Date.now() - heldSince < MIN_HOLD_MS) {
      recorder.discard();
      return;
    }

    transcribing.value = true;
    try {
      const { bytes, mime } = await recorder.stop();
      if (bytes.byteLength === 0) return;

      // Raw bytes rather than a JSON field: a few seconds of audio as a
      // number array is several times the size and has to be rebuilt
      // byte by byte on the other side.
      const said = await invoke<string>("voice_transcribe", bytes, {
        headers: { "x-audio-mime": mime },
      });
      const spoken = said.trim();
      if (spoken === "") {
        // Silence, or a room too loud to pick anything out of it.
        // Nothing is sent: an empty turn would have the device
        // apologising for a question that was never asked.
        failure.value = "I didn't catch that.";
        return;
      }
      failure.value = null;
      await send(spoken, true);
    } catch (error) {
      console.error("could not hear the owner", error);
      failure.value = voiceErrorMessage(error);
    } finally {
      transcribing.value = false;
    }
  }

  /// The pointer left the control, the window lost focus, the page is
  /// going away. Whatever was being recorded is abandoned and the
  /// microphone released -- never left open because a gesture ended in
  /// an unexpected way.
  function cancelTalk() {
    wantsToTalk = false;
    if (held) {
      held.discard();
      held = null;
    }
    recording.value = false;
  }

  async function setMicMode(on: boolean) {
    micMode.value = on;
    if (!on) {
      cancelTalk();
      hush();
    } else {
      await checkConfigured();
    }
    await setStoredMicMode(on);
  }

  async function restoreMicMode() {
    micMode.value = await getStoredMicMode();
    if (micMode.value) await checkConfigured();
  }

  /// Reads whatever is new in the reply being written. Driven from one
  /// place only -- see `useVoiceNarration` below.
  function readReplyAloud(index: number, text: string, done: boolean) {
    if (index !== spokenTurn) {
      spokenTurn = index;
      // A reply that is already finished the first time it is seen was
      // never streamed here -- it is a conversation the owner reopened.
      // Reading their own history back at them is not what they asked
      // for, so it is marked as already said.
      if (done) {
        cursor = speakableText(text).length;
        return;
      }
      // A live turn: read it from its beginning.
      cursor = 0;
    }
    const speech = nextSpeech(text, cursor, done);
    cursor = speech.cursor;
    say(speech.chunks);
  }

  return {
    micMode,
    configured,
    recording,
    speaking,
    state,
    caption,
    failure,
    beginTalk,
    endTalk,
    cancelTalk,
    hush,
    setMicMode,
    restoreMicMode,
    checkConfigured,
    readReplyAloud,
  };
}

/// Reads the reply out as it is written.
///
/// Deliberately separate from `useVoice`, and called from exactly one
/// component: the layer, which exists only while mic mode is on. Every
/// other consumer of `useVoice` wants the state and the controls, and a
/// watcher registered per consumer would mean the same reply read out
/// once per component that happened to be mounted.
///
/// Watching the transcript rather than intercepting the stream keeps
/// this entirely to one side of the conversation: nothing here can
/// delay, reorder or swallow a token on its way to the screen, and the
/// device can only ever say what the owner can also read.
export function useVoiceNarration() {
  const { micMode, readReplyAloud } = useVoice();
  const { entries, busy } = useConversation();

  watch(
    () => {
      const index = entries.value.length - 1;
      const last = entries.value[index];
      return {
        index,
        text: last?.kind === "assistant" ? last.content : null,
        done: !busy.value,
      };
    },
    (now) => {
      if (!import.meta.client || !micMode.value || now.text === null) return;
      readReplyAloud(now.index, now.text, now.done);
    },
    { deep: true },
  );
}
