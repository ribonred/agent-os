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
5. **Light pool** (setup screens only) -- a soft horizontal ellipse
   of light under the orb, as if it were an object sitting on the
   counter it actually ships to. Grounds it in physical space. Setup is
   where the orb stands alone and gets the full screen; once the device
   is in use the orb lives at the top of the conversation pane at 56px,
   beside the owner's things rather than in front of them.

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

- Reuse the conversation surface's bare assistant text, quiet owner messages,
  streaming behavior, and single bottom input, with the orb at 48px. Setup is
  full-screen and centered rather than the two-pane shell, so the orb stays at
  its setup size here; it settles to 56px in the conversation pane once the
  device is in use. The product should feel continuous as it moves from being
  named to getting acquainted.
- No back control and no generic "Ask me anything" empty state. The agent's
  first generated question appears automatically.
- Keep service discovery invisible unless a check fails or the owner asks.
  Postgres/Redis versions are agent context, not a technical setup dashboard.
- Profile review happens in the same conversation. The agent presents one
  compact summary; the owner answers naturally with confirmation or a
  correction. Do not turn the five unknowns into cards or a form.
- The device becomes usable only after the confirmed profile is actually
  saved. Setup then gives way to the two-pane shell: the conversation the
  owner was just having settles into its pane on the left, and their own
  files appear beside it. The transition should feel like the conversation
  opening up -- literally, here -- not a success ceremony or an
  administrative completion screen.

## The conversation surface

Chat is the product's primary surface -- not a feature screen, the thing
the device is for. It must read as "talking to the device," never as a
chat app skin:

- **A persistent pane, not a page.** Once setup is done the conversation
  occupies a fixed-width column on the left, beside the file view (below).
  It is never navigated away from: the owner can look through their
  things while the device is still speaking, and a reply that started
  before they moved keeps streaming. The pane may collapse to a narrow
  rail when someone wants the width, but the orb is never removed from
  the screen -- a screen with no orb reads as a device that is switched
  off.
- **The orb is the other party.** A small presence orb (56px) sits at the
  top of the conversation pane; there is no assistant avatar, name badge, or
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
  bottom, send on Enter. No toolbar, no attach button, no file picker,
  no model picker -- routing is the device's decision
  (constitution.md discloses it only on request).
- **Context comes from selection, not from attaching.** Touching
  a file places one quiet `--surface` chip directly
  above the input naming what was selected, with a single dismiss
  control. The next message carries it; sending clears it. Never more
  than one chip at a time.

  This supersedes an earlier "no attachments" rule, and the distinction
  is the whole point rather than a loophole: there is no picker and no
  dialog, and nothing can become context that isn't already visible on
  screen. The owner points at a thing they can see instead of
  navigating a hierarchy to find it, which is the difference between an
  affordance a non-technical owner can use and one they cannot. "One
  input, one action" is intact -- the chip is a statement of what
  they're looking at, not a second control.
- **Errors are spoken, loudly.** A failed backend renders as an error
  line in `--danger` in the conversation flow itself -- never a silent
  retry, never a toast that vanishes.

  This holds on every surface, with the error placed where the failure
  is: something that went wrong with one file or folder renders on that
  row or tile; something that broke the whole surface replaces the
  content area; something the agent hit stays in the conversation. No
  toasts anywhere, a file surface included -- that is precisely where
  the reflex to add one is strongest, and a message that vanishes is a
  message the owner didn't read.

  What's shown is always written for the owner, never the underlying
  system's own words. "I couldn't read this one" is the message; the
  raw string from any layer goes to the log, not to the screen.
  constitution.md forbids surfacing error codes and system state, and
  a raw error string passed straight through to the UI is exactly that.

## The file view

The other half of the main surface, beside the conversation. It is **a
file manager**: the owner's real directories, nested to whatever depth
their disk actually has, showing what is really there. The agentic part
is not that the files are curated, digested, or re-labelled before the
owner sees them -- it is that they can point at something here and ask
the device about it.

An earlier version of this section framed this surface as "shelves" --
a curated library of what the device had been given, with cleaned-up
names and no nesting. That was wrong, and wrong in a specific way worth
recording: it invented a model the product does not have. The owner's
files are their own, they already have a shape, and a surface that
paraphrases that shape makes their device *harder* to reason about, not
easier. Familiarity is the accessibility win here, not abstraction.

The owner has no other way to reach their files. The kiosk session runs
this shell and nothing else -- no desktop, no terminal, no second file
manager to fall back on. Every affordance they need must exist here, and
every one they don't need is weight they carry.

- **Show what is actually there.** Real filenames with their extensions,
  real sizes, real dates. `price-list-2026.xlsx` is
  `price-list-2026.xlsx`. Someone should be able to match what they see
  here against what they'd see anywhere else that names the same file.
- **Folders and files in one list, folders first, then alphabetical.**
  What a file manager does. Ordering by name rather than by recency
  keeps a directory's shape stable as its contents change, so the thing
  the owner learned the position of stays where they left it.
  Case-insensitive: names sort the way a person reads them, not the way
  ASCII orders them.
- **A folder says what it holds; a file says how big it is.** Enough to
  decide whether to open something, and nothing more.
- **No absolute paths.** The breadcrumb names folders from the owner's
  own files downward -- no leading slash, no home directory, nothing
  above where they can actually go. The device's own filesystem is not
  something the owner should have to think about, and there is nowhere
  above their home for them to navigate to anyway.
- **Hidden files stay hidden.** Dotfiles are the machine's business.
- **Colour encodes state, never identity.** Files are never
  colour-coded by kind: that is an icon's job, and a colour legend is
  something the owner has to learn. `--accent` marks focus or where the
  device is pointing; `--danger` marks a genuine failure.
- **The gestures are the ones people already have.** Single click
  selects; double click opens a folder; ctrl (or cmd) click adds and
  removes one; shift click takes the range. These are not chosen for
  elegance -- they are chosen because someone who has used any computer
  before already knows them, and inventing a "simpler" scheme here would
  mean the owner has to learn this device specifically. Familiarity is
  the accessibility win.

  Selection and focus are shown differently: a selected row is filled
  with `--accent` at low opacity, the focused row carries a ring. Once
  more than one row can be picked, "where I am" and "what I chose" are
  genuinely different questions and must not share one indicator.

  Everything reachable by mouse is reachable by keyboard -- arrows move,
  shift+arrow extends, Enter opens, Escape clears. The device ships to
  counters where a mouse may not be the thing at hand.

- **Selection does not survive leaving the folder.** Opening a different
  directory clears it, and so does a file disappearing from underneath
  it. A selection that points at something the owner can no longer see
  is worse than no selection.

- **The list does not render what's inside a file.** Reading a document
  is what the conversation is for -- asking the device is a better
  answer than a cramped preview beside a chat pane.

  It also keeps the webview's reach at zero. Every filesystem operation
  stays behind a small set of named actions in the native layer, which
  never hands the browser a path or the ability to read a file directly.
  Paths that arrive from the interface are re-resolved and checked
  against the owner's home before anything is read. On a device sold to
  someone who will never audit it, that boundary is worth more than a
  preview pane.

This surface introduces **no new colour tokens**. Everything above is
`--surface`, `--surface-raised`, `--text-primary`, `--text-secondary`,
`--accent`, `--danger`.

## Icons

Kind is communicated by a small line icon at the head of every row: a
folder for a directory, and for a file, what kind of file it is --
spreadsheet, document, picture, sound, video, archive, plain text, or a
neutral mark when the device cannot tell.

These come from an icon set (Lucide), imported one icon at a time so that
only the handful actually used is bundled. **Nothing is ever fetched at
runtime** -- the device may never see a network, and a surface that
degrades without one is not an appliance. That constraint, not the
choice of set, is the part that matters if this is ever revisited.

An earlier version of this section drew kind as abstract "motifs" of bars
and blocks instead, to avoid taking on an icon dependency at all. That
was the wrong trade: hand-drawn shapes cost far more to build and tune
than the dependency saved, and they read as smudges rather than as
things. A conventional icon is also *easier* for the owner -- a folder
that looks like a folder needs no learning, which is the whole point.

What survives from that reasoning, and still holds:

- **Icons orient; the name informs.** They are drawn quietly in
  `--text-secondary`, never at full contrast, and they sit beside the
  name rather than competing with it.
- **Never coloured, never badged.** Colour on this surface means state,
  not kind. An icon tinted to signal something is a legend the owner has
  to learn.
- **One mark per kind the view genuinely distinguishes**, and no more.
  A larger icon set is a larger vocabulary, and every extra symbol is one
  more thing to decode.
- **A directory always shows a folder**, whatever is inside it. What it
  holds is said by its item count; borrowing one file's icon to stand
  for a whole directory would misdescribe it. A folder reads a step
  brighter than the files beside it, because it is the thing you
  navigate by.

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
thinking: a different pattern, speaking: synced to output). CSS
transitions and keyframes are enough for this -- no external animation
library needed yet. A knowledge-graph visualizer is a real future
enhancement (explicitly deferred, not scoped here) that will need its own
motion/interaction spec when it's actually built.

**The pointing gesture.** When a reply concerns a particular file or
folder, that row breathes once -- a single `--accent` ring, roughly
900ms, then nothing. This is the device pointing at what it's talking
about, standing in for the gesture a person would make, and it is the
only motion in the file view that isn't an enter or exit transition. It
never repeats and never persists: a row left permanently highlighted is
decoration, which is the one thing this system's motion rule forbids.

**Things arrive visibly.** When a file appears in the directory being
viewed -- the owner copied it in, or asked the device to fetch it -- the
row enters with the same quiet rise-and-fade used elsewhere, rather than
being there on the next repaint as if it had always been. Watching the
device put something down is what makes it read as an assistant doing
work instead of a folder that changed behind your back.

Every animation here honours `prefers-reduced-motion`.

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
