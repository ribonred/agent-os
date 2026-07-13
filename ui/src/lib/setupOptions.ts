// Mirrors brain/onboarding.md's "Mandatory device setup" section exactly.
// Change that doc first, then this file, never the other way round.

export type LanguageCode =
  | "id"
  | "en"
  | "zh"
  | "ja"
  | "ko"
  | "vi"
  | "th"
  | "ms"
  | "tl"
  | "hi";

export interface LanguageOption {
  code: LanguageCode;
  label: string;
  nativeLabel: string;
}

// Indonesia first, deliberately not alphabetical -- primary market.
export const LANGUAGE_OPTIONS: LanguageOption[] = [
  { code: "id", label: "Indonesian", nativeLabel: "Bahasa Indonesia" },
  { code: "en", label: "English", nativeLabel: "English" },
  { code: "zh", label: "Chinese (Simplified)", nativeLabel: "中文（简体）" },
  { code: "ja", label: "Japanese", nativeLabel: "日本語" },
  { code: "ko", label: "Korean", nativeLabel: "한국어" },
  { code: "vi", label: "Vietnamese", nativeLabel: "Tiếng Việt" },
  { code: "th", label: "Thai", nativeLabel: "ภาษาไทย" },
  { code: "ms", label: "Malay", nativeLabel: "Bahasa Melayu" },
  { code: "tl", label: "Filipino", nativeLabel: "Tagalog" },
  { code: "hi", label: "Hindi", nativeLabel: "हिन्दी" },
];

export type PersonaId = "balanced" | "warm-patient" | "straight-efficient" | "formal-precise";

export interface PersonaOption {
  id: PersonaId;
  label: string;
  description: string;
}

export const PERSONA_OPTIONS: PersonaOption[] = [
  {
    id: "balanced",
    label: "Balanced",
    description: "Warm and direct in equal measure. Recommended default.",
  },
  {
    id: "warm-patient",
    label: "Warm & Patient",
    description:
      "More encouragement, more explanation per answer, slower pace.",
  },
  {
    id: "straight-efficient",
    label: "Straight & Efficient",
    description: "Minimal small talk, gets to the point fast.",
  },
  {
    id: "formal-precise",
    label: "Formal & Precise",
    description: "A more measured, professional register.",
  },
];
