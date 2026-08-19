import { Channel, invoke } from "@tauri-apps/api/core";
import { waitForAgentReady } from "~/lib/agentStatus";
import { agentErrorMessage, streamErrorMessage } from "~/lib/agentErrors";
import { toolSummary } from "~/lib/toolNames";
import { getOnboardingQuestionCount } from "~/lib/setupStore";
import type { Conversation } from "~/lib/sessionList";

// The conversation's state and streaming, owned by the shell rather than
// by a page. The pane is mounted once for the life of the shell, so a
// reply that started before the owner went looking through their things
// keeps arriving while they browse -- navigating the main area no longer
// tears the stream down, and neither does shrinking to the pill.
//
// One composable for both the normal conversation and setup. They differ
// in which command runs a turn and in what ends them, not in how a turn
// behaves, and an earlier second copy of this loop meant every
// improvement had to be made twice or silently wasn't.

// Named for what it is rather than a bare "Entry": the file view exports
// an Entry too, and Nuxt's auto-import silently resolves a collision to
// one of them.
export type Turn =
  | { kind: "user"; content: string }
  | { kind: "assistant"; content: string }
  | { kind: "error"; content: string }
  | { kind: "tool"; name: string; summary: string; phase: ToolPhase }
  | {
      kind: "approval";
      runId: string;
      description: string;
      command: string;
      choices: string[];
      answer: string | null;
    };

export type ToolPhase = "started" | "completed" | "failed";

type StreamEvent =
  | { type: "run"; runId: string }
  | { type: "token"; content: string }
  | { type: "tool"; name: string; phase: ToolPhase }
  | {
      type: "approval";
      description: string;
      command: string;
      choices: string[];
    }
  | { type: "done" }
  | { type: "cancelled" }
  | { type: "error"; message: string };

export type OrbState = "idle" | "thinking" | "speaking";

export type ConversationMode = "chat" | "onboarding";

/// Setup keeps its own transcript so a restart mid-onboarding resumes
/// the interview rather than joining the middle of a chat.
const STATE_KEYS: Record<ConversationMode, string> = {
  chat: "conversation",
  onboarding: "onboarding",
};

export function useConversation(mode: ConversationMode = "chat") {
  const key = STATE_KEYS[mode];
  // Setup's failures are worded for someone who has owned the device
  // for two minutes; the running device's are worded for someone using
  // it. Same table, different entry.
  const surface = mode === "onboarding" ? "setup" : "chat";
  // useState rather than module-level refs: one instance shared by every
  // consumer, without leaking across app instances.
  const entries = useState<Turn[]>(`${key}:entries`, () => []);
  const input = useState<string>(`${key}:input`, () => "");
  const busy = useState<boolean>(`${key}:busy`, () => false);
  const streaming = useState<boolean>(`${key}:streaming`, () => false);
  const daemonError = useState<string | null>(`${key}:daemonError`, () => null);
  const ready = useState<boolean>(`${key}:ready`, () => false);
  /// The turn currently running, which is what a permission answer and
  /// the stop control both have to address.
  const runId = useState<string | null>(`${key}:runId`, () => null);
  /// The conversation being shown. Null before the owner has said
  /// anything: the gateway opens one on the first turn, so until then
  /// there is genuinely nothing to name or come back to.
  const sessionId = useState<string | null>(`${key}:sessionId`, () => null);
  /// Set while the runtime is holding a command: the orb keeps thinking
  /// and the input stays out of the way until the owner answers.
  const awaitingApproval = useState<boolean>(
    `${key}:awaitingApproval`,
    () => false,
  );

  // Orb rhythm is the only status indicator: thinking between send and
  // first token, speaking while tokens flow, idle otherwise.
  const orbState = computed<OrbState>(() =>
    busy.value ? (streaming.value ? "speaking" : "thinking") : "idle",
  );

  async function connect() {
    if (ready.value) return;
    try {
      await waitForAgentReady();
      ready.value = true;
    } catch (error) {
      daemonError.value = agentErrorMessage(surface, error);
    }
  }

  /// The live assistant turn, or null once something else closed it.
  function liveReply(index: number) {
    const turn = entries.value[index];
    return turn && turn.kind === "assistant" ? turn : null;
  }

  function handleEvent(event: StreamEvent, reply: { index: number }) {
    if (event.type === "run") {
      runId.value = event.runId;
      return;
    }

    if (event.type === "token") {
      streaming.value = true;
      const turn = liveReply(reply.index);
      if (turn) turn.content += event.content;
      return;
    }

    if (event.type === "tool") {
      const summary = toolSummary(event.name);
      if (event.phase === "started") {
        const turn = liveReply(reply.index);
        const row: Turn = {
          kind: "tool",
          name: event.name,
          summary,
          phase: "started",
        };
        if (turn && turn.content !== "") {
          // The device said something, then went and did something: the
          // row belongs after what it said, and anything it says next
          // belongs after the row.
          entries.value.push(row, { kind: "assistant", content: "" });
          reply.index = entries.value.length - 1;
        } else {
          // Nothing said yet -- the row goes where the reply is waiting.
          entries.value.splice(reply.index, 0, row);
          reply.index += 1;
        }
        return;
      }
      // Close the most recent row for that tool that is still running.
      for (let i = entries.value.length - 1; i >= 0; i -= 1) {
        const turn = entries.value[i];
        if (turn?.kind === "tool" && turn.name === event.name && turn.phase === "started") {
          turn.phase = event.phase;
          return;
        }
      }
      return;
    }

    if (event.type === "approval") {
      awaitingApproval.value = true;
      entries.value.splice(reply.index, 0, {
        kind: "approval",
        runId: runId.value ?? "",
        description: event.description,
        command: event.command,
        choices: event.choices,
        answer: null,
      });
      reply.index += 1;
      return;
    }

    if (event.type === "error") {
      entries.value[reply.index] = {
        kind: "error",
        content: streamErrorMessage(event.message),
      };
      return;
    }
    // "done" and "cancelled" are bookkeeping -- the turn ending is
    // already visible in the orb settling.
  }

  /// Runs one turn. `content` is what the owner said; setup's opening
  /// turn has nothing to say and lets the agent speak first.
  ///
  /// Returns whatever the command resolved to, which is how setup learns
  /// its interview is finished.
  async function runTurn(content: string | null): Promise<boolean> {
    busy.value = true;
    streaming.value = false;
    awaitingApproval.value = false;
    if (content !== null) entries.value.push({ kind: "user", content });

    // Only the new turn is sent: the Hermes gateway owns the
    // conversation history server-side, scoped to the session the Rust
    // layer holds.
    const before = entries.value.length;
    entries.value.push({ kind: "assistant", content: "" });
    const reply = { index: entries.value.length - 1 };

    const onEvent = new Channel<StreamEvent>();
    onEvent.onmessage = (event) => handleEvent(event, reply);

    try {
      if (mode === "onboarding") {
        return await invoke<boolean>("agent_onboarding_chat", {
          input: content,
          questionCount: await getOnboardingQuestionCount(),
          onEvent,
        });
      }

      // Whatever was selected when they pressed send, and where they
      // were looking. Captured now and cleared immediately: the turn is
      // about what they were looking at then, and leaving the chip up
      // would suggest it applies to the next question too.
      const { paths, folder, clear: clearContext } = useContext();
      const contextPaths = [...paths.value];
      const currentFolder = folder.value;
      clearContext();

      await invoke("agent_chat", {
        input: content ?? "",
        // The native layer turns these into a sentence: where the owner
        // is and which thing they mean, written from their own files
        // downward and never absolutely.
        contextPaths: contextPaths.length > 0 ? contextPaths : null,
        currentFolder: currentFolder === "" ? null : currentFolder,
        onEvent,
      });
      return false;
    } finally {
      // A turn that produced nothing at all is worth saying out loud
      // rather than leaving a blank line. A turn that said nothing but
      // *did* something is not: a permission card or a row saying what
      // the device did is the turn's content, and calling that "no
      // response" contradicts what the owner is looking at.
      const turn = liveReply(reply.index);
      if (turn && turn.content === "") {
        entries.value.splice(reply.index, 1);
        const produced = entries.value.slice(before);
        if (produced.length === 0) {
          entries.value.push({
            kind: "error",
            content: "The assistant returned no response.",
          });
        }
      }
      busy.value = false;
      streaming.value = false;
      awaitingApproval.value = false;
      runId.value = null;
      // The gateway opens a conversation on the first turn, so this is
      // where the shell finds out which one it is now in -- and what the
      // list has to highlight.
      if (mode === "chat" && sessionId.value === null) {
        try {
          sessionId.value = await invoke<string | null>("sessions_active");
        } catch {
          // Only the highlight in the list is affected.
        }
      }
    }
  }

  /// A stored transcript, drawn as the turns it was drawn as the first
  /// time. Tool rows arrive carrying only the runtime's own name for the
  /// tool; the sentence the owner reads is added here, from the same
  /// table live rows go through, so a reopened conversation and a fresh
  /// one cannot describe the same action differently.
  function hydrate(turns: { kind: string; content?: string; name?: string }[]) {
    entries.value = turns.map((turn) =>
      turn.kind === "tool"
        ? ({
            kind: "tool",
            name: turn.name ?? "",
            summary: toolSummary(turn.name ?? ""),
            phase: "completed",
          } as Turn)
        : ({ kind: turn.kind, content: turn.content ?? "" } as Turn),
    );
  }

  /// Shows a conversation the owner picked, and makes it the one the
  /// next turn continues.
  async function openSession(id: string) {
    if (busy.value) return;
    try {
      const turns = await invoke<{ kind: string; content?: string; name?: string }[]>(
        "sessions_open",
        { sessionId: id },
      );
      hydrate(turns);
      sessionId.value = id;
    } catch (error) {
      entries.value = [
        { kind: "error", content: agentErrorMessage(surface, error) },
      ];
    }
  }

  /// Clears the pane for a new subject. Nothing is created until the
  /// owner says something -- an empty conversation in their list would be
  /// one they never had.
  async function newConversation() {
    if (busy.value) return;
    try {
      await invoke("sessions_new");
    } catch (error) {
      entries.value.push({
        kind: "error",
        content: agentErrorMessage(surface, error),
      });
      return;
    }
    entries.value = [];
    input.value = "";
    sessionId.value = null;
  }

  /// Puts the device back where its owner left it. Run once as the pane
  /// mounts: a device that sits on a counter is picked up mid-thought,
  /// and starting every launch on an empty pane silently discards what
  /// was being talked about.
  ///
  /// Failing here is not worth saying out loud. The owner asked for
  /// nothing; an empty pane they can type into is a working device, and
  /// an error line about a conversation they had not asked to see would
  /// be the first thing they read on switching it on.
  async function restore() {
    if (entries.value.length > 0 || busy.value) return;
    try {
      const id = await invoke<string | null>("sessions_active");
      if (!id) return;
      const turns = await invoke<{ kind: string; content?: string; name?: string }[]>(
        "sessions_open",
        { sessionId: id },
      );
      hydrate(turns);
      sessionId.value = id;
    } catch {
      // Left as a fresh conversation.
    }
  }

  /// The conversations the owner can go back to, newest first.
  async function listSessions(offset = 0) {
    return await invoke<{ sessions: Conversation[]; hasMore: boolean }>(
      "sessions_list",
      { limit: 30, offset },
    );
  }

  /// Sends what is typed, or an answer the owner tapped instead.
  ///
  /// Resolves to whether this turn ended the conversation, which only
  /// setup asks about -- normal chat has no end.
  async function send(spoken?: string): Promise<boolean> {
    const content = (spoken ?? input.value).trim();
    if (!content || busy.value) return false;
    input.value = "";
    try {
      return await runTurn(content);
    } catch (error) {
      entries.value.push({
        kind: "error",
        content: agentErrorMessage(surface, error),
      });
      return false;
    }
  }

  /// Answers a permission request. The gateway's own vocabulary goes
  /// back to it; the owner saw words instead.
  async function answerApproval(turn: Turn, choice: string) {
    if (turn.kind !== "approval" || turn.answer !== null) return;
    try {
      await invoke("agent_approve", { runId: turn.runId, choice });
      turn.answer = choice;
      awaitingApproval.value = false;
    } catch (error) {
      entries.value.push({
        kind: "error",
        content: agentErrorMessage(surface, error),
      });
    }
  }

  /// Interrupts a turn the owner no longer wants to wait for.
  async function stop() {
    if (!runId.value) return;
    try {
      await invoke("agent_stop", { runId: runId.value });
    } catch (error) {
      entries.value.push({
        kind: "error",
        content: agentErrorMessage(surface, error),
      });
    }
  }

  return {
    entries,
    input,
    busy,
    streaming,
    daemonError,
    awaitingApproval,
    runId,
    sessionId,
    orbState,
    connect,
    runTurn,
    send,
    answerApproval,
    stop,
    restore,
    openSession,
    newConversation,
    listSessions,
  };
}
