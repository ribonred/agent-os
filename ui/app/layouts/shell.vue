<script setup lang="ts">
// The two-pane shell: the conversation on the left, the owner's things on
// the right. This is a layout rather than a page for one load-bearing
// reason -- Nuxt keeps a layout instance alive across routes that share
// it, so the conversation pane does not remount when the main area
// changes and a reply keeps streaming while the owner looks around.
import { getChatCollapsed, setChatCollapsed } from "~/lib/shelfStore";

// Minimized, the whole shell is the pill: the file view has no room and
// no purpose at that size, and the conversation is the only thing the
// owner came back for. The page underneath keeps its state, so the way
// back is a resize rather than a reload.
const { mode } = useWindowMode();

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
  <PillShell v-if="mode === 'minimized'" />
  <div v-else class="shell">
    <!-- The window has no decoration, so it draws its own resize
         border. The pill has none: it sizes itself to what there is to
         read, and a hand-resized pill would fight that. -->
    <WindowEdges />
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
