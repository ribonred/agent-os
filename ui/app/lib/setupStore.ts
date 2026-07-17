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

// Setup is "done" once both are chosen -- this is what gates the root
// screen per onboarding.md's mandatory-before-any-conversation rule.
export async function isSetupComplete(): Promise<boolean> {
  const [language, persona] = await Promise.all([getLanguage(), getPersona()]);
  return language !== null && persona !== null;
}
