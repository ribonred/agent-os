<script setup lang="ts">
import { invoke } from "@tauri-apps/api/core";
import { agentErrorMessage } from "~/lib/agentErrors";

definePageMeta({ layout: false });

// Speech is a separate decision from where the device thinks, and a
// separate key. A device can perfectly well think on its own hardware
// and still speak, or think in the cloud and stay silent -- so this is
// its own screen rather than a second field on the cloud one.
//
// Same posture as that screen and enforced in the same place: the key
// never comes back from the backend, only which source is active. See
// src-tauri/src/cloud_key.rs.
type KeyStatus = "none" | "keyring" | "provisioned";

const status = ref<KeyStatus | null>(null); // null = still checking
const keyInput = ref("");
const busy = ref(false);
const errorMessage = ref<string | null>(null);

const { configured, setMicMode } = useVoice();

async function refreshStatus() {
  try {
    status.value = await invoke<KeyStatus>("voice_key_status");
  } catch (e) {
    status.value = "none";
    errorMessage.value = agentErrorMessage("cloudKey", e);
  }
  // The control beside the composer appears and disappears with this, so
  // it is told rather than left to find out on the next launch.
  configured.value = status.value !== "none";
}

onMounted(refreshStatus);

async function save() {
  busy.value = true;
  errorMessage.value = null;
  try {
    await invoke("voice_key_save", { key: keyInput.value });
    keyInput.value = "";
    await refreshStatus();
  } catch (e) {
    errorMessage.value = agentErrorMessage("cloudKey", e);
  } finally {
    busy.value = false;
  }
}

async function disconnect() {
  busy.value = true;
  errorMessage.value = null;
  try {
    await invoke("voice_key_delete");
    // Falls back to a provisioned key if the device ships one -- the
    // status reflects whatever is actually active now.
    await refreshStatus();
    // Nothing can be spoken any more, so the device stops offering to.
    if (status.value === "none") await setMicMode(false);
  } catch (e) {
    errorMessage.value = agentErrorMessage("cloudKey", e);
  } finally {
    busy.value = false;
  }
}
</script>

<template>
  <main>
    <header class="enter-fade">
      <PresenceOrb :size="72" />
      <h1>Talking out loud</h1>
      <p class="eyebrow">Bicara · Optional</p>
    </header>

    <section class="enter-rise">
      <p class="explain">
        With this connected you can hold a button and speak to me instead
        of typing, and I'll answer out loud. Everything I say is still
        written down, so nothing is lost if you'd rather read it.
      </p>

      <template v-if="status === 'keyring'">
        <div class="status">
          <span class="status-dot" aria-hidden="true"></span>
          <span>I can listen and speak.</span>
        </div>
        <button
          type="button"
          class="secondary"
          :disabled="busy"
          @click="disconnect"
        >
          Disconnect and remove the key
        </button>
      </template>

      <template v-else-if="status === 'provisioned'">
        <div class="status">
          <span class="status-dot" aria-hidden="true"></span>
          <span>Speech came ready to use on this device.</span>
        </div>
        <form @submit.prevent="save">
          <input
            v-model="keyInput"
            type="password"
            placeholder="Use your own key instead"
            :disabled="busy"
            autocomplete="off"
          />
          <button type="submit" :disabled="busy || keyInput.trim() === ''">
            Replace
          </button>
        </form>
      </template>

      <template v-else-if="status === 'none'">
        <form @submit.prevent="save">
          <input
            v-model="keyInput"
            type="password"
            placeholder="ElevenLabs API key"
            :disabled="busy"
            autocomplete="off"
          />
          <button type="submit" :disabled="busy || keyInput.trim() === ''">
            Connect
          </button>
        </form>
      </template>

      <p v-if="errorMessage" class="error" role="alert">{{ errorMessage }}</p>

      <!-- Said here rather than discovered at the counter: the mini-PC
           this ships on has no microphone of its own. -->
      <p class="note">
        You'll also need a microphone plugged in — a headset or a USB
        microphone. I'll tell you if I can't hear one.
      </p>

      <NuxtLink class="back" to="/settings">Back</NuxtLink>
    </section>
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

.note {
  margin: 0;
  color: var(--text-secondary);
  font-size: 0.82rem;
  line-height: 1.6;
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

.enter-fade {
  animation: fade-in 0.7s both;
}

.enter-rise {
  animation: rise-in 0.45s 0.25s both;
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
