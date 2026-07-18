<script setup lang="ts">
import { LANGUAGE_OPTIONS } from "~/lib/setupOptions";
import { setLanguage } from "~/lib/setupStore";

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

const greetingIndex = ref(0);
const selecting = ref<string | null>(null);
let timer: ReturnType<typeof setInterval> | undefined;

onMounted(() => {
  timer = setInterval(() => {
    greetingIndex.value = (greetingIndex.value + 1) % GREETINGS.length;
  }, 2200);
});
onUnmounted(() => clearInterval(timer));

async function choose(code: (typeof LANGUAGE_OPTIONS)[number]["code"]) {
  selecting.value = code;
  await setLanguage(code);
  await navigateTo("/setup/name");
}
</script>

<template>
  <main>
    <header class="enter-fade">
      <PresenceOrb :size="72" />
      <div class="greeting" aria-live="off">
        <Transition name="gfade" mode="out-in">
          <h1 :key="greetingIndex">{{ GREETINGS[greetingIndex] }}</h1>
        </Transition>
      </div>
      <p class="eyebrow">Pilih bahasa · Choose your language</p>
    </header>

    <ul>
      <li
        v-for="(option, i) in LANGUAGE_OPTIONS"
        :key="option.code"
        class="enter-rise"
        :style="{ animationDelay: `${250 + i * 45}ms` }"
      >
        <button
          type="button"
          :disabled="selecting !== null"
          @click="choose(option.code)"
        >
          <span class="native">{{ option.nativeLabel }}</span>
          <span class="label">{{ option.label }}</span>
        </button>
      </li>
    </ul>
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

.gfade-enter-active {
  transition: opacity 0.45s;
}
.gfade-leave-active {
  transition: opacity 0.3s;
}
.gfade-enter-from,
.gfade-leave-to {
  opacity: 0;
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

.enter-fade {
  animation: fade-in 0.7s both;
}

.enter-rise {
  animation: rise-in 0.45s both;
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
  .gfade-enter-active,
  .gfade-leave-active {
    transition: none;
  }
}
</style>
