import { Channel, invoke } from "@tauri-apps/api/core";
import { waitForAgentReady } from "~/lib/agentStatus";
import { agentErrorMessage, streamErrorMessage } from "~/lib/agentErrors";

// The conversation's state and streaming, owned by the shell rather than
// by a page. The pane is mounted once for the life of the shell, so a
// reply that started before the owner went looking through their things
// keeps arriving while they browse -- navigating the main area no longer
// tears the stream down.

// Named for what it is rather than a bare "Entry": the file view exports
// an Entry too, and Nuxt's auto-import silently resolves a collision to
// one of them.
export type Turn =
  | { kind: "user"; content: string }
  | { kind: "assistant"; content: string }
  | { kind: "error"; content: string };

type StreamEvent =
  | { type: "token"; content: string }
  | { type: "done" }
  | { type: "error"; message: string };

export type OrbState = "idle" | "thinking" | "speaking";

const STATE_KEY = "conversation";

export function useConversation() {
  // useState rather than module-level refs: one instance shared by every
  // consumer, without leaking across app instances.
  const entries = useState<Turn[]>(`${STATE_KEY}:entries`, () => []);
  const input = useState<string>(`${STATE_KEY}:input`, () => "");
  const busy = useState<boolean>(`${STATE_KEY}:busy`, () => false);
  const streaming = useState<boolean>(`${STATE_KEY}:streaming`, () => false);
  const daemonError = useState<string | null>(
    `${STATE_KEY}:daemonError`,
    () => null,
  );
  const ready = useState<boolean>(`${STATE_KEY}:ready`, () => false);

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
      daemonError.value = agentErrorMessage("chat", error);
    }
  }

  async function send() {
    const content = input.value.trim();
    if (!content || busy.value) return;

    // Whatever was selected when they pressed send. Captured now and
    // cleared immediately: the turn is about what they were looking at
    // then, and leaving the chip up would suggest it applies to the next
    // question too.
    const { paths, clear: clearContext } = useContext();
    const contextPaths = [...paths.value];
    clearContext();

    input.value = "";
    busy.value = true;
    streaming.value = false;
    entries.value.push({ kind: "user", content });

    // Only the new turn is sent: the Hermes gateway owns the conversation
    // history server-side, scoped to the session the Rust layer holds.
    entries.value.push({ kind: "assistant", content: "" });
    const replyIndex = entries.value.length - 1;

    const onEvent = new Channel<StreamEvent>();
    onEvent.onmessage = (event) => {
      const reply = entries.value[replyIndex];
      if (!reply) return;
      if (event.type === "token") {
        streaming.value = true;
        if (reply.kind === "assistant") reply.content += event.content;
      } else if (event.type === "error") {
        entries.value[replyIndex] = {
          kind: "error",
          content: streamErrorMessage(event.message),
        };
      }
      // "done" is bookkeeping only -- nothing to show.
    };

    try {
      await invoke("agent_chat", {
        input: content,
        // The native layer resolves these into names and folders; a path
        // never reaches the model, and never comes back out in a reply.
        contextPaths: contextPaths.length > 0 ? contextPaths : null,
        onEvent,
      });
      // An empty reply with no error event means the stream never
      // produced content -- say so rather than leaving a blank line.
      const reply = entries.value[replyIndex];
      if (reply && reply.kind === "assistant" && reply.content === "") {
        entries.value[replyIndex] = {
          kind: "error",
          content: "The assistant returned no response.",
        };
      }
    } catch (error) {
      entries.value[replyIndex] = {
        kind: "error",
        content: agentErrorMessage("chat", error),
      };
    } finally {
      busy.value = false;
      streaming.value = false;
    }
  }

  return { entries, input, busy, streaming, daemonError, orbState, connect, send };
}
