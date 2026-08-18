<script setup lang="ts">
// The device asking permission before it runs something risky.
//
// In the conversation flow, never a modal and never a toast: the
// exchange is happening here, and a message that vanishes is a message
// the owner didn't read. Plain language first -- constitution.md forbids
// surfacing internals unasked, and a shell command presented as the
// question is a decision the owner has no way to make.

import { approvalQuestion } from "~/lib/approvalWords";

const { description, command, choices, answer } = defineProps<{
  description: string;
  command: string;
  choices: string[];
  answer: string | null;
}>();

const emit = defineEmits<{ answer: [choice: string] }>();

// The runtime's vocabulary, in words the owner can act on. Only the
// choices it actually offered are shown -- it sends fewer when a
// command may not be permanently allowed, and one it did not offer
// would be refused anyway.
const LABELS: Record<string, string> = {
  once: "Just this once",
  session: "Yes, for now",
  always: "Always allow this",
  deny: "No",
};

// What the owner sees afterwards, so the record reads as their decision
// rather than as an echo of the button.
const CHOSEN: Record<string, string> = {
  once: "You allowed this once.",
  session: "You allowed this for now.",
  always: "You allowed this from now on.",
  deny: "You said no.",
};

const label = (choice: string) => LABELS[choice] ?? choice;
// The runtime names the rule that caught the command ("delete in root
// path"), which is a question the owner has no way to answer. This is
// what they are actually being asked.
const question = computed(() => approvalQuestion(description));
const offered = computed(() => choices.filter((choice) => choice in LABELS));
const showCommand = ref(false);
</script>

<template>
  <section class="card" :class="{ answered: answer !== null }" role="group">
    <p class="ask">{{ question }}</p>

    <template v-if="answer === null">
      <button
        v-if="command"
        type="button"
        class="disclose"
        :aria-expanded="showCommand"
        @click="showCommand = !showCommand"
      >
        {{ showCommand ? "Hide the details" : "Show me the details" }}
      </button>
      <pre v-if="showCommand && command" class="command">{{ command }}</pre>

      <div class="choices">
        <button
          v-for="choice in offered"
          :key="choice"
          type="button"
          :class="['choice', { no: choice === 'deny' }]"
          @click="emit('answer', choice)"
        >
          {{ label(choice) }}
        </button>
      </div>
    </template>

    <p v-else class="answered-line">{{ CHOSEN[answer] ?? label(answer) }}</p>
  </section>
</template>

<style scoped>
.card {
  background: var(--surface-raised);
  border: 1px solid color-mix(in srgb, var(--accent) 22%, transparent);
  border-radius: 12px;
  padding: 0.9rem 1rem;
  display: flex;
  flex-direction: column;
  gap: 0.6rem;
}

/* Once answered it is a record, not a question: it stops asking for
   attention but never disappears, so the owner can see what they agreed
   to. */
.card.answered {
  background: none;
  border-color: color-mix(in srgb, var(--text-secondary) 20%, transparent);
  padding: 0.5rem 0;
}

.ask {
  margin: 0;
  color: var(--text-primary);
  font-size: 0.95rem;
  line-height: 1.5;
}

.card.answered .ask {
  color: var(--text-secondary);
  font-size: 0.88rem;
}

.disclose {
  align-self: flex-start;
  background: none;
  border: none;
  padding: 0;
  color: var(--text-secondary);
  font-family: var(--font-family);
  font-size: 0.85rem;
  text-decoration: underline;
  text-underline-offset: 3px;
  cursor: pointer;
}

.disclose:hover {
  color: var(--text-primary);
}

.command {
  margin: 0;
  padding: 0.6rem 0.7rem;
  background: var(--surface);
  border-radius: 8px;
  color: var(--text-secondary);
  font-family: ui-monospace, "Cascadia Code", "Source Code Pro", monospace;
  font-size: 0.82rem;
  line-height: 1.45;
  /* Wrapped rather than scrolled, unlike every other wide block on this
     surface: the owner is being asked to judge this exact text, and half
     a command scrolled off the edge is half a decision. */
  white-space: pre-wrap;
  overflow-wrap: anywhere;
}

.choices {
  display: flex;
  flex-wrap: wrap;
  gap: 0.5rem;
}

.choice {
  background: var(--surface);
  color: var(--text-primary);
  border: 1px solid color-mix(in srgb, var(--accent) 35%, transparent);
  border-radius: 999px;
  padding: 0.45rem 0.9rem;
  font-family: var(--font-family);
  font-size: 0.88rem;
  cursor: pointer;
  transition: background 0.15s ease, border-color 0.15s ease;
}

.choice:hover {
  background: color-mix(in srgb, var(--accent) 12%, var(--surface));
}

/* Saying no is never the styled-down option: it is a real answer, and
   the owner must not have to hunt for it. */
.choice.no {
  border-color: color-mix(in srgb, var(--danger) 35%, transparent);
}

.choice.no:hover {
  background: color-mix(in srgb, var(--danger) 12%, var(--surface));
}

.choice:focus-visible,
.disclose:focus-visible {
  outline: 2px solid var(--accent);
  outline-offset: 2px;
}

.answered-line {
  margin: 0;
  color: var(--text-secondary);
  font-size: 0.85rem;
}

@media (prefers-reduced-motion: reduce) {
  .choice {
    transition: none;
  }
}
</style>
