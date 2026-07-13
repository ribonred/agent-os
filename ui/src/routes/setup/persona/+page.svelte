<script lang="ts">
  import { onMount } from "svelte";
  import { fade, fly } from "svelte/transition";
  import { goto } from "$app/navigation";
  import PresenceOrb from "$lib/components/PresenceOrb.svelte";
  import { PERSONA_OPTIONS } from "$lib/setupOptions";
  import { setPersona } from "$lib/setupStore";

  let selecting = $state<string | null>(null);
  let mounted = $state(false);

  onMount(() => {
    mounted = true;
  });

  async function choose(id: (typeof PERSONA_OPTIONS)[number]["id"]) {
    selecting = id;
    await setPersona(id);
    await goto("/");
  }
</script>

<main>
  {#if mounted}
    <header in:fade={{ duration: 700 }}>
      <PresenceOrb size={72} />
      <h1>How should I be with you?</h1>
      <p class="eyebrow">Pilih kepribadian · Choose a personality</p>
    </header>

    <div class="cards">
      {#each PERSONA_OPTIONS as option, i (option.id)}
        <button
          type="button"
          class="card"
          class:recommended={option.id === "balanced"}
          disabled={selecting !== null}
          onclick={() => choose(option.id)}
          in:fly={{ y: 14, duration: 450, delay: 250 + i * 70 }}
        >
          {#if option.id === "balanced"}
            <span class="badge">Recommended</span>
          {/if}
          <span class="label">{option.label}</span>
          <span class="description">{option.description}</span>
        </button>
      {/each}
    </div>
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

  .cards {
    width: 100%;
    max-width: 640px;
    display: grid;
    grid-template-columns: repeat(2, 1fr);
    gap: 0.75rem;
  }

  @media (max-width: 560px) {
    .cards {
      grid-template-columns: 1fr;
    }
  }

  .card {
    position: relative;
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    gap: 0.4rem;
    text-align: left;
    background: var(--surface);
    color: var(--text-primary);
    border: 1px solid rgba(255, 255, 255, 0.04);
    border-radius: 12px;
    padding: 1.25rem;
    font-family: var(--font-family);
    cursor: pointer;
    transition:
      border-color 0.15s,
      background 0.15s,
      transform 0.15s;
  }

  .card:hover:not(:disabled) {
    border-color: color-mix(in srgb, var(--accent) 55%, transparent);
    background: var(--surface-raised);
    transform: translateY(-1px);
  }

  .card:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: 2px;
  }

  .card:disabled {
    opacity: 0.5;
    cursor: default;
  }

  .card.recommended {
    border-color: color-mix(in srgb, var(--accent-warm) 55%, transparent);
  }

  .badge {
    position: absolute;
    top: 0.9rem;
    right: 1rem;
    color: var(--accent-warm);
    font-size: 0.68rem;
    font-weight: 600;
    letter-spacing: 0.14em;
    text-transform: uppercase;
  }

  .label {
    font-weight: 600;
    font-size: 1.05rem;
  }

  .description {
    color: var(--text-secondary);
    font-size: 0.85rem;
    line-height: 1.4;
  }
</style>
