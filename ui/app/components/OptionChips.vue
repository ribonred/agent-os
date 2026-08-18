<script setup lang="ts">
// The answers the device offered, as things to tap.
//
// A shortcut, never a gate: the owner can ignore every one of them and
// type something else, and that is a normal reply rather than a mistake
// to correct. Tapping sends the answer as their message, so it becomes
// an ordinary turn in the conversation and not a form field.

const { options, disabled = false } = defineProps<{
  options: string[];
  disabled?: boolean;
}>();

const emit = defineEmits<{ pick: [option: string] }>();
</script>

<template>
  <div v-if="options.length > 0" class="chips">
    <button
      v-for="(option, index) in options"
      :key="option"
      type="button"
      :class="['chip', { recommended: index === 0 }]"
      :disabled="disabled"
      @click="emit('pick', option)"
    >
      {{ option }}
    </button>
  </div>
</template>

<style scoped>
.chips {
  display: flex;
  flex-wrap: wrap;
  gap: 0.45rem;
}

.chip {
  background: var(--surface);
  color: var(--text-primary);
  border: 1px solid color-mix(in srgb, var(--text-secondary) 25%, transparent);
  border-radius: 999px;
  padding: 0.4rem 0.85rem;
  font-family: var(--font-family);
  font-size: 0.88rem;
  cursor: pointer;
  transition: background 0.15s ease, border-color 0.15s ease;
}

/* The one the device would choose itself, marked so the owner can see
   at a glance what it recommends -- a quiet edge, not a filled button:
   this is a suggestion, and the others are equally real answers. */
.chip.recommended {
  border-color: color-mix(in srgb, var(--accent) 45%, transparent);
}

.chip:hover:not(:disabled) {
  background: color-mix(in srgb, var(--accent) 10%, var(--surface));
}

.chip:disabled {
  opacity: 0.5;
  cursor: default;
}

.chip:focus-visible {
  outline: 2px solid var(--accent);
  outline-offset: 2px;
}

@media (prefers-reduced-motion: reduce) {
  .chip {
    transition: none;
  }
}
</style>
