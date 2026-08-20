<script setup lang="ts">
// The whole-screen way to the same picker the conversation offers. Two
// doors to one room: the chip beside the composer is for someone who
// already knows what they want to change, this is for someone looking
// for it.
definePageMeta({ layout: false });
</script>

<template>
  <main>
    <header>
      <NuxtLink to="/settings" class="back" aria-label="Go back">‹</NuxtLink>
      <h1>What I think with</h1>
    </header>

    <!-- The picker positions itself against its container, so it gets one
         of its own rather than covering the heading above. -->
    <div class="frame">
      <ModelPicker @close="navigateTo('/settings')" />
    </div>
  </main>
</template>

<style scoped>
main {
  display: flex;
  flex-direction: column;
  height: 100vh;
  overflow: hidden;
}

header {
  flex: 0 0 auto;
  display: flex;
  align-items: center;
  gap: 0.6rem;
  padding: 1.5rem 1.75rem 0.5rem;
}

h1 {
  margin: 0;
  color: var(--text-primary);
  font-size: 1.1rem;
  font-weight: 400;
}

.back {
  color: var(--text-secondary);
  font-size: 1.5rem;
  line-height: 1;
  text-decoration: none;
}

.back:hover,
.back:focus-visible {
  color: var(--text-primary);
}

.frame {
  position: relative;
  flex: 1 1 0;
  min-height: 0;
}
</style>
