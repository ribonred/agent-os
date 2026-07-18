<script setup lang="ts">
// The third and final setup step (design/DESIGN.md "Naming screen"):
// the owner christens the device. The product's first free-text input
// -- no suggestions, no placeholder names; the name is the owner's
// first act of ownership, not a menu choice.
import { AGENT_NAME_MAX_LENGTH } from "~/lib/setupOptions";
import { setAgentName } from "~/lib/setupStore";

const name = ref("");
const saving = ref(false);

const ready = computed(() => name.value.trim() !== "");

async function choose() {
  if (!ready.value || saving.value) return;
  saving.value = true;
  await setAgentName(name.value);
  await navigateTo("/");
}
</script>

<template>
  <main>
    <header class="enter-fade">
      <PresenceOrb :size="72" />
      <h1>What will you call me?</h1>
      <p class="eyebrow">Beri saya nama · Give me a name</p>
    </header>

    <form class="enter-rise" @submit.prevent="choose">
      <input
        v-model="name"
        type="text"
        :maxlength="AGENT_NAME_MAX_LENGTH"
        :disabled="saving"
        autocomplete="off"
        autofocus
        aria-label="The name you give this assistant"
      />
      <button type="submit" :disabled="!ready || saving">Continue</button>
    </form>
  </main>
</template>

<style scoped>
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

form {
  width: 100%;
  max-width: 420px;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 1rem;
}

input {
  width: 100%;
  background: var(--surface);
  color: var(--text-primary);
  border: 1px solid rgba(255, 255, 255, 0.06);
  border-radius: 12px;
  padding: 0.9rem 1.1rem;
  font-family: var(--font-family);
  font-size: 1.05rem;
  text-align: center;
}

input:focus-visible {
  outline: 2px solid var(--accent);
  outline-offset: 1px;
}

input:disabled {
  opacity: 0.55;
}

button {
  background: var(--surface);
  color: var(--text-primary);
  border: 1px solid color-mix(in srgb, var(--accent) 55%, transparent);
  border-radius: 12px;
  padding: 0.7rem 2.2rem;
  font-family: var(--font-family);
  font-size: 0.95rem;
  cursor: pointer;
  transition:
    border-color 0.15s,
    background 0.15s,
    opacity 0.15s;
}

button:hover:not(:disabled) {
  background: var(--surface-raised);
}

button:focus-visible {
  outline: 2px solid var(--accent);
  outline-offset: 2px;
}

button:disabled {
  opacity: 0.4;
  cursor: default;
}

.enter-fade {
  animation: fade-in 0.7s both;
}

.enter-rise {
  animation: rise-in 0.45s both;
  animation-delay: 250ms;
}

@keyframes fade-in {
  from {
    opacity: 0;
  }
  to {
    opacity: 1;
  }
}

@keyframes rise-in {
  from {
    opacity: 0;
    transform: translateY(14px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

@media (prefers-reduced-motion: reduce) {
  .enter-fade,
  .enter-rise {
    animation: none;
  }
}
</style>
