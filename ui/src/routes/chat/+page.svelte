<script lang="ts">
  import { onMount, tick } from "svelte";
  import { fade } from "svelte/transition";
  import { invoke, Channel } from "@tauri-apps/api/core";
  import PresenceOrb from "$lib/components/PresenceOrb.svelte";

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

  let mounted = $state(false);
  let entries = $state<Entry[]>([]);
  let input = $state("");
  let busy = $state(false);
  let streaming = $state(false);
  let daemonError = $state<string | null>(null);
  let scroller: HTMLElement | undefined = $state();

  // Orb rhythm is the only status indicator: thinking between send and
  // first token, speaking while tokens flow, idle otherwise.
  let orbState = $derived(
    busy ? (streaming ? "speaking" : "thinking") : "idle",
  ) as "idle" | "thinking" | "speaking";

  onMount(async () => {
    mounted = true;
    try {
      await invoke("agent_status");
    } catch (e) {
      daemonError = String(e);
    }
  });

  async function autoscroll() {
    await tick();
    scroller?.scrollTo({ top: scroller.scrollHeight });
  }

  async function send() {
    const content = input.trim();
    if (!content || busy) return;
    input = "";
    busy = true;
    streaming = false;
    entries = [...entries, { kind: "user", content }];
    await autoscroll();

    // Full turn history goes to the daemon -- it owns the system prompt;
    // error entries are UI-local and never sent back as context.
    const messages = entries
      .filter((e) => e.kind !== "error")
      .map((e) => ({
        role: e.kind === "user" ? "user" : "assistant",
        content: e.content,
      }));

    const reply: Entry = { kind: "assistant", content: "" };
    entries = [...entries, reply];
    const replyIndex = entries.length - 1;

    const onEvent = new Channel<StreamEvent>();
    onEvent.onmessage = (event) => {
      if (event.type === "token") {
        streaming = true;
        entries[replyIndex].content += event.content;
        autoscroll();
      } else if (event.type === "error") {
        entries[replyIndex] = { kind: "error", content: event.message };
        autoscroll();
      }
      // "done" carries backend/model -- routing is disclosed on request
      // only (constitution.md), so the UI reads it and shows nothing.
    };

    try {
      await invoke("agent_chat", { messages, onEvent });
      // An empty reply with no error event means the stream never
      // produced content -- say so rather than leaving a blank line.
      if (
        entries[replyIndex].kind === "assistant" &&
        entries[replyIndex].content === ""
      ) {
        entries[replyIndex] = {
          kind: "error",
          content: "The assistant returned no response.",
        };
      }
    } catch (e) {
      entries[replyIndex] = { kind: "error", content: String(e) };
    } finally {
      busy = false;
      streaming = false;
      await autoscroll();
    }
  }
</script>

<main>
  {#if mounted}
    <header in:fade={{ duration: 500 }}>
      <a class="back" href="/" aria-label="Back to home">‹</a>
      <PresenceOrb size={48} state={orbState} />
    </header>

    <section class="conversation" bind:this={scroller}>
      {#if daemonError}
        <p class="error" role="alert">{daemonError}</p>
      {:else if entries.length === 0}
        <p class="empty" in:fade={{ duration: 700, delay: 200 }}>
          Ask me anything.
        </p>
      {/if}
      {#each entries as entry}
        {#if entry.kind === "user"}
          <p class="user">{entry.content}</p>
        {:else if entry.kind === "assistant"}
          <p class="assistant">{entry.content}</p>
        {:else}
          <p class="error" role="alert">{entry.content}</p>
        {/if}
      {/each}
    </section>

    <form
      onsubmit={(e) => {
        e.preventDefault();
        send();
      }}
    >
      <input
        type="text"
        placeholder="Say something…"
        bind:value={input}
        disabled={busy || daemonError !== null}
        autocomplete="off"
      />
    </form>
  {/if}
</main>

<style>
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
</style>
