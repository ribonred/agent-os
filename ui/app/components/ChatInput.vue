<script setup lang="ts">
// One quiet box for what the owner is about to say. A native textarea
// rather than a rich editor: the job is line breaks and paste, not
// formatting chrome, and a candidate window for another script must
// still work.

import { shouldSubmitComposer } from "~/lib/composerKeys";

const model = defineModel<string>({ default: "" });

const props = withDefaults(
  defineProps<{
    placeholder?: string;
    disabled?: boolean;
    variant?: "pane" | "setup" | "pill";
    autofocus?: boolean;
  }>(),
  {
    placeholder: "",
    disabled: false,
    variant: "pane",
    autofocus: false,
  },
);

const emit = defineEmits<{ submit: [] }>();

const field = ref<HTMLTextAreaElement | null>(null);

function resize() {
  const el = field.value;
  if (!el) return;
  // Collapse first so deleting a line shrinks the box instead of
  // leaving the previous scrollHeight stuck on it.
  el.style.overflowY = "hidden";
  el.style.height = "auto";
  el.style.height = `${el.scrollHeight}px`;
  const max = Number.parseFloat(getComputedStyle(el).maxHeight);
  // A scrollbar on a one-line draft is chrome the owner did not ask
  // for. Only show one once the box has actually hit its ceiling.
  if (Number.isFinite(max) && el.scrollHeight > max + 1) {
    el.style.overflowY = "auto";
  }
}

watch(model, () => nextTick(resize));
onMounted(resize);

function onKeydown(event: KeyboardEvent) {
  if (!shouldSubmitComposer(event)) return;
  event.preventDefault();
  if (props.disabled) return;
  emit("submit");
}
</script>

<template>
  <textarea
    ref="field"
    v-model="model"
    class="composer"
    :data-variant="props.variant"
    rows="1"
    :placeholder="props.placeholder"
    :disabled="props.disabled"
    :autofocus="props.autofocus"
    autocomplete="off"
    enterkeyhint="send"
    @keydown="onKeydown"
  />
</template>

<style scoped>
.composer {
  display: block;
  margin: 0;
  min-width: 0;
  resize: none;
  overflow-x: hidden;
  overflow-y: hidden;
  color: var(--text-primary);
  font-family: var(--font-family);
  line-height: 1.4;
  scrollbar-width: thin;
  scrollbar-color: color-mix(in srgb, var(--text-secondary) 45%, transparent) transparent;
}

.composer:disabled {
  opacity: 0.55;
}

.composer[data-variant="pane"],
.composer[data-variant="setup"] {
  background: var(--surface);
  border: 1px solid rgba(255, 255, 255, 0.06);
  border-radius: 12px;
  padding: 0.9rem 1.1rem;
  font-size: 0.98rem;
  max-height: calc(1.8rem + 1.4em * 6);
}

.composer[data-variant="pane"] {
  flex: 1 1 0;
}

.composer[data-variant="setup"] {
  width: 100%;
}

.composer[data-variant="pane"]:focus-visible,
.composer[data-variant="setup"]:focus-visible {
  outline: 2px solid var(--accent);
  outline-offset: 1px;
}

.composer[data-variant="pill"] {
  flex: 1 1 0;
  background: none;
  border: none;
  padding: 0.5rem 0.2rem;
  font-size: 0.95rem;
  max-height: calc(1rem + 1.4em);
}

.composer[data-variant="pill"]:focus {
  outline: none;
}
</style>
