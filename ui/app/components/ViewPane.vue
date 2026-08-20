<script setup lang="ts">
// A view: a page the device built, shown to the owner.
//
// The page is model-authored markup, so it is never put into this
// document. It loads over the `view://` scheme into a frame sandboxed
// with no `allow-scripts` and no `allow-same-origin` -- an opaque
// origin with no scripting, no storage, and no path back to the Tauri
// command bridge that lives in this window. See design/DESIGN.md.
//
// Two surfaces, the same shape as the file view: the list of what the
// device has made, and one of them open.

import { getViewTheme, setViewTheme, type ViewTheme } from "~/lib/shelfStore";

const { views, current, open, error, load, show } = useViews();

// Light by default, because a view is a document and it gets printed.
// The choice rides in the URL rather than being read from the store by
// the native side: the page is served as a pure function of what was
// asked for, and changing it changes the address, which is what makes
// the frame actually reload.
const theme = ref<ViewTheme>("light");
onMounted(async () => (theme.value = await getViewTheme()));

async function toggleTheme() {
  theme.value = theme.value === "dark" ? "light" : "dark";
  await setViewTheme(theme.value);
}

onMounted(load);

// The agent may have built or rewritten a view during its turn -- re-read
// when it finishes rather than leaving the owner looking at a stale list.
const { busy } = useConversation();
watch(busy, (isBusy, was) => {
  if (was && !isBusy) load();
});

const src = computed(() =>
  current.value
    ? `view://localhost/${encodeURI(current.value.name)}/index.html?theme=${theme.value}`
    : "",
);

/// Where the figures came from, said the way the owner names their own
/// files. Never a path, and never absent when the view declared one.
const source = computed(() => {
  const from = current.value?.from ?? [];
  if (from.length === 0) return null;
  const names = from.map((path) => path.split("/").pop() ?? path);
  if (names.length === 1) return `From ${names[0]}.`;
  const last = names[names.length - 1];
  return `From ${names.slice(0, -1).join(", ")} and ${last}.`;
});

// Printing goes through this window rather than the frame: with no
// same-origin there is nothing to call print() on inside it. The print
// stylesheet in the shell hides everything except the frame, so what
// comes out of the printer is the view and nothing around it.
function print() {
  window.print();
}

function relativeDay(ms: number) {
  if (!ms) return "";
  const days = Math.floor((Date.now() - ms) / 86_400_000);
  if (days <= 0) return "today";
  if (days === 1) return "yesterday";
  if (days < 7) return `${days} days ago`;
  return new Date(ms).toLocaleDateString(undefined, {
    day: "numeric",
    month: "long",
  });
}
</script>

<template>
  <section class="views">
    <header class="bar">
      <button
        v-if="current"
        type="button"
        class="back"
        aria-label="Back to Your Views"
        @click="open = null"
      >
        ‹
      </button>

      <h1 class="title">{{ current ? current.title : "Your Views" }}</h1>

      <button
        v-if="current"
        type="button"
        class="action"
        :aria-pressed="theme === 'dark'"
        :title="theme === 'dark' ? 'Show this page light' : 'Show this page dark'"
        @click="toggleTheme"
      >
        {{ theme === "dark" ? "Light" : "Dark" }}
      </button>

      <button v-if="current" type="button" class="action" @click="print">
        Print
      </button>
    </header>

    <p v-if="error" class="surface-error" role="alert">
      {{ error }}
      <button type="button" @click="load">Try again</button>
    </p>

    <div v-else-if="views.length === 0" class="empty enter-fade">
      <p class="lead">I haven't made anything yet.</p>
      <p class="hint">Ask me for a summary of something and I'll build it here.</p>
    </div>

    <!-- The page itself. Sandboxed with no allowances at all: it may
         render, and it may do nothing else. -->
    <div v-else-if="current" class="frame enter-fade">
      <iframe
        :src="src"
        :title="current.title"
        sandbox=""
        referrerpolicy="no-referrer"
      />
    </div>

    <div v-else class="rows enter-fade">
      <button
        v-for="view in views"
        :key="view.name"
        type="button"
        class="row"
        @click="show(view.name)"
      >
        <span class="name">{{ view.title }}</span>
        <span v-if="view.asked" class="asked">{{ view.asked }}</span>
        <span class="when">{{ relativeDay(view.modified) }}</span>
      </button>
    </div>

    <!-- Said under the page rather than over it: the owner reads the
         answer first, and checks where it came from second. -->
    <p v-if="current && source" class="source">{{ source }}</p>
  </section>
</template>

<style scoped>
.views {
  display: flex;
  flex-direction: column;
  /* Its container, not the viewport -- the tab strip is above it. */
  height: 100%;
}

.bar {
  display: flex;
  align-items: center;
  gap: 0.6rem;
  flex: 0 0 auto;
  height: 56px;
  padding: 0 1.75rem;
  border-bottom: 1px solid rgba(255, 255, 255, 0.05);
}

.title {
  flex: 1;
  min-width: 0;
  margin: 0;
  color: var(--text-primary);
  font-size: 0.95rem;
  font-weight: 400;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.back {
  background: none;
  border: none;
  color: var(--text-secondary);
  font-size: 1.5rem;
  line-height: 1;
  padding: 0;
  cursor: pointer;
}

.back:hover {
  color: var(--text-primary);
}

.action {
  flex: 0 0 auto;
  background: var(--surface);
  color: var(--text-secondary);
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 10px;
  padding: 0.35rem 0.8rem;
  font-family: var(--font-family);
  font-size: 0.82rem;
  cursor: pointer;
}

.action:hover {
  color: var(--text-primary);
  background: var(--surface-raised);
}

.frame {
  flex: 1 1 0;
  min-height: 0;
}

.frame iframe {
  display: block;
  width: 100%;
  height: 100%;
  border: none;
  background: var(--bg);
}

.rows {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow-y: auto;
}

/* One row per thing the device has made -- the same weight as a file
   row, because to the owner these sit alongside their own things. */
.row {
  display: flex;
  align-items: baseline;
  gap: 1rem;
  width: 100%;
  text-align: left;
  background: none;
  border: none;
  border-bottom: 1px solid rgba(255, 255, 255, 0.04);
  padding: 0.9rem 1.75rem;
  font-family: var(--font-family);
  cursor: pointer;
}

.row:hover {
  background: var(--surface);
}

.row:focus-visible {
  outline: 2px solid var(--accent);
  outline-offset: -2px;
}

.name {
  flex: 0 0 auto;
  color: var(--text-primary);
  font-size: 0.95rem;
}

.asked {
  flex: 1;
  min-width: 0;
  color: var(--text-secondary);
  font-size: 0.85rem;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.when {
  flex: 0 0 auto;
  color: var(--text-secondary);
  font-size: 0.82rem;
}

.source {
  flex: 0 0 auto;
  margin: 0;
  padding: 0.7rem 1.75rem;
  border-top: 1px solid rgba(255, 255, 255, 0.05);
  color: var(--text-secondary);
  font-size: 0.82rem;
}

.empty {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 0.5rem;
  padding: 2rem;
  text-align: center;
}

.lead {
  margin: 0;
  color: var(--text-primary);
  font-size: 1.05rem;
  font-weight: 300;
}

.hint {
  margin: 0;
  color: var(--text-secondary);
  font-size: 0.92rem;
}

.surface-error {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 0.9rem;
  margin: 0;
  color: var(--danger);
  font-size: 0.95rem;
}

.surface-error button {
  background: var(--surface);
  color: var(--text-primary);
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 10px;
  padding: 0.5rem 1rem;
  font-family: var(--font-family);
  font-size: 0.88rem;
  cursor: pointer;
}

.enter-fade {
  animation: fade-in 0.35s both;
}

@keyframes fade-in {
  from {
    opacity: 0;
  }
  to {
    opacity: 1;
  }
}

@media (prefers-reduced-motion: reduce) {
  .enter-fade {
    animation: none;
  }
}
</style>
