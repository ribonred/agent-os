<script lang="ts">
  // The product's signature element -- layer structure specified in
  // design/DESIGN.md ("The presence orb"). Size scales the whole
  // composition; `grounded` adds the light pool for the idle/home screen
  // where the orb sits like a physical object.
  let { size = 160, grounded = false }: { size?: number; grounded?: boolean } =
    $props();
</script>

<div class="orb" style:--orb-size="{size}px" aria-hidden="true">
  <div class="atmosphere"></div>
  <div class="core"></div>
  <div class="shading"></div>
  <div class="ring"></div>
  {#if grounded}
    <div class="pool"></div>
  {/if}
</div>

<style>
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
      color-mix(in srgb, var(--orb-cyan) 22%, transparent) 0%,
      color-mix(in srgb, var(--orb-violet) 10%, transparent) 45%,
      transparent 70%
    );
    animation: breathe 6s ease-in-out infinite;
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
    animation: revolve 20s linear infinite;
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
    animation: breathe 6s ease-in-out infinite;
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

  /* .pool overrides breathe's transform with its own centering */
  .pool {
    animation-name: pool-breathe;
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
  }
</style>
