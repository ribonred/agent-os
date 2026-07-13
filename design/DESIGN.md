# Design system

Governs every screen in `ui/`. The goal: an ambient AI presence that feels
competent and warm at the same time -- closer to a genuinely helpful
assistant than either a cold enterprise dashboard or a gimmicky chatbot
skin. Something like Jarvis, something like TARS, but built for someone
who has never touched a computer beyond a phone.

## Why dark-first

The device is meant to feel *present*, not like a window you open and
close -- closer to an ambient object on a counter or desk than a
traditional app. A dark, glow-friendly palette reads as "the AI is here"
more than a bright, flat one does, and it's easier on the eyes for
something that may stay visible for long stretches in a shop or clinic.
Light mode is a real future need (bright retail counters, accessibility)
but isn't specified here yet -- dark is what ships first.

## Color tokens

```
--bg:            #0B0E14   near-black, not pure black -- softer, less harsh
--surface:       #12151C   cards/panels, one step up from bg
--surface-raised:#1A1E27   modals, elevated elements

--text-primary:  #E8EAED   off-white, not pure white -- less glare
--text-secondary:#8B93A1   muted, for secondary/meta text

--accent:        #3DDCFF   electric cyan -- the AI's "presence" color.
                            Used for the listening/thinking/speaking
                            indicator, active states, focus rings.
--accent-warm:   #F6B673   warm amber/gold -- confirmation, warmth,
                            highlights. Exists specifically so the
                            palette isn't purely cold-tech; pairs the
                            "competent" cyan with something that reads
                            as approachable, matching the "warm, direct"
                            persona tone from brain/constitution.md.

--success:       #4ADE80   distinct from accent-warm on purpose --
                            warmth and "this succeeded" are different
                            meanings and shouldn't share one color.
--danger:        #FF6B6B   errors, destructive-action confirmation --
                            clear but not harsh/alarming.
```

Two accents, not more. Every new UI element should reach for one of the
tokens above before inventing a new color -- if something doesn't fit,
that's a sign the design system needs a deliberate addition, not a
one-off hex value in a component.

## Typography

System font stack (`-apple-system, "Segoe UI", Roboto, sans-serif` plus
platform fallbacks), not a bundled custom webfont. This is a deliberate
choice tied to task 7's language requirement: the device needs to render
Indonesian, English, and likely Chinese/Japanese/Korean/Thai/Vietnamese
script correctly. Bundling one custom font with full coverage for all of
those scripts would be large and still worse than each OS's own
already-tuned system font for that script. Let the OS supply the right
glyphs per locale instead of fighting it.

## Motion

Subtle, ambient, meaningful -- not decorative. The core recurring element
is the AI presence indicator: a soft pulsing glow (using `--accent`) that
changes rhythm by state (idle: slow breathe, listening: faster/brighter,
thinking: a different pattern, speaking: synced to output). Svelte's
built-in transitions (`fade`, `fly`, `scale`) are enough for this --
no external animation library needed yet. A knowledge-graph visualizer is
a real future enhancement (explicitly deferred, not scoped here) that
will need its own motion/interaction spec when it's actually built.

## Supporting tooling

- **`tauri-plugin-log`** (official) -- structured local logging, wired in
  from the start specifically so a build issue produces something to read
  instead of something to guess at. This is what "provide tooling to scan
  logs" actually means in practice.
- **No cloud crash-reporting SaaS** (e.g. Sentry) by default. This is a
  deliberate call, not an oversight: the product's whole positioning is
  "your AI, not the cloud's," and constitution.md already commits to
  routing transparency (local/cloud only disclosed on request). Shipping
  telemetry to a third party by default would contradict that. Local logs
  plus an easy way to pull them off the device when something goes wrong
  is the substitute.
- **`tauri-plugin-store`** for non-secret preferences (selected language,
  selected persona). Deliberately NOT used for the OpenRouter API key --
  that needs OS-keyring-backed storage (task 016), not a plain JSON file.
