<script lang="ts">
  import { goto } from "$app/navigation";
  import { PERSONA_OPTIONS } from "$lib/setupOptions";
  import { setPersona } from "$lib/setupStore";

  let selecting = $state<string | null>(null);

  async function choose(id: (typeof PERSONA_OPTIONS)[number]["id"]) {
    selecting = id;
    await setPersona(id);
    await goto("/");
  }
</script>

<main>
  <div class="cards">
    {#each PERSONA_OPTIONS as option (option.id)}
      <button
        type="button"
        class="card"
        class:recommended={option.id === "balanced"}
        disabled={selecting !== null}
        onclick={() => choose(option.id)}
      >
        <span class="label">{option.label}</span>
        <span class="description">{option.description}</span>
      </button>
    {/each}
  </div>
</main>

<style>
  main {
    min-height: 100vh;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 2rem;
  }

  .cards {
    width: 100%;
    max-width: 640px;
    display: grid;
    grid-template-columns: repeat(2, 1fr);
    gap: 0.75rem;
  }

  .card {
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    gap: 0.4rem;
    text-align: left;
    background: var(--surface);
    color: var(--text-primary);
    border: 1px solid transparent;
    border-radius: 12px;
    padding: 1.25rem;
    font-family: var(--font-family);
    cursor: pointer;
    transition:
      border-color 0.15s,
      background 0.15s;
  }

  .card:hover:not(:disabled) {
    border-color: var(--accent);
    background: var(--surface-raised);
  }

  .card:disabled {
    opacity: 0.5;
    cursor: default;
  }

  .card.recommended {
    border-color: var(--accent-warm);
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
