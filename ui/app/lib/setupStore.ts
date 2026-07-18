import { Store } from "@tauri-apps/plugin-store";
import type { LanguageCode, PersonaId } from "./setupOptions";

const STORE_FILE = "settings.json";

let storePromise: ReturnType<typeof Store.load> | null = null;

function getStore() {
  if (!storePromise) {
    storePromise = Store.load(STORE_FILE);
  }
  return storePromise;
}

export async function getLanguage(): Promise<LanguageCode | null> {
  const store = await getStore();
  return (await store.get<LanguageCode>("language")) ?? null;
}

export async function setLanguage(code: LanguageCode): Promise<void> {
  const store = await getStore();
  await store.set("language", code);
  await store.save();
}

export async function getPersona(): Promise<PersonaId | null> {
  const store = await getStore();
  return (await store.get<PersonaId>("persona")) ?? null;
}

export async function setPersona(id: PersonaId): Promise<void> {
  const store = await getStore();
  await store.set("persona", id);
  await store.save();
}

export async function getAgentName(): Promise<string | null> {
  const store = await getStore();
  const name = (await store.get<string>("agentName")) ?? null;
  return name && name.trim() !== "" ? name : null;
}

export async function setAgentName(name: string): Promise<void> {
  const store = await getStore();
  await store.set("agentName", name.trim());
  await store.save();
}

// Setup is "done" once all three are chosen -- this is what gates the
// root screen per onboarding.md's mandatory-before-any-conversation
// rule.
export async function isSetupComplete(): Promise<boolean> {
  return (await firstIncompleteSetupStep()) === null;
}

// The first missing step in flow order, or null when setup is complete.
// This is also the whole migration story: a device set up before the
// naming step existed has language + persona but no name, and lands
// directly on /setup/name instead of redoing the flow.
export async function firstIncompleteSetupStep(): Promise<string | null> {
  const [language, persona, name] = await Promise.all([
    getLanguage(),
    getPersona(),
    getAgentName(),
  ]);
  if (language === null) return "/setup/language";
  if (persona === null) return "/setup/persona";
  if (name === null) return "/setup/name";
  return null;
}
