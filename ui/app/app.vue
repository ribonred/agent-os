<script setup lang="ts">
// Mandatory setup gate per onboarding.md -- skip the check while
// already on a setup route, or choosing language would immediately
// redirect back to itself before persona is chosen.
//
// Fails closed, not open: if the store read throws for any reason (a
// corrupted store file, a plugin error), treat setup as incomplete and
// redirect rather than silently letting the user through to the main
// screen unconfigured. A gate that fails open on error isn't a gate.
import { isSetupComplete } from "~/lib/setupStore";

const route = useRoute();

onMounted(async () => {
  if (route.path.startsWith("/setup")) return;
  let complete: boolean;
  try {
    complete = await isSetupComplete();
  } catch {
    complete = false;
  }
  if (!complete) {
    await navigateTo("/setup/language");
  }
});

useHead({ title: "agentic-os" });
</script>

<template>
  <NuxtPage />
</template>
