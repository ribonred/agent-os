<script lang="ts">
  import { onMount } from "svelte";
  import { fade, fly } from "svelte/transition";
  import { goto } from "$app/navigation";
  import PresenceOrb from "$lib/components/PresenceOrb.svelte";
  import { LANGUAGE_OPTIONS } from "$lib/setupOptions";
  import { setLanguage } from "$lib/setupStore";

  // The device can't know the user's language yet -- so it greets in all
  // of them, Indonesian first (see DESIGN.md "First-boot greeting").
  const GREETINGS = [
    "Halo",
    "Hello",
    "你好",
    "こんにちは",
    "안녕하세요",
    "Xin chào",
    "สวัสดี",
    "Selamat datang",
    "Kumusta",
    "नमस्ते",
  ];

  let greetingIndex = $state(0);
  let selecting = $state<string | null>(null);
  let mounted = $state(false);

  onMount(() => {
    mounted = true;
    const timer = setInterval(() => {
      greetingIndex = (greetingIndex + 1) % GREETINGS.length;
    }, 2200);
    return () => clearInterval(timer);
  });

  async function choose(code: (typeof LANGUAGE_OPTIONS)[number]["code"]) {
    selecting = code;
    await setLanguage(code);
    await goto("/setup/persona");
  }
</script>

<main>
  {#if mounted}
    <header in:fade={{ duration: 700 }}>
      <PresenceOrb size={72} />
      <div class="greeting" aria-live="off">
        {#key greetingIndex}
          <h1 in:fade={{ duration: 450 }} out:fade={{ duration: 300 }}>
            {GREETINGS[greetingIndex]}
          </h1>
        {/key}
      </div>
      <p class="eyebrow">Pilih bahasa · Choose your language</p>
    </header>

    <ul>
      {#each LANGUAGE_OPTIONS as option, i (option.code)}
        <li in:fly={{ y: 14, duration: 450, delay: 250 + i * 45 }}>
          <button
            type="button"
            disabled={selecting !== null}
            onclick={() => choose(option.code)}
          >
            <span class="native">{option.nativeLabel}</span>
            <span class="label">{option.label}</span>
          </button>
        </li>
      {/each}
    </ul>
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

  /* Fixed height so the greeting swap never shifts the layout below it */
  .greeting {
    position: relative;
    height: 3.2rem;
    min-width: 20rem;
    display: grid;
    place-items: center;
  }

  .greeting h1 {
    position: absolute;
    margin: 0;
    font-size: 2.4rem;
    font-weight: 200;
    letter-spacing: 0.02em;
    color: var(--text-primary);
  }

  .eyebrow {
    margin: 0;
    color: var(--text-secondary);
    font-size: 0.78rem;
    font-weight: 500;
    letter-spacing: 0.22em;
    text-transform: uppercase;
  }

  ul {
    list-style: none;
    margin: 0;
    padding: 0;
    width: 100%;
    max-width: 420px;
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  button {
    width: 100%;
    display: flex;
    justify-content: space-between;
    align-items: baseline;
    background: var(--surface);
    color: var(--text-primary);
    border: 1px solid rgba(255, 255, 255, 0.04);
    border-radius: 10px;
    padding: 0.9rem 1.1rem;
    font-family: var(--font-family);
    font-size: 1rem;
    cursor: pointer;
    transition:
      border-color 0.15s,
      background 0.15s,
      transform 0.15s;
  }

  button:hover:not(:disabled) {
    border-color: color-mix(in srgb, var(--accent) 55%, transparent);
    background: var(--surface-raised);
    transform: translateY(-1px);
  }

  button:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: 2px;
  }

  button:disabled {
    opacity: 0.5;
    cursor: default;
  }

  .native {
    font-weight: 500;
  }

  .label {
    color: var(--text-secondary);
    font-size: 0.85rem;
  }

  @media (prefers-reduced-motion: reduce) {
    .greeting h1 {
      transition: none;
    }
  }
</style>
