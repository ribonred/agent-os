import { describe, expect, it } from "bun:test";
import { familyColor, group, matches, type ProviderGroup } from "./modelGroups";

const model = (
  id: string,
  familyName: string,
  extra: Partial<{ featured: boolean; fast: boolean }> = {},
) => ({
  id,
  name: id.split("/").pop()!,
  family: id.includes("/") ? id.split("/")[0]! : "anthropic",
  familyName,
  fast: extra.fast ?? false,
  reasoning: true,
  featured: extra.featured ?? false,
});

const PROVIDERS: ProviderGroup[] = [
  {
    slug: "openrouter",
    name: "OpenRouter",
    isCurrent: true,
    authenticated: true,
    models: [
      model("openai/gpt-5.5", "OpenAI"),
      model("anthropic/claude-opus-5", "Anthropic", { featured: true }),
      model("anthropic/claude-haiku-4.5", "Anthropic", { fast: true }),
      model("x-ai/grok-4.6", "xAI"),
    ],
  },
  {
    slug: "anthropic",
    name: "Anthropic",
    isCurrent: false,
    authenticated: true,
    models: [model("claude-fable-5", "Anthropic")],
  },
];

describe("finding a model", () => {
  it("matches on the maker, the model or the provider", () => {
    const opus = PROVIDERS[0]!.models[1]!;
    expect(matches(opus, "OpenRouter", "anthropic")).toBe(true);
    expect(matches(opus, "OpenRouter", "opus")).toBe(true);
    expect(matches(opus, "OpenRouter", "openrouter")).toBe(true);
    expect(matches(opus, "OpenRouter", "grok")).toBe(false);
  });

  it("takes the words in whatever order they were typed", () => {
    const opus = PROVIDERS[0]!.models[1]!;
    expect(matches(opus, "OpenRouter", "claude opus")).toBe(true);
    expect(matches(opus, "OpenRouter", "opus claude")).toBe(true);
    // Every word has to land, so a search narrows rather than widens.
    expect(matches(opus, "OpenRouter", "opus gemini")).toBe(false);
  });

  it("shows everything when nothing has been typed", () => {
    expect(matches(PROVIDERS[0]!.models[0]!, "OpenRouter", "   ")).toBe(true);
  });
});

describe("grouping models", () => {
  it("groups by maker inside each provider", () => {
    const [openrouter] = group(PROVIDERS, "");
    expect(openrouter!.families.map((f) => f.familyName)).toEqual([
      "Anthropic",
      "OpenAI",
      "xAI",
    ]);
    expect(openrouter!.families[0]!.models).toHaveLength(2);
  });

  it("leads each maker with the shortlisted model", () => {
    const [openrouter] = group(PROVIDERS, "");
    const anthropic = openrouter!.families.find((f) => f.familyName === "Anthropic")!;
    // Thirty-seven models is a list; the featured one is the closest
    // thing to a recommendation the device has.
    expect(anthropic.models[0]!.id).toBe("anthropic/claude-opus-5");
    expect(anthropic.models[0]!.featured).toBe(true);
  });

  it("drops a provider a search has emptied", () => {
    const groups = group(PROVIDERS, "grok");
    expect(groups).toHaveLength(1);
    expect(groups[0]!.slug).toBe("openrouter");
    expect(groups[0]!.count).toBe(1);
  });

  it("keeps a direct provider's bare id under its own maker", () => {
    // Anthropic returns "claude-fable-5", not "anthropic/claude-fable-5".
    const groups = group(PROVIDERS, "fable");
    expect(groups).toHaveLength(1);
    expect(groups[0]!.slug).toBe("anthropic");
    expect(groups[0]!.families[0]!.familyName).toBe("Anthropic");
  });

  it("returns nothing rather than everything when a search matches nothing", () => {
    expect(group(PROVIDERS, "not-a-model")).toEqual([]);
  });
});

describe("colouring a maker", () => {
  it("gives a maker the same colour every time", () => {
    // A colour that moves between launches tells the owner nothing they
    // can rely on, which is worse than no colour at all.
    for (const family of ["anthropic", "sakana", "tencent", "some-new-lab"]) {
      expect(familyColor(family)).toBe(familyColor(family));
    }
  });

  it("pins the makers the owner scans for most", () => {
    expect(familyColor("nvidia")).toBe("var(--family-9)");
    expect(familyColor("anthropic")).toBe("var(--family-1)");
    expect(familyColor("openai")).toBe("var(--family-8)");
    // Case and stray whitespace are the same maker.
    expect(familyColor("  OpenAI ")).toBe("var(--family-8)");
  });

  it("still finds a colour for a maker it has never heard of", () => {
    for (const family of ["sakana", "stepfun", "tencent", "moa", "zzz-lab"]) {
      expect(familyColor(family)).toMatch(/^var\(--family-([1-9]|10)\)$/);
    }
  });

  it("has a colour for no maker at all", () => {
    expect(familyColor("")).toBe("var(--family-none)");
    expect(familyColor("   ")).toBe("var(--family-none)");
  });

  it("never reaches for a semantic colour", () => {
    // --accent means the assistant itself, --success and --danger mean
    // what they say. A maker must never borrow one.
    const used = new Set(
      ["anthropic", "openai", "google", "sakana", "moa", "zzz"].map(familyColor),
    );
    for (const colour of used) {
      expect(colour).not.toContain("accent");
      expect(colour).not.toContain("success");
      expect(colour).not.toContain("danger");
    }
  });
});

describe("a heading that would say nothing twice", () => {
  it("drops a maker heading that only repeats its provider", () => {
    // The Anthropic provider's own models are Anthropic's. Showing
    // "Anthropic" under "ANTHROPIC" reads as a duplicate, not a group.
    const groups = group(PROVIDERS, "fable");
    const direct = groups.find((g) => g.slug === "anthropic")!;
    expect(direct.families[0]!.showName).toBe(false);
  });

  it("keeps the heading when the maker is not the provider", () => {
    const [openrouter] = group(PROVIDERS, "");
    for (const family of openrouter!.families) {
      expect(family.showName).toBe(true);
    }
  });
});
