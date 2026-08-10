<script setup lang="ts">
// The two-pane shell: the conversation on the left, the owner's things on
// the right. This is a layout rather than a page for one load-bearing
// reason -- Nuxt keeps a layout instance alive across routes that share
// it, so the conversation pane does not remount when the main area
// changes and a reply keeps streaming while the owner looks around.
import { getChatCollapsed, setChatCollapsed } from "~/lib/shelfStore";

const collapsed = ref(false);

onMounted(async () => {
  collapsed.value = await getChatCollapsed();
});

async function setCollapsed(value: boolean) {
  collapsed.value = value;
  await setChatCollapsed(value);
}
</script>

<template>
  <div class="shell">
    <ConversationPane
      :collapsed="collapsed"
      @update:collapsed="setCollapsed"
    />
    <main class="main">
      <slot />
    </main>
  </div>
</template>

<style scoped>
.shell {
  display: flex;
  height: 100vh;
  overflow: hidden;
}

.main {
  flex: 1;
  min-width: 0;
  height: 100vh;
  overflow-y: auto;
}
</style>
