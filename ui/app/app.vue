<script setup lang="ts">
// Mandatory setup gate per onboarding.md -- skip the check while
// already on a setup route, or choosing language would immediately
// redirect back to itself before persona is chosen.
//
// Fails closed, not open: if the store read throws for any reason (a
// corrupted store file, a plugin error), treat setup as incomplete and
// redirect rather than silently letting the user through to the main
// screen unconfigured. A gate that fails open on error isn't a gate.
import { firstIncompleteSetupStep } from "~/lib/setupStore";

const route = useRoute();

onMounted(async () => {
  // Pin the webview zoom explicitly. On the device's bare kiosk
  // compositor no settings daemon exists, GTK's screen resolution stays
  // at its -1 "unknown" sentinel, and WebKitGTK turns that into an
  // automatic zoom of -1/96 -- a negative, near-zero devicePixelRatio
  // that destroys the entire layout (fonts collapse, the viewport goes
  // negative). An appliance owns its zoom baseline; diagnosed live on a
  // VM install by reading the broken devicePixelRatio off the device.
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
  <!-- NuxtLayout is what lets a page opt into the two-pane shell. Without
       it a page's `layout` is ignored entirely and every route renders
       bare, so setup screens (layout: false) look correct while the shell
       silently never appears. -->
  <NuxtLayout>
    <NuxtPage />
  </NuxtLayout>
</template>
