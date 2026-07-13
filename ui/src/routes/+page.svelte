<script lang="ts">
  // Idle-state placeholder only -- the presence indicator's other states
  // (listening/thinking/speaking) and everything else on this screen
  // (mandatory language/persona setup, then the actual assistant UI) are
  // task 020 and beyond. This proves the design tokens render correctly
  // and nothing more.
</script>

<main>
  <div class="presence" aria-hidden="true"></div>
</main>

<style>
  main {
    height: 100vh;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .presence {
    width: 120px;
    height: 120px;
    border-radius: 50%;
    background: radial-gradient(circle, var(--accent) 0%, transparent 70%);
    animation: breathe 4s ease-in-out infinite;
  }

  @keyframes breathe {
    0%,
    100% {
      opacity: 0.5;
      transform: scale(0.9);
    }
    50% {
      opacity: 1;
      transform: scale(1.05);
    }
  }
</style>
