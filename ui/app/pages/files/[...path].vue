<script setup lang="ts">
// Any directory below home. A catch-all rather than a single segment,
// because folders nest to whatever depth the owner's disk actually has.
definePageMeta({ layout: "shell" });

const route = useRoute();

const path = computed(() => {
  const raw = route.params.path;
  const segments = Array.isArray(raw) ? raw : [raw ?? ""];
  return segments.filter(Boolean).join("/");
});
</script>

<template>
  <FileBrowser :path="path" />
</template>
