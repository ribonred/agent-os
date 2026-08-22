<script setup lang="ts">
// The product's signature element -- layer structure specified in
// design/DESIGN.md ("The presence orb"). Size scales the whole
// composition; `grounded` adds the light pool for the idle/home screen
// where the orb sits like a physical object. `orbState` changes rhythm,
// not shape (Motion spec): the orb's tempo IS the status indicator.
type OrbState = "idle" | "listening" | "thinking" | "speaking";

const props = withDefaults(
  defineProps<{ size?: number; grounded?: boolean; orbState?: OrbState }>(),
  { size: 160, grounded: false, orbState: "idle" },
);

// Rhythm and brightness only -- the layer structure never changes per
// state (design/DESIGN.md). `listening` is the one state the owner is
// causing rather than watching, so it is both the fastest breath and the
// brightest atmosphere: it has to be readable out of the corner of an
// eye as "it is hearing me", from across a counter.
const rhythm: Record<
  OrbState,
  { breathe: string; revolve: string; inner: string; mid: string }
> = {
  idle: { breathe: "6s", revolve: "20s", inner: "22%", mid: "10%" },
  listening: { breathe: "1.4s", revolve: "9s", inner: "34%", mid: "16%" },
  thinking: { breathe: "1.8s", revolve: "6s", inner: "22%", mid: "10%" },
  speaking: { breathe: "3.2s", revolve: "11s", inner: "28%", mid: "13%" },
};

const style = computed(() => ({
  "--orb-size": `${props.size}px`,
  "--orb-breathe": rhythm[props.orbState].breathe,
  "--orb-revolve": rhythm[props.orbState].revolve,
  "--orb-atmosphere-inner": rhythm[props.orbState].inner,
  "--orb-atmosphere-mid": rhythm[props.orbState].mid,
}));
</script>

<template>
  <div class="orb" :style="style" aria-hidden="true">
    <div class="atmosphere"></div>
    <div class="core"></div>
    <div class="shading"></div>
    <div class="ring"></div>
    <div v-if="grounded" class="pool"></div>
  </div>
</template>

<style scoped>
.orb {
  position: relative;
  width: var(--orb-size);
  height: var(--orb-size);
}

.orb > * {
  position: absolute;
  border-radius: 50%;
}

/* Layer 1: atmosphere -- the breath lives here */
.atmosphere {
  inset: -55%;
  background: radial-gradient(
    circle,
    color-mix(in srgb, var(--orb-cyan) var(--orb-atmosphere-inner, 22%), transparent) 0%,
    color-mix(in srgb, var(--orb-violet) var(--orb-atmosphere-mid, 10%), transparent) 45%,
    transparent 70%
  );
  /* The colour moves with the state as well as the tempo, so a change
     of rhythm is legible in a still frame and not only in motion --
     which is what someone glancing over from the other side of a
     counter actually gets. */
  transition: background 0.4s ease;
  animation: breathe var(--orb-breathe, 6s) ease-in-out infinite;
}

/* Layer 2: core -- the iridescent sphere, rotating on transform
   (not gradient angle) for webview compatibility */
.core {
  inset: 0;
  background: conic-gradient(
    from 210deg,
    var(--orb-cyan),
    var(--orb-violet) 30%,
    var(--orb-deep) 55%,
    var(--orb-violet) 74%,
    var(--orb-warm) 85%,
    var(--orb-cyan)
  );
  /* Heavy blur relative to size on purpose: it melts the conic
     segments into continuous iridescence -- with less blur the core
     reads as pie slices, not living light */
  filter: blur(calc(var(--orb-size) / 11)) saturate(1.3);
  animation: revolve var(--orb-revolve, 20s) linear infinite;
}

/* Layer 3: shading -- specular highlight + darkened lower edge is
   what makes it read as a sphere instead of a colored circle.
   Normal blending, not soft-light: blend modes wash out to nothing
   against a near-black backdrop */
.shading {
  inset: 0;
  overflow: hidden;
  background:
    radial-gradient(
      circle at 33% 27%,
      rgba(255, 255, 255, 0.38) 0%,
      rgba(255, 255, 255, 0.08) 24%,
      transparent 48%
    ),
    radial-gradient(
      ellipse 90% 55% at 50% 112%,
      rgba(4, 6, 12, 0.6) 20%,
      transparent 65%
    );
}

/* Layer 4: ring -- the machined, instrument-like counterpoint */
.ring {
  inset: -7%;
  border: 1px solid color-mix(in srgb, var(--orb-cyan) 32%, transparent);
  box-shadow: 0 0 18px color-mix(in srgb, var(--orb-cyan) 12%, transparent);
}

/* Layer 5: light pool -- the orb sitting on a real surface */
.pool {
  left: 50%;
  top: 128%;
  width: 160%;
  height: 14%;
  transform: translateX(-50%);
  border-radius: 50%;
  background: radial-gradient(
    ellipse,
    color-mix(in srgb, var(--orb-cyan) 16%, transparent) 0%,
    transparent 70%
  );
  filter: blur(6px);
  animation: pool-breathe var(--orb-breathe, 6s) ease-in-out infinite;
}

@keyframes breathe {
  0%,
  100% {
    opacity: 0.55;
    transform: scale(0.96);
  }
  50% {
    opacity: 1;
    transform: scale(1.03);
  }
}

@keyframes pool-breathe {
  0%,
  100% {
    opacity: 0.4;
    transform: translateX(-50%) scale(0.94);
  }
  50% {
    opacity: 0.85;
    transform: translateX(-50%) scale(1.04);
  }
}

@keyframes revolve {
  from {
    transform: rotate(0deg);
  }
  to {
    transform: rotate(360deg);
  }
}

@media (prefers-reduced-motion: reduce) {
  .atmosphere,
  .core,
  .pool {
    animation: none;
  }

  /* The brightness still changes -- it is what the state IS, and losing
     it would leave the orb saying nothing at all here. It just changes
     at once instead of easing. */
  .atmosphere {
    transition: none;
  }
}
</style>
