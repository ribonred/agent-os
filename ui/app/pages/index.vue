<script setup lang="ts">
// Idle/home screen -- the orb at rest on its surface. Touching the orb
// (or the invitation) opens the conversation; the whole center of the
// screen is the way in, no button chrome.
</script>

<template>
  <main>
    <NuxtLink class="wake" to="/chat" aria-label="Start a conversation">
      <div class="enter-fade">
        <PresenceOrb :size="180" grounded />
      </div>
      <p class="invitation enter-rise">Ready when you are.</p>
    </NuxtLink>
    <NuxtLink class="cloud-link enter-fade-late" to="/settings/cloud">
      Cloud settings
    </NuxtLink>
  </main>
</template>

<style scoped>
main {
  min-height: 100vh;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 4.5rem;
  background: radial-gradient(
    ellipse 90% 70% at 50% 40%,
    transparent 55%,
    rgba(0, 0, 0, 0.35) 100%
  );
}

.wake {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 4.5rem;
  text-decoration: none;
  cursor: pointer;
}

.invitation {
  margin: 0;
  color: var(--text-secondary);
  font-size: 0.95rem;
  font-weight: 300;
  letter-spacing: 0.06em;
  transition: color 0.3s;
}

.wake:hover .invitation,
.wake:focus-visible .invitation {
  color: var(--text-primary);
}

.cloud-link {
  position: fixed;
  right: 1.5rem;
  bottom: 1.25rem;
  color: color-mix(in srgb, var(--text-secondary) 60%, transparent);
  font-size: 0.78rem;
  letter-spacing: 0.08em;
  text-decoration: none;
}

.cloud-link:hover,
.cloud-link:focus-visible {
  color: var(--text-primary);
}

.enter-fade {
  animation: fade-in 0.9s both;
}

.enter-rise {
  animation: rise-in 0.7s 0.5s both;
}

.enter-fade-late {
  animation: fade-in 0.6s 0.9s both;
}

@keyframes fade-in {
  from {
    opacity: 0;
  }
  to {
    opacity: 1;
  }
}

@keyframes rise-in {
  from {
    opacity: 0;
    transform: translateY(10px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

@media (prefers-reduced-motion: reduce) {
  .enter-fade,
  .enter-rise,
  .enter-fade-late {
    animation: none;
  }
}
</style>
