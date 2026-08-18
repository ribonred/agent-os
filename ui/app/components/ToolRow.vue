<script setup lang="ts">
// One thing the device did, said in one line.
//
// Not a progress indicator, and it must never become one: the orb
// already says the device is working. This says *what* it did, which is
// the question the owner actually has when a reply takes a while, and a
// device that goes quiet and then produces an answer is asking to be
// taken on faith.

const { summary, phase } = defineProps<{
  summary: string;
  phase: "started" | "completed" | "failed";
}>();
</script>

<template>
  <p :class="['tool', phase]">
    <span class="mark" aria-hidden="true" />
    {{ summary }}<span v-if="phase === 'failed'">, but it didn't work</span>
  </p>
</template>

<style scoped>
.tool {
  display: flex;
  align-items: center;
  gap: 0.55rem;
  margin: 0;
  color: var(--text-secondary);
  font-size: 0.85rem;
  line-height: 1.4;
}

.mark {
  width: 5px;
  height: 5px;
  border-radius: 50%;
  flex: 0 0 auto;
  background: color-mix(in srgb, var(--text-secondary) 60%, transparent);
}

/* Still doing it: the accent marks where the device is, exactly as it
   does everywhere else. */
.started .mark {
  background: var(--accent);
  animation: pulse 1.8s ease-in-out infinite;
}

.failed {
  color: var(--danger);
}

.failed .mark {
  background: var(--danger);
  animation: none;
}

@keyframes pulse {
  0%,
  100% {
    opacity: 0.35;
  }
  50% {
    opacity: 1;
  }
}

@media (prefers-reduced-motion: reduce) {
  .started .mark {
    animation: none;
    opacity: 1;
  }
}
</style>
