<script setup lang="ts">
// One assistant reply, rendered. Markdown adds structure, not chrome:
// there is still no bubble and no avatar -- this is text on the canvas,
// with lists that look like lists.

import { codeBlockText, renderMarkdown } from "~/lib/markdown";

const { text } = defineProps<{ text: string }>();

const body = ref<HTMLElement | null>(null);
const html = computed(() => renderMarkdown(text));
/// Which block was just copied, so the control can say so instead of
/// leaving the owner wondering whether anything happened.
const copied = ref<number | null>(null);

// The copy control is added to the rendered markup rather than being a
// component, because the markup is a sanitized string by the time it
// exists. One listener on the container, so a reply that grows a
// hundred code blocks does not grow a hundred listeners.
function decorate() {
  const container = body.value;
  if (!container) return;
  container.querySelectorAll("pre").forEach((block, index) => {
    if (block.querySelector(".copy")) return;
    const button = document.createElement("button");
    button.type = "button";
    button.className = "copy";
    button.dataset.index = String(index);
    button.textContent = "Copy";
    block.append(button);
  });
}

watch(() => html.value, () => nextTick(decorate), { immediate: true });
onMounted(decorate);

async function onClick(event: MouseEvent) {
  const target = (event.target as HTMLElement | null)?.closest?.(".copy");
  if (!(target instanceof HTMLElement)) return;
  const block = target.closest("pre");
  if (!block) return;
  try {
    await navigator.clipboard.writeText(codeBlockText(block));
    copied.value = Number(target.dataset.index);
    target.textContent = "Copied";
    setTimeout(() => {
      target.textContent = "Copy";
      copied.value = null;
    }, 1600);
  } catch {
    // Nothing to say: a clipboard that refuses is not the owner's
    // problem to solve, and the text is still on screen to select.
  }
}
</script>

<template>
  <!-- eslint-disable vue/no-v-html -- sanitized in lib/markdown.ts to a
       fixed tag list with no attributes at all -->
  <div ref="body" class="body" @click="onClick" v-html="html" />
</template>

<style scoped>
.body {
  color: var(--text-primary);
  font-size: 1rem;
  line-height: 1.65;
}

.body :deep(p) {
  margin: 0 0 0.85em;
}

.body :deep(p:last-child) {
  margin-bottom: 0;
}

/* A reply is speech, not a document: a heading orients, it does not
   announce. Large type in a narrow pane reads as shouting. */
.body :deep(h1),
.body :deep(h2),
.body :deep(h3),
.body :deep(h4),
.body :deep(h5),
.body :deep(h6) {
  margin: 1.2em 0 0.5em;
  font-size: 1rem;
  font-weight: 600;
  letter-spacing: 0.01em;
  color: var(--text-primary);
}

.body :deep(ul),
.body :deep(ol) {
  margin: 0 0 0.85em;
  padding-left: 1.35em;
}

.body :deep(li) {
  margin: 0.2em 0;
}

.body :deep(strong) {
  font-weight: 600;
}

.body :deep(blockquote) {
  margin: 0 0 0.85em;
  padding-left: 0.9em;
  border-left: 2px solid color-mix(in srgb, var(--text-secondary) 40%, transparent);
  color: var(--text-secondary);
}

.body :deep(code) {
  font-family: ui-monospace, "Cascadia Code", "Source Code Pro", monospace;
  font-size: 0.88em;
  background: var(--surface-raised);
  border-radius: 5px;
  padding: 0.12em 0.35em;
}

/* Anything wide scrolls inside its own container: the pane has a
   measure that suits bare prose and nothing may widen it. */
.body :deep(pre) {
  position: relative;
  margin: 0 0 0.85em;
  padding: 0.8rem 0.9rem;
  background: var(--surface-raised);
  border-radius: 10px;
  overflow-x: auto;
}

.body :deep(pre code) {
  background: none;
  padding: 0;
  white-space: pre;
}

.body :deep(table) {
  display: block;
  overflow-x: auto;
  border-collapse: collapse;
  margin: 0 0 0.85em;
  font-size: 0.92em;
}

.body :deep(th),
.body :deep(td) {
  border: 1px solid color-mix(in srgb, var(--text-secondary) 25%, transparent);
  padding: 0.35em 0.6em;
  text-align: left;
}

.body :deep(hr) {
  border: none;
  border-top: 1px solid color-mix(in srgb, var(--text-secondary) 25%, transparent);
  margin: 1.2em 0;
}

.body :deep(.copy) {
  position: absolute;
  top: 0.45rem;
  right: 0.45rem;
  background: var(--surface);
  color: var(--text-secondary);
  border: 1px solid color-mix(in srgb, var(--text-secondary) 25%, transparent);
  border-radius: 7px;
  padding: 0.2rem 0.5rem;
  font-family: var(--font-family);
  font-size: 0.75rem;
  cursor: pointer;
  opacity: 0;
  transition: opacity 0.15s ease;
}

.body :deep(pre:hover .copy),
.body :deep(.copy:focus-visible) {
  opacity: 1;
}

@media (prefers-reduced-motion: reduce) {
  .body :deep(.copy) {
    transition: none;
  }
}
</style>
