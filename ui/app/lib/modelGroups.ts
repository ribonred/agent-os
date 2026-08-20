// How the model list reads: provider, then maker, then model.
//
// Pure shaping, kept out of the composable so it can be tested the way
// the rest of this project's logic is -- the composable does the talking
// to the native layer, this decides what the owner sees.

export type ModelEntry = {
  id: string;
  name: string;
  family: string;
  familyName: string;
  fast: boolean;
  reasoning: boolean;
  featured: boolean;
};

export type ProviderGroup = {
  slug: string;
  name: string;
  isCurrent: boolean;
  authenticated: boolean;
  models: ModelEntry[];
};

export type ModelOptions = {
  current: string;
  providers: ProviderGroup[];
};

/// One maker's models, under one heading.
export type FamilyGroup = {
  family: string;
  familyName: string;
  models: ModelEntry[];
  /// False when the heading would only repeat the provider directly
  /// above it. A provider the device talks to directly is its own maker,
  /// so "Anthropic → Anthropic" is a heading that says nothing twice.
  showName: boolean;
};

export type ProviderView = {
  slug: string;
  name: string;
  isCurrent: boolean;
  authenticated: boolean;
  families: FamilyGroup[];
  /// How many models this provider has once the search is applied.
  count: number;
};

/// How many colours the scale has. DESIGN.md "Family tokens".
const FAMILY_COLOURS = 10;

/// The makers that get a fixed colour rather than a derived one.
///
/// Two reasons to pin rather than hash everything. The common makers are
/// the ones the owner scans for most, so they are worth a colour chosen
/// to sit apart from its neighbours rather than one that happened to
/// fall out of a hash. And several of these read naturally: NVIDIA green,
/// Anthropic amber.
const PINNED: Record<string, number> = {
  anthropic: 1,
  moonshotai: 2,
  mistralai: 3,
  xiaomi: 4,
  "x-ai": 5,
  deepseek: 6,
  google: 7,
  openai: 8,
  nvidia: 9,
  qwen: 10,
};

/// A small, stable hash. FNV-1a, because the requirement is only that a
/// maker keeps its colour between launches and between devices -- a
/// colour that moves is telling the owner nothing they can rely on, and
/// anything that depends on position in a list moves the moment a
/// provider adds a model.
function hash(text: string): number {
  let value = 0x811c9dc5;
  for (let i = 0; i < text.length; i += 1) {
    value ^= text.charCodeAt(i);
    value = Math.imul(value, 0x01000193) >>> 0;
  }
  return value;
}

/// The colour for one maker, as a CSS variable.
///
/// Decoration only: it sits beside a name already written out in words,
/// so nobody has to tell teal from green to use the screen.
export function familyColor(family: string): string {
  const slug = family.trim().toLowerCase();
  if (!slug) return "var(--family-none)";
  const pinned = PINNED[slug];
  if (pinned) return `var(--family-${pinned})`;
  return `var(--family-${(hash(slug) % FAMILY_COLOURS) + 1})`;
}

/// Search across everything the owner can see on a row, so typing what
/// is in front of them finds it -- the maker, the model, the provider.
export function matches(model: ModelEntry, provider: string, query: string) {
  const needle = query.trim().toLowerCase();
  if (!needle) return true;
  const haystack = [
    model.id,
    model.name,
    model.familyName,
    model.family,
    provider,
  ]
    .join(" ")
    .toLowerCase();
  // Every word must appear somewhere, so "claude opus" finds it and the
  // order the owner types them in does not matter.
  return needle.split(/\s+/).every((word) => haystack.includes(word));
}

/// Provider, then maker. Featured models lead each maker -- Hermes'
/// shortlist is the closest thing to a recommendation the device has,
/// and thirty-seven models is a list where six is a choice.
export function group(providers: ProviderGroup[], query: string): ProviderView[] {
  return providers
    .map((provider) => {
      const kept = provider.models.filter((m) => matches(m, provider.name, query));
      const families = new Map<string, FamilyGroup>();
      for (const model of kept) {
        const existing = families.get(model.family);
        if (existing) existing.models.push(model);
        else
          families.set(model.family, {
            family: model.family,
            familyName: model.familyName,
            models: [model],
            showName:
              model.familyName.trim().toLowerCase() !==
              provider.name.trim().toLowerCase(),
          });
      }
      for (const family of families.values()) {
        family.models.sort(
          (a, b) => Number(b.featured) - Number(a.featured) || a.name.localeCompare(b.name),
        );
      }
      return {
        slug: provider.slug,
        name: provider.name,
        isCurrent: provider.isCurrent,
        authenticated: provider.authenticated,
        families: [...families.values()].sort((a, b) =>
          a.familyName.localeCompare(b.familyName),
        ),
        count: kept.length,
      };
    })
    // A provider with nothing left after a search is not an answer.
    .filter((provider) => provider.count > 0);
}
