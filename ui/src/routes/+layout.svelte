<script lang="ts">
  import "../app.css";
  import { onMount } from "svelte";
  import { goto } from "$app/navigation";
  import { page } from "$app/state";
  import { isSetupComplete } from "$lib/setupStore";
  import type { Snippet } from "svelte";

  let { children }: { children: Snippet } = $props();

  // Mandatory setup gate per onboarding.md -- skip the check while
  // already on a setup route, or choosing language would immediately
  // redirect back to itself before persona is chosen.
  //
  // Fails closed, not open: if the store read throws for any reason (a
  // corrupted store file, a plugin error), treat setup as incomplete and
  // redirect rather than silently letting the user through to the main
  // screen unconfigured. A gate that fails open on error isn't a gate.
  onMount(async () => {
    if (page.url.pathname.startsWith("/setup")) return;
    let complete: boolean;
    try {
      complete = await isSetupComplete();
    } catch {
      complete = false;
    }
    if (!complete) {
      await goto("/setup/language");
    }
  });
</script>

{@render children()}
