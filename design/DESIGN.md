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

### Orb-only tokens

The presence orb is the one element allowed a richer range than the
two-accent rule, because it *is* the product's face -- a flat single-color
glow reads as a template, not a presence. Its palette is still derived
from the system, not free: it blends the two poles the product already
stands on (cool competence = cyan, warm approachability = amber) through
a violet bridge between them. These tokens are for the orb only -- they
never appear on buttons, text, borders, or any other element.

```
--orb-cyan:   #3DDCFF   same as --accent -- the orb's dominant hue
--orb-violet: #7A5CFA   the bridge -- exists only inside the orb's
                         gradient, never as a standalone UI color
--orb-warm:   #F6B673   same as --accent-warm -- a brief flare in the
                         rotation, the "warmth" made literal
--orb-deep:   #1B2A5E   deep blue -- the orb's shadowed side, gives the
                         sphere its volume
```

## The presence orb

The signature element -- every screen carries it, from first boot
onward. Not a flat disc: a layered composition, each layer with one job.

1. **Atmosphere** -- a large, very soft radial glow behind everything,
   breathing slowly (the original idle pulse lives here now).
2. **Core** -- the sphere itself: a conic gradient cycling
   cyan → violet → deep blue → a brief warm flare → cyan, blurred
   slightly and rotating slowly (~20s). Rotation is on the element
   transform, not the gradient angle -- broader webview compat, no
   @property dependency.
3. **Shading** -- a radial specular highlight offset to the upper left
   plus a darker lower edge, which is what makes it read as a sphere
   with volume instead of a colored circle.
4. **Ring** -- one thin, precise luminous ring just outside the core.
   The machined, instrument-like counterpoint to the glow: the TARS
   side of the personality, where the glow is the Jarvis side.
5. **Light pool** (idle/home screen only) -- a soft horizontal ellipse
   of light under the orb, as if it were an object sitting on the
   counter it actually ships to. Grounds it in physical space.

States (idle: slow breathe, listening: faster/brighter, thinking:
different pattern, speaking: synced) modulate the atmosphere and core
timing -- the layer structure never changes per state.

## Boot identity

The product experience starts at power-on, not at the GUI: the boot
sequence must show the brand mark on a dark screen and nothing else --
no scrolling kernel text, no bootloader menu, no login prompt. The mark
is `design/logo.jpg` (white monogram on near-black), displayed by the
boot splash (Plymouth) from early boot until the shell's compositor
takes over: glyph dead-center on pure black, spinner below it. The
build derives a transparent-background RGBA glyph from the JPEG
(alpha from luminance) -- the splash renderer composites no-alpha
images as invisible, and the asset's near-black field would otherwise
show as a grey seam on the pure-black background. The firmware vendor
logo that precedes it is outside the OS's control.

## First-boot greeting

The language-selection screen opens with a cycling greeting -- Halo,
Hello, 你好, こんにちは, 안녕하세요, Xin chào, สวัสดี, … -- one word at a
time, Indonesian first, fading between languages on the list. This is
the one screen where the device cannot know the user's language yet, and
the cycle *is* the answer: it says "I speak yours" in every supported
script before a single choice is made. Fixed-height container so the
swap never shifts layout.

## Naming screen

The second deterministic setup step, and the product's first free-text input:
the device asks the owner to give it a name. This is the emotional peak
of setup -- the moment the box becomes *theirs* -- so it keeps the setup
screens' ceremony, not a form's bureaucracy:

- Same skeleton as the persona screen: orb (72px) above a thin-weight
  h1 ("What will you call me?"), bilingual eyebrow ("Beri saya nama ·
  Give me a name").
- One centered single-line input styled like the conversation input bar
  (quiet `--surface` field, max-width ~420px), submit on Enter or a
  single continue button; the button stays disabled until the trimmed
  input is non-empty. Max length 60 characters; any script.
- No suggestions, no placeholder personality names -- the name is the
  owner's first act of ownership, not a menu choice.

The chosen name lives in the agent's *voice* only (it introduces itself
by name, answers to it). It does NOT become a name badge, avatar, or
header in the UI -- the conversation-surface rule below ("the orb is
the other party") stays exactly as it is.

## Guided onboarding conversation

Naming flows directly into a dedicated conversation where the agent speaks
first. This is still setup, not the normal chat screen:

- Reuse the conversation surface's 48px orb, bare assistant text, quiet owner
  messages, streaming behavior, and single bottom input. The product should
  feel continuous as it moves from being named to getting acquainted.
- No back control and no generic "Ask me anything" empty state. The agent's
  first generated question appears automatically.
- Keep service discovery invisible unless a check fails or the owner asks.
  Postgres/Redis versions are agent context, not a technical setup dashboard.
- Profile review happens in the same conversation. The agent presents one
  compact summary; the owner answers naturally with confirmation or a
  correction. Do not turn the five unknowns into cards or a form.
- Home becomes available only after the confirmed profile is actually saved.
  The transition should feel like the conversation opening up, not a success
  ceremony or administrative completion screen.

## The conversation surface

Chat is the product's primary surface -- not a feature screen, the thing
the device is for. It must read as "talking to the device," never as a
chat app skin:

- **The orb is the other party.** A small presence orb (48px) sits at the
  top of the conversation; there is no assistant avatar, name badge, or
  message bubble on the assistant side. Assistant text renders directly
  on the canvas in `--text-primary`, full measure, like the device is
  speaking into the room.
- **User messages are quiet.** Right-aligned, `--surface` pill, smaller
  type in `--text-secondary`. The user's words are context; the reply is
  the content.
- **Streaming is visible.** Tokens append as they arrive -- no spinner,
  no "typing..." placeholder. Before the first token the orb shifts to
  its thinking rhythm (per Motion); while tokens flow it speaks; idle
  when done. The orb's state IS the status indicator; nothing textual
  duplicates it.
- **One input, one action.** A single quiet input bar pinned at the
  bottom, send on Enter. No toolbar, no attachments yet (ingest comes
  through its own flow later), no model picker -- routing is the
  device's decision (constitution.md discloses it only on request).
- **Errors are spoken, loudly.** A failed backend renders as an error
  line in `--danger` in the conversation flow itself, with the actual
  message -- never a silent retry, never a toast that vanishes.

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
  selected persona, the owner-given agent name). Deliberately NOT used
  for the OpenRouter API key --
  that needs OS-keyring-backed storage (task 016), not a plain JSON file.
