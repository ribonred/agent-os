<script lang="ts">
  import { fade, fly } from "svelte/transition";
  import PresenceOrb from "$lib/components/PresenceOrb.svelte";

  // Idle/home screen -- the orb at rest on its surface. Touching the orb
  // (or the invitation) opens the conversation; the whole center of the
  // screen is the way in, no button chrome.
  let mounted = $state(false);
  $effect(() => {
    mounted = true;
  });
</script>

<main>
  {#if mounted}
    <a class="wake" href="/chat" aria-label="Start a conversation">
      <div in:fade={{ duration: 900 }}>
        <PresenceOrb size={180} grounded />
      </div>
      <p class="invitation" in:fly={{ y: 10, duration: 700, delay: 500 }}>
        Ready when you are.
      </p>
    </a>
    <a class="cloud-link" href="/settings/cloud" in:fade={{ delay: 900 }}>
      Cloud settings
    </a>
  {/if}
</main>

<style>
  main {
    min-height: 100vh;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 4.5rem;
    background: radial-gradient(
      ellipse 90% 70% at 50% 40%,
      transparent 55%,
      rgba(0, 0, 0, 0.35) 100%
    );
  }

  .wake {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 4.5rem;
    text-decoration: none;
    cursor: pointer;
  }

  .invitation {
    margin: 0;
    color: var(--text-secondary);
    font-size: 0.95rem;
    font-weight: 300;
    letter-spacing: 0.06em;
    transition: color 0.3s;
  }

  .wake:hover .invitation,
  .wake:focus-visible .invitation {
    color: var(--text-primary);
  }

  .cloud-link {
    position: fixed;
    right: 1.5rem;
    bottom: 1.25rem;
    color: color-mix(in srgb, var(--text-secondary) 60%, transparent);
    font-size: 0.78rem;
    letter-spacing: 0.08em;
    text-decoration: none;
  }

  .cloud-link:hover,
  .cloud-link:focus-visible {
    color: var(--text-primary);
  }
</style>
