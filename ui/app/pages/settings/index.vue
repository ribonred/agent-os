<script setup lang="ts">
// The few things about this device the owner can decide, written as
// decisions rather than as settings. There is no settings *system* here
// on purpose: everything else about the device is changed by asking it.

import { invoke } from "@tauri-apps/api/core";

definePageMeta({ layout: false });

const asking = ref(false);
const saving = ref(false);
const failure = ref<string | null>(null);

/// Held down for a few seconds to reach the development reset. Not a
/// hidden feature so much as one nobody can reach by accident: it
/// discards setup, and the command behind it does not exist in a
/// shipped build at all.
const RESET_HOLD_MS = 3000;
const resetting = ref(false);
const resetDone = ref(false);
let holdTimer: ReturnType<typeof setTimeout> | null = null;

onMounted(async () => {
  asking.value = await invoke<boolean>("approval_mode_get");
});

async function setAsking(next: boolean) {
  saving.value = true;
  failure.value = null;
  try {
    await invoke("approval_mode_set", { enabled: next });
    asking.value = next;
  } catch {
    // Errors are spoken where the failure is, never as a toast.
    failure.value = "I couldn't change that. Nothing has been altered.";
  } finally {
    saving.value = false;
  }
}

function startHold() {
  holdTimer = setTimeout(async () => {
    resetting.value = true;
    try {
      await invoke("dev_reset_setup");
      resetDone.value = true;
    } catch {
      // Silent on a shipped device: the command refuses there, and
      // there is nothing the owner asked for to report on.
    } finally {
      resetting.value = false;
    }
  }, RESET_HOLD_MS);
}

function endHold() {
  if (holdTimer) clearTimeout(holdTimer);
  holdTimer = null;
}

onBeforeUnmount(endHold);
</script>

<template>
  <main>
    <header>
      <NuxtLink to="/" class="back" aria-label="Go back">‹</NuxtLink>
      <h1>Settings</h1>
    </header>

    <section class="setting">
      <div class="text">
        <h2>Ask me before doing anything risky</h2>
        <p>
          I'll stop and check with you before running anything that could
          delete or change things on this device. Off by default, because
          most of what I do is ordinary and stopping for all of it would
          waste your time.
        </p>
      </div>
      <button
        type="button"
        role="switch"
        :aria-checked="asking"
        :class="['switch', { on: asking }]"
        :disabled="saving"
        @click="setAsking(!asking)"
      >
        <span class="knob" />
      </button>
    </section>
    <p v-if="failure" class="error" role="alert">{{ failure }}</p>

    <NuxtLink to="/settings/cloud" class="setting link">
      <div class="text">
        <h2>Where I think</h2>
        <p>Whether I use this device's own hardware or a service online.</p>
      </div>
      <span class="chevron" aria-hidden="true">›</span>
    </NuxtLink>

    <!-- Held, not clicked, and unlabelled: the command behind it does
         not exist in a shipped build. -->
    <button
      type="button"
      class="hold"
      :aria-label="resetDone ? 'Setup cleared' : 'Device'"
      @mousedown="startHold"
      @mouseup="endHold"
      @mouseleave="endHold"
      @touchstart.passive="startHold"
      @touchend="endHold"
    >
      <span v-if="resetDone">Setup cleared. Restart to begin again.</span>
      <span v-else-if="resetting">…</span>
      <span v-else>agentic-os</span>
    </button>
  </main>
</template>

<style scoped>
main {
  height: 100vh;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  gap: 1rem;
  max-width: 640px;
  margin: 0 auto;
  padding: 2rem 1.5rem 3rem;
}

header {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  margin-bottom: 0.5rem;
}

h1 {
  margin: 0;
  font-size: 1.35rem;
  font-weight: 300;
  letter-spacing: 0.01em;
  color: var(--text-primary);
}

.back {
  color: var(--text-secondary);
  font-size: 1.6rem;
  line-height: 1;
  text-decoration: none;
  padding: 0 0.25rem;
}

.back:hover {
  color: var(--text-primary);
}

.setting {
  display: flex;
  align-items: center;
  gap: 1.25rem;
  background: var(--surface);
  border-radius: 14px;
  padding: 1.1rem 1.25rem;
  text-decoration: none;
}

.text {
  flex: 1;
  min-width: 0;
}

h2 {
  margin: 0 0 0.35rem;
  font-size: 1rem;
  font-weight: 500;
  color: var(--text-primary);
}

.text p {
  margin: 0;
  color: var(--text-secondary);
  font-size: 0.88rem;
  line-height: 1.55;
}

.switch {
  flex: 0 0 auto;
  width: 48px;
  height: 28px;
  border-radius: 999px;
  border: 1px solid color-mix(in srgb, var(--text-secondary) 30%, transparent);
  background: var(--bg);
  cursor: pointer;
  padding: 2px;
  display: flex;
  justify-content: flex-start;
  transition: background 0.18s ease, border-color 0.18s ease;
}

.switch.on {
  background: color-mix(in srgb, var(--accent) 30%, var(--bg));
  border-color: color-mix(in srgb, var(--accent) 55%, transparent);
  justify-content: flex-end;
}

.switch:disabled {
  opacity: 0.6;
  cursor: default;
}

.knob {
  width: 22px;
  height: 22px;
  border-radius: 50%;
  background: var(--text-secondary);
  transition: background 0.18s ease;
}

.switch.on .knob {
  background: var(--accent);
}

.chevron {
  color: var(--text-secondary);
  font-size: 1.3rem;
}

.error {
  margin: 0;
  color: var(--danger);
  font-size: 0.88rem;
}

.hold {
  margin-top: auto;
  align-self: center;
  background: none;
  border: none;
  color: color-mix(in srgb, var(--text-secondary) 55%, transparent);
  font-family: var(--font-family);
  font-size: 0.78rem;
  padding: 1rem;
  cursor: default;
  user-select: none;
}

.switch:focus-visible,
.link:focus-visible,
.back:focus-visible,
.hold:focus-visible {
  outline: 2px solid var(--accent);
  outline-offset: 2px;
  border-radius: 10px;
}

@media (prefers-reduced-motion: reduce) {
  .switch,
  .knob {
    transition: none;
  }
}
</style>
