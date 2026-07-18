import { invoke } from "@tauri-apps/api/core";

const READY_ATTEMPTS = 60;
const READY_INTERVAL_MS = 1000;

function delay(milliseconds: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, milliseconds));
}

export async function waitForAgentReady(): Promise<void> {
  let lastError: unknown;
  for (let attempt = 0; attempt < READY_ATTEMPTS; attempt += 1) {
    try {
      await invoke("agent_status");
      return;
    } catch (error) {
      lastError = error;
    }
    if (attempt + 1 < READY_ATTEMPTS) {
      await delay(READY_INTERVAL_MS);
    }
  }
  throw new Error(
    `The agent gateway did not become ready within 60 seconds: ${String(lastError)}`,
  );
}