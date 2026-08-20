<script setup lang="ts">
// Choosing what the device thinks with.
//
// Covers the conversation pane full-height rather than opening beside it,
// the same as the list of earlier conversations: nothing new is added
// around the pane for a list, and covering means it reads the same at
// every window size. The one difference is that this is also a whole
// settings screen, so it renders inside whatever it is given.

import { familyColor } from "~/lib/modelGroups";

const { current, groups, query, loading, error, load, choose } = useModels();
const emit = defineEmits<{ close: [] }>();

const saving = ref<string | null>(null);

onMounted(() => load());

async function pick(id: string) {
  if (id === current.value?.id || saving.value) return;
  saving.value = id;
  const ok = await choose(id);
  saving.value = null;
  if (ok) emit("close");
}

const nothingFound = computed(
  () => !loading.value && !error.value && groups.value.length === 0,
);
</script>

<template>
  <div class="picker">
    <header>
      <h2>What I think with</h2>
      <button type="button" class="close" aria-label="Back" @click="emit('close')">
        Close
      </button>
    </header>

    <input
      v-model="query"
      class="find"
      type="search"
      placeholder="Search"
      aria-label="Search models"
    />

    <div class="scroller">
      <p v-if="error" class="error" role="alert">
        {{ error }}
        <button type="button" @click="load(true)">Try again</button>
      </p>

      <p v-else-if="loading && !groups.length" class="quiet">One moment…</p>

      <p v-else-if="nothingFound" class="quiet">
        Nothing matches “{{ query }}”.
      </p>

      <template v-else>
        <section v-for="provider in groups" :key="provider.slug">
          <h3 class="provider">
            {{ provider.name }}
            <!-- Shown rather than hidden: "this needs a key" is a better
                 answer than a list that is quietly shorter. -->
            <span v-if="!provider.authenticated" class="needs-key">needs a key</span>
          </h3>

          <div
            v-for="family in provider.families"
            :key="family.family"
            class="family"
            :style="{ '--family': familyColor(family.family) }"
          >
            <!-- A maker whose name is the provider's needs no heading of
                 its own; the colour still marks the group. -->
            <h4 v-if="family.showName">
              <!-- Decoration beside a name already written in words: the
                   colour makes scanning fifty rows quicker and carries
                   nothing on its own. -->
              <span class="mark" aria-hidden="true" />
              {{ family.familyName }}
            </h4>
            <button
              v-for="model in family.models"
              :key="model.id"
              type="button"
              :class="['row', { on: model.id === current?.id }]"
              :disabled="saving !== null"
              :aria-current="model.id === current?.id"
              @click="pick(model.id)"
            >
              <span class="name">{{ model.name }}</span>
              <span v-if="model.fast" class="tag">fast</span>
              <span v-if="saving === model.id" class="tag">switching…</span>
              <span v-else-if="model.id === current?.id" class="tag on">in use</span>
            </button>
          </div>
        </section>
      </template>
    </div>
  </div>
</template>

<style scoped>
.picker {
  position: absolute;
  inset: 0;
  z-index: 5;
  display: flex;
  flex-direction: column;
  background: var(--bg);
}

header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 0.5rem;
  padding: 1.1rem 1rem 0.6rem;
}

h2 {
  margin: 0;
  color: var(--text-primary);
  font-size: 0.95rem;
  font-weight: 500;
}

.close {
  padding: 0.2rem 0.45rem;
  background: none;
  border: 0;
  border-radius: 6px;
  color: var(--text-secondary);
  font-size: 0.9rem;
  line-height: 1;
  cursor: pointer;
}

.close:hover {
  color: var(--text-primary);
}

.find {
  margin: 0 1rem 0.5rem;
  padding: 0.45rem 0.6rem;
  background: var(--surface);
  border: 1px solid transparent;
  border-radius: 9px;
  color: var(--text-primary);
  font: inherit;
  font-size: 0.85rem;
}

.find:focus {
  outline: none;
  border-color: var(--accent);
}

.scroller {
  flex: 1;
  overflow-y: auto;
  padding: 0 0.5rem 1.5rem;
}

/* The provider is the account the device bills; the maker is what the
   owner actually recognises. So the provider is the quieter heading. */
.provider {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  margin: 1rem 0 0.2rem;
  padding: 0 0.5rem;
  color: var(--text-secondary);
  font-size: 0.72rem;
  font-weight: 600;
  letter-spacing: 0.08em;
  text-transform: uppercase;
}

.needs-key {
  color: var(--accent-warm);
  font-weight: 400;
  letter-spacing: 0.02em;
  text-transform: none;
}

/* The rule ties a maker's rows together down the side; it is faint
   enough to be a grouping and not a highlight. */
.family {
  margin: 0.7rem 0 0.1rem;
  border-left: 2px solid color-mix(in srgb, var(--family) 30%, transparent);
  padding-left: 0.55rem;
}

.family h4 {
  display: flex;
  align-items: center;
  gap: 0.45rem;
  margin: 0 0 0.1rem;
  padding: 0 0.5rem;
  color: var(--text-primary);
  font-size: 0.85rem;
  font-weight: 600;
}

/* The colour itself, at full strength, on a mark and nothing else. On
   text it would compete with --accent, which on this device means the
   assistant. */
.mark {
  flex: 0 0 auto;
  width: 7px;
  height: 7px;
  border-radius: 50%;
  background: var(--family);
}

.row {
  display: flex;
  align-items: baseline;
  gap: 0.5rem;
  width: 100%;
  text-align: left;
  background: none;
  border: 0;
  border-radius: 8px;
  padding: 0.42rem 0.5rem;
  color: var(--text-primary);
  font-family: var(--font-family);
  font-size: 0.9rem;
  cursor: pointer;
}

.row:hover:not(:disabled) {
  background: var(--surface);
}

.row:focus-visible {
  outline: 2px solid var(--accent);
  outline-offset: -2px;
}

.row:disabled {
  cursor: default;
}

.row.on {
  background: color-mix(in srgb, var(--accent) 12%, transparent);
}

.name {
  flex: 1;
  min-width: 0;
}

.tag {
  flex: 0 0 auto;
  color: var(--text-secondary);
  font-size: 0.74rem;
}

.tag.on {
  color: var(--accent);
}

.quiet {
  margin: 1.5rem 1rem;
  color: var(--text-secondary);
  font-size: 0.9rem;
}

.error {
  margin: 1.5rem 1rem;
  color: var(--danger);
  font-size: 0.9rem;
}

.error button {
  display: block;
  margin-top: 0.6rem;
  background: var(--surface);
  color: var(--text-primary);
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 10px;
  padding: 0.4rem 0.9rem;
  font-family: var(--font-family);
  font-size: 0.85rem;
  cursor: pointer;
}
</style>
