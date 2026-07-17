// Tauri + Nuxt per the official integration guide (v2.tauri.app):
// no SSR (Tauri is a static webview host), fixed port for the dev
// server, TAURI_ env prefix exposed, src-tauri excluded from watch.
export default defineNuxtConfig({
  compatibilityDate: "2025-07-15",
  devtools: { enabled: false },
  telemetry: false,
  ssr: false,
  css: ["~/assets/css/main.css"],
  devServer: {
    host: "0",
  },
  vite: {
    clearScreen: false,
    envPrefix: ["VITE_", "TAURI_"],
    server: {
      strictPort: true,
    },
  },
  ignore: ["**/src-tauri/**"],
});
