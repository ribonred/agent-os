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

export async function beginOnboarding(name: string): Promise<void> {
  const store = await getStore();
  await store.set("agentName", name.trim());
  await store.set("onboardingStarted", true);
  await store.set("onboardingQuestionCount", 0);
  // Drop any half-finished checklist from a previous attempt so the
  // shell driver starts at owner_name again.
  await store.delete("onboardingState");
  await store.save();
}

export async function getOnboardingQuestionCount(): Promise<number> {
  const store = await getStore();
  const count = (await store.get<number>("onboardingQuestionCount")) ?? 0;
  return Number.isInteger(count) && count >= 0 ? count : 0;
}

export async function setOnboardingQuestionCount(count: number): Promise<void> {
  const store = await getStore();
  await store.set("onboardingQuestionCount", Math.max(0, Math.min(15, Math.trunc(count))));
  await store.save();
}

export async function completeOnboarding(): Promise<void> {
  const store = await getStore();
  await store.set("onboardingComplete", true);
  await store.save();
}

export async function isSetupComplete(): Promise<boolean> {
  return (await firstIncompleteSetupStep()) === null;
}

// A persona is never written by the new two-screen flow, so all three
// legacy values together prove that an upgraded device completed the
// old setup. Incomplete old setups join the new flow at the first value
// they still need.
export async function firstIncompleteSetupStep(): Promise<string | null> {
  const store = await getStore();
  const [language, persona, name, onboardingStarted, onboardingComplete] = await Promise.all([
    getLanguage(),
    getPersona(),
    getAgentName(),
    store.get<boolean>("onboardingStarted"),
    store.get<boolean>("onboardingComplete"),
  ]);
  if (language === null) return "/setup/language";
  if (name === null) return "/setup/name";
  if (onboardingComplete === true) return null;
  if (persona !== null && onboardingStarted !== true) return null;
  return "/setup/onboarding";
}
