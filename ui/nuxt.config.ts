// Tauri + Nuxt per the official integration guide (v2.tauri.app):
// no SSR (Tauri is a static webview host), fixed port for the dev
// server, TAURI_ env prefix exposed, src-tauri excluded from watch.
export default defineNuxtConfig({
  compatibilityDate: "2025-07-15",
  devtools: { enabled: false },
  telemetry: false,
  ssr: false,
  css: ["~/assets/css/main.css"],
  // Dev-only. The desktop shell waits for the dev server on
  // http://localhost:3000, so bind exactly that. Binding "0" (0.0.0.0)
  // also serves localhost, but Nuxt then prints only "Network:" URLs
  // and no "Local:" line, hiding the address the shell actually polls
  // and making a failed handoff look like a mystery. None of this
  // reaches the device: the packaged app loads pre-built static files
  // and never starts a dev server.
  devServer: {
    host: "127.0.0.1",
    port: 3000,
  },
  vite: {
    clearScreen: false,
    envPrefix: ["VITE_", "TAURI_"],
    server: {
      strictPort: true,
    },
  },
  // Test files run under `bun test`, which supplies its own globals and
  // module types. Nuxt's app typecheck knows nothing about those, so
  // leaving them in scope reports a phantom missing-module error on a
  // file that is not part of the shipped app anyway.
  ignore: ["**/src-tauri/**", "**/*.test.ts"],
  typescript: {
    tsConfig: {
      exclude: ["../app/**/*.test.ts"],
    },
  },
});
