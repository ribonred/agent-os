<script setup lang="ts">
// Kind, drawn as a small line icon (design/DESIGN.md "Icons").
//
// Lucide, imported per-icon so only the handful below ship -- nothing is
// fetched at runtime, which matters on a device that may never see a
// network. One mark per kind the view genuinely distinguishes, no more:
// every extra symbol is one more thing to decode.
import {
  Folder,
  FileText,
  FileSpreadsheet,
  FileImage,
  FileType,
  FileAudio,
  FileVideo,
  FileArchive,
  File,
} from "lucide-vue-next";
import type { EntryKind } from "~/composables/useShelf";

const { kind = "file" } = defineProps<{ kind?: EntryKind }>();

const icon = computed(() => {
  switch (kind) {
    case "folder":
      return Folder;
    case "table":
      return FileSpreadsheet;
    case "image":
      return FileImage;
    case "document":
      return FileType;
    case "audio":
      return FileAudio;
    case "video":
      return FileVideo;
    case "archive":
      return FileArchive;
    case "text":
      return FileText;
    default:
      return File;
  }
});
</script>

<template>
  <!-- Wrapped rather than styling the icon component directly: the class
       would land on the child's root SVG, where scoped styles don't
       reach it and the size prop alone loses to flex stretching. -->
  <span :class="['motif', kind === 'folder' ? 'is-folder' : 'is-file']" aria-hidden="true">
    <component :is="icon" :size="20" :stroke-width="1.75" />
  </span>
</template>

<style scoped>
.motif {
  display: inline-flex;
  flex: 0 0 auto;
  color: var(--text-secondary);
}

/* Explicit pixels, not percentages: inside an inline-flex wrapper with
   no definite width, `width: 100%` resolves to zero and the glyph
   collapses to a hairline. */
.motif :deep(svg) {
  display: block;
  width: 20px;
  height: 20px;
}

/* A folder is the thing you navigate by, so it reads a step brighter
   than the files sitting beside it. */
.motif.is-folder {
  color: var(--text-primary);
  opacity: 0.8;
}
</style>
