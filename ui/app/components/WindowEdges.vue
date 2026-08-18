<script setup lang="ts">
// The window's own resize border, drawn by the page.
//
// An undecorated window has no resize border, because that border is
// part of the decoration the design deliberately does without. Without
// something like this the owner cannot change the window's size at all,
// on a desktop where every other window can be dragged to fit. The
// strips are a few pixels wide, carry the cursor the edge would have
// had, and hand the drag straight to the window manager.

import type { ResizeDirection } from "~/composables/useWindowMode";

const { resizeDrag } = useWindowMode();

const EDGES: ResizeDirection[] = [
  "north",
  "south",
  "east",
  "west",
  "north-west",
  "north-east",
  "south-west",
  "south-east",
];
</script>

<template>
  <div class="edges" aria-hidden="true">
    <span
      v-for="edge in EDGES"
      :key="edge"
      :class="['edge', edge]"
      @mousedown.prevent="resizeDrag(edge)"
    />
  </div>
</template>

<style scoped>
/* Pointer-events only on the strips themselves: the container must not
   swallow a click meant for the conversation. */
.edges {
  position: fixed;
  inset: 0;
  pointer-events: none;
  z-index: 50;
}

.edge {
  position: absolute;
  pointer-events: auto;
}

/* Wide enough to hit with a mouse, narrow enough not to steal a click
   from the row or button underneath. Corners sit above the sides. */
.north,
.south {
  left: 6px;
  right: 6px;
  height: 4px;
  cursor: ns-resize;
}

.east,
.west {
  top: 6px;
  bottom: 6px;
  width: 4px;
  cursor: ew-resize;
}

.north { top: 0; }
.south { bottom: 0; }
.west { left: 0; }
.east { right: 0; }

.north-west,
.north-east,
.south-west,
.south-east {
  width: 10px;
  height: 10px;
}

.north-west { top: 0; left: 0; cursor: nwse-resize; }
.north-east { top: 0; right: 0; cursor: nesw-resize; }
.south-west { bottom: 0; left: 0; cursor: nesw-resize; }
.south-east { bottom: 0; right: 0; cursor: nwse-resize; }
</style>
