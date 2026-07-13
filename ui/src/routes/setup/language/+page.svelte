<script lang="ts">
  import { goto } from "$app/navigation";
  import { LANGUAGE_OPTIONS } from "$lib/setupOptions";
  import { setLanguage } from "$lib/setupStore";

  let selecting = $state<string | null>(null);

  async function choose(code: (typeof LANGUAGE_OPTIONS)[number]["code"]) {
    selecting = code;
    await setLanguage(code);
    await goto("/setup/persona");
  }
</script>

<main>
  <ul>
    {#each LANGUAGE_OPTIONS as option (option.code)}
      <li>
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
</main>

<style>
  main {
    min-height: 100vh;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 2rem;
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
    border: 1px solid transparent;
    border-radius: 10px;
    padding: 0.9rem 1.1rem;
    font-family: var(--font-family);
    font-size: 1rem;
    cursor: pointer;
    transition:
      border-color 0.15s,
      background 0.15s;
  }

  button:hover:not(:disabled) {
    border-color: var(--accent);
    background: var(--surface-raised);
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
</style>
