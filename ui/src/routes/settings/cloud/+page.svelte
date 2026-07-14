<script lang="ts">
  import { onMount } from "svelte";
  import { fade, fly } from "svelte/transition";
  import { invoke } from "@tauri-apps/api/core";
  import { goto } from "$app/navigation";
  import PresenceOrb from "$lib/components/PresenceOrb.svelte";

  // The key itself never comes back from the backend -- only which
  // source is active. See src-tauri/src/cloud_key.rs for the posture:
  // "keyring" = the user's own key; "provisioned" = a key shipped with
  // the device (a file the UI cannot delete, only override); "none".
  type KeyStatus = "none" | "keyring" | "provisioned";

  let mounted = $state(false);
  let status = $state<KeyStatus | null>(null); // null = still checking
  let keyInput = $state("");
  let busy = $state(false);
  let errorMessage = $state<string | null>(null);

  async function refreshStatus() {
    try {
      status = await invoke<KeyStatus>("cloud_key_status");
    } catch (e) {
      status = "none";
      errorMessage = String(e);
    }
  }

  onMount(async () => {
    mounted = true;
    await refreshStatus();
  });

  async function save() {
    busy = true;
    errorMessage = null;
    try {
      await invoke("cloud_key_save", { key: keyInput });
      keyInput = "";
      await refreshStatus();
    } catch (e) {
      errorMessage = String(e);
    } finally {
      busy = false;
    }
  }

  async function disconnect() {
    busy = true;
    errorMessage = null;
    try {
      await invoke("cloud_key_delete");
      // Falls back to a provisioned key if the device ships one -- the
      // status reflects whatever is actually active now.
      await refreshStatus();
    } catch (e) {
      errorMessage = String(e);
    } finally {
      busy = false;
    }
  }
</script>

<main>
  {#if mounted}
    <header in:fade={{ duration: 700 }}>
      <PresenceOrb size={72} />
      <h1>Connect to the cloud</h1>
      <p class="eyebrow">Hubungkan cloud · Optional</p>
    </header>

    <section in:fly={{ y: 14, duration: 450, delay: 250 }}>
      <p class="explain">
        Your assistant works fully on this device without an internet
        account. Connecting an OpenRouter key lets it reach a larger model
        online when a task needs more than this device can do on its own.
      </p>

      {#if status === "keyring"}
        <div class="status" in:fade={{ duration: 300 }}>
          <span class="status-dot" aria-hidden="true"></span>
          <span>Cloud is connected.</span>
        </div>
        <button
          type="button"
          class="secondary"
          disabled={busy}
          onclick={disconnect}
        >
          Disconnect and remove the key
        </button>
      {:else if status === "provisioned"}
        <div class="status" in:fade={{ duration: 300 }}>
          <span class="status-dot" aria-hidden="true"></span>
          <span>Cloud came pre-configured on this device.</span>
        </div>
        <form
          onsubmit={(e) => {
            e.preventDefault();
            save();
          }}
        >
          <input
            type="password"
            placeholder="Use your own key instead"
            bind:value={keyInput}
            disabled={busy}
            autocomplete="off"
          />
          <button type="submit" disabled={busy || keyInput.trim() === ""}>
            Replace
          </button>
        </form>
      {:else if status === "none"}
        <form
          onsubmit={(e) => {
            e.preventDefault();
            save();
          }}
        >
          <input
            type="password"
            placeholder="OpenRouter API key"
            bind:value={keyInput}
            disabled={busy}
            autocomplete="off"
          />
          <button type="submit" disabled={busy || keyInput.trim() === ""}>
            Connect
          </button>
        </form>
      {/if}

      {#if errorMessage}
        <p class="error" role="alert">{errorMessage}</p>
      {/if}

      <a class="back" href="/">Back</a>
    </section>
  {/if}
</main>

<style>
  main {
    min-height: 100vh;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 2.25rem;
    padding: 3rem 2rem;
  }

  header {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 1.25rem;
  }

  h1 {
    margin: 0;
    font-size: 2.1rem;
    font-weight: 200;
    letter-spacing: 0.02em;
  }

  .eyebrow {
    margin: 0;
    color: var(--text-secondary);
    font-size: 0.78rem;
    font-weight: 500;
    letter-spacing: 0.22em;
    text-transform: uppercase;
  }

  section {
    width: 100%;
    max-width: 460px;
    display: flex;
    flex-direction: column;
    gap: 1.25rem;
  }

  .explain {
    margin: 0;
    color: var(--text-secondary);
    font-size: 0.92rem;
    line-height: 1.6;
    text-align: center;
  }

  form {
    display: flex;
    gap: 0.5rem;
  }

  input {
    flex: 1;
    background: var(--surface);
    color: var(--text-primary);
    border: 1px solid rgba(255, 255, 255, 0.06);
    border-radius: 10px;
    padding: 0.85rem 1rem;
    font-family: var(--font-family);
    font-size: 0.95rem;
  }

  input:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: 1px;
  }

  button {
    background: var(--surface-raised);
    color: var(--text-primary);
    border: 1px solid color-mix(in srgb, var(--accent) 45%, transparent);
    border-radius: 10px;
    padding: 0.85rem 1.4rem;
    font-family: var(--font-family);
    font-size: 0.95rem;
    font-weight: 500;
    cursor: pointer;
    transition:
      border-color 0.15s,
      background 0.15s;
  }

  button:hover:not(:disabled) {
    border-color: var(--accent);
  }

  button:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: 2px;
  }

  button:disabled {
    opacity: 0.5;
    cursor: default;
  }

  button.secondary {
    border-color: rgba(255, 255, 255, 0.1);
    align-self: center;
  }

  button.secondary:hover:not(:disabled) {
    border-color: var(--danger);
    color: var(--danger);
  }

  .status {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 0.5rem;
    color: var(--text-primary);
    font-size: 0.95rem;
  }

  .status-dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: var(--success);
    box-shadow: 0 0 8px color-mix(in srgb, var(--success) 60%, transparent);
  }

  .error {
    margin: 0;
    color: var(--danger);
    font-size: 0.88rem;
    text-align: center;
  }

  .back {
    align-self: center;
    color: var(--text-secondary);
    font-size: 0.85rem;
    text-decoration: none;
  }

  .back:hover {
    color: var(--text-primary);
  }
</style>
