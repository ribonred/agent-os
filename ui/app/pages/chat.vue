<script setup lang="ts">
// The conversation is a pane in the shell now, not a page of its own.
// This redirect exists so an older link -- the home screen's "wake"
// affordance, a bookmark on a dev machine -- lands somewhere sensible
// rather than on a blank route. Safe to delete after one release.
definePageMeta({ layout: false });

await navigateTo("/", { replace: true });
</script>

<template>
  <div />
</template>
