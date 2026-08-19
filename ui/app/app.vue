<script setup lang="ts">
import { firstIncompleteSetupStep } from "~/lib/setupStore";

const route = useRoute();
const { load: loadWindowMode } = useWindowMode();

onMounted(async () => {
  await loadWindowMode();
  try {
    const { getCurrentWebview } = await import("@tauri-apps/api/webview");
    await getCurrentWebview().setZoom(1.0);
  } catch {
    // Non-Tauri contexts (browser-based visual checks) have no webview.
  }

  if (route.path.startsWith("/setup")) return;
  let missingStep: string | null;
  try {
    missingStep = await firstIncompleteSetupStep();
  } catch {
    missingStep = "/setup/language";
  }
  if (missingStep !== null) {
    await navigateTo(missingStep);
  }
});

useHead({ title: "agentic-os" });
</script>

<template>
  <NuxtLayout>
    <NuxtPage />
  </NuxtLayout>
</template>
