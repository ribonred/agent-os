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

### Family tokens

One place needs more colours than two accents can give it: the model
picker, where the owner is scanning a list of fifty models belonging to
seventeen different makers. Grouped under headings alone, that list is a
wall of near-identical text, and the maker is the thing the owner is
actually navigating by.

This is the deliberate addition the rule above asks for, not a one-off:
the palette lives here, it is a fixed scale rather than a colour per
maker invented on the day, and it is used in exactly one place.

```
--family-1:  #E8A33D   amber
--family-2:  #E2703A   orange
--family-3:  #D9584B   clay
--family-4:  #C75CA8   magenta
--family-5:  #A87BE8   violet
--family-6:  #6E7FE8   indigo
--family-7:  #4FA3E8   blue
--family-8:  #3FB8A8   teal
--family-9:  #5FBF6A   green
--family-10: #9CBF3F   lime
--family-none: #8A93A8 slate -- a maker the scale has no entry for
```

Four rules keep this from becoming a second palette competing with the
first:

- **Decoration only, and never the only carrier of meaning.** A family
  colour appears as a small mark beside a name that is already written
  out in words. Nobody has to distinguish teal from green to use the
  screen; the colour is there to make the scanning quicker, and removing
  every one of them would lose speed and no information. That redundancy
  is also what makes a colour-coded list acceptable for someone who
  cannot tell two of these apart.

- **Never on text, never on a control.** The mark is a dot and a faint
  rule. A coloured label would compete with `--accent`, which on this
  device means the assistant itself and must not come to mean "made by
  Google".

- **Deliberately not the semantic colours.** None of these is
  `--success`, `--danger` or `--accent`, and the green and blue here are
  shifted away from them on purpose. A green dot beside a model must not
  read as "this one is working"; a red one must not read as "something is
  wrong with this".

- **A maker keeps its colour.** Assignment is fixed, not by position in a
  list that changes when a provider adds a model. If Anthropic is violet
  today it is violet next week, or the colour is telling the owner
  nothing they can rely on.

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

## Window modes

The assistant runs as an application on Ubuntu's own desktop session, not
as the only thing the screen can show. That makes the window itself a
design surface with two states, and the owner moves between them freely:

- **Full.** Maximized and undecorated: the two-pane shell, conversation
  on the left, the owner's files beside it. This is the place the owner
  returns to, and what the session starts in.

  Maximized rather than true fullscreen, deliberately. The desktop
  underneath is meant to stay reachable but quiet; a fullscreen window
  hides the system bar the owner may need to get back to something else,
  which turns "quiet" into "hidden" and makes the device feel like it is
  trapping them.

  Undecorated does not mean fixed. A window with no decoration also has
  no title bar to drag and no border to pull, and on a desktop where
  every other window can be moved and sized to fit, one that cannot is
  not minimal -- it is stuck. So the shell supplies both itself: the top
  of the conversation pane behaves as the title bar (drag to move,
  double-click to fill the screen or give it back), and a few pixels at
  each edge and corner carry the resize cursor and hand the drag to the
  window manager. Neither draws anything. The owner should find the
  window behaves the way windows do, and never notice why.

  A window the owner has sized themselves is the one they get back. The
  shell remembers whether the full window was maximized and, when it was
  not, where it sat and how big it was, so returning from the pill
  restores what they left rather than filling the screen over the top of
  whatever they had arranged. Only a device that has never been sized
  falls back to maximized, which is the right first impression and a
  poor second one.

- **Minimized.** A small floating pill -- the orb and one input -- that
  stays on top of whatever else is open and follows the owner across
  workspaces. They drag it wherever it suits them and it is still there
  next time the device starts.

  This is the mode that makes the assistant an assistant rather than a
  destination. Someone reading an invoice in a browser or typing into a
  spreadsheet can ask a question without leaving what they are doing,
  which is exactly the moment they most want to.

  It grows while there is something to read -- the last few turns of the
  conversation, a permission card, a question waiting for an answer --
  and settles back to a single line when the exchange is done. The pill
  is never a window the owner has to manage: no title bar, no scrollback
  to hunt through, no second copy of the conversation, and no resize
  border -- it sizes itself to what there is to read, and a hand-sized
  pill would only fight that. It is the same conversation, seen through
  a smaller opening.

Both modes carry the orb, and switching between them never interrupts a
reply in progress. Those two constraints are what make this one shell in
two shapes rather than two products: the orb is the device's presence and
a screen without it reads as switched off, and a reply that dies because
the owner wanted their screen back would teach them not to use the pill
at all.

The transition is a resize, not a new window. What the owner was reading
is what they keep reading.

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

  The pane holds *a* conversation rather than *the* conversation. The
  device reopens the last one when it starts, because it is a thing that
  sits on a counter and gets picked up mid-thought -- "what did we decide
  about the Tuesday invoice?" has to have an answer the morning after.
  Starting a new one is a deliberate act, never something a restart does
  on the owner's behalf.
- **Earlier conversations are one tap away, and they cover the pane.**
  Two quiet icons join the orb in the pane's control row: start a new
  conversation, and open the earlier ones. Nothing new is added around
  the pane for this -- no second sidebar, no third column. The device is
  the assistant beside the owner's work, and a list of past chats is not
  worth permanently narrowing the work to hold.

  Opening it covers the conversation column, full height. Picking one
  loads it and the list closes; that is the whole interaction. Covering
  rather than opening beside means the list has the pane's full measure
  to be readable in, and means this behaves identically at every window
  size instead of having a layout that only works when maximized.

  One row per conversation: what it was called, a line of what it was
  about, and when. Grouped by *Today*, *Yesterday*, *Earlier this week*,
  *Before that* -- the owner remembers when they asked something far
  better than they remember what the thread ended up being called, and a
  flat list sorted by a date they have to read is a list they have to
  read all of.
- **The device names its own conversations.** A title is written from the
  owner's opening words, and it may improve a second or two after a
  conversation begins as the device settles on a better one. A
  conversation with nothing in it yet reads "New conversation".

  Never an id, and never a timestamp standing in for a name. A row
  labelled with machine identity is the device admitting it did not
  understand what was said to it, on the one surface whose entire job is
  to prove otherwise.
- **Keep, rename, delete -- and nothing else.** A kept conversation sits
  at the top of the list under its own heading; "keep" is the owner's
  word for it, because "pin" describes the mechanism rather than the
  intent.

  Renaming and deleting live behind a per-row menu, never on the row
  itself. A delete control a few pixels from something the owner taps
  constantly is a trap, and this list is made of things that cannot be
  got back. Deleting asks once, in plain language, and says what is being
  lost.

  There is deliberately no archive. Something the owner is told is put
  away, but which they cannot then go and find, is worse than either
  keeping it or deleting it honestly.
- **Switching is a full-size act.** The floating pill has no list: it is
  the conversation the owner is already in, and choosing between things
  is not what that shape is for. While a reply is arriving, the list
  still opens and reads, but the rows do not respond -- the reply has
  somewhere to land and a question the device asked has somewhere to be
  answered. The way to leave early is the Stop control that is already
  there.
- **The orb is the other party.** A small presence orb (56px) sits at the
  top of the conversation pane; there is no assistant avatar, name badge, or
  message bubble on the assistant side. Assistant text renders directly
  on the canvas in `--text-primary`, full measure, like the device is
  speaking into the room.
- **User messages are quiet.** Right-aligned, `--surface` pill, smaller
  type in `--text-secondary`. The user's words are context; the reply is
  the content.
- **The reply is rendered, not dumped.** Assistant text is markdown. A
  list that arrives as `- one` `- two` and renders as literal hyphens
  tells the owner the device is showing them raw machine output, which is
  the opposite of what this product claims to be.

  The subset, in full: paragraphs and line breaks, bold, italic,
  strikethrough, bulleted and numbered lists, headings, blockquote,
  inline code, fenced code blocks with a way to copy them, horizontal
  rules, and **tables**. This list is the contract the agent is written
  against (`brain/chat-protocol.md`) and the list the sanitizer enforces
  (`ui/app/lib/markdown.ts`); the three move together or the device
  promises something the screen does not do.

  Markdown adds structure, not chrome. The bare-text-on-canvas rule above
  is unchanged: no bubble, no avatar, no card around a reply. Headings
  are quieter than the body would suggest -- a reply is speech, not a
  document, and a large heading in a narrow pane reads as shouting.
  Anything wider than the subset is a request to revisit this section,
  not a component to add on the day.

- **A table is for answering with data, not for decorating an answer.**
  Anything the owner would otherwise have to hold in their head across
  several sentences -- three months of totals, what is in five files,
  this option against that one -- reads better as rows. That is the case
  the shell renders tables for, and the agent is told to reach for one
  there.

  It is also the case where the pane's measure bites: a table wider than
  a few short columns scrolls sideways, and a column the owner has to
  drag to see is one they will not read. Few columns, short headers, and
  prose for anything that is really a sentence.

  Anything wide -- a code block, a table -- scrolls inside its own
  container rather than widening the pane. The pane has a measure that
  suits bare prose and nothing is allowed to widen it.

- **Nothing in a reply is fetched, and nothing in a reply is clickable.**
  Images never render and links are not a tag this surface has: the
  device may never see a network, and a broken-image icon or a dead link
  in the middle of an answer is worse than the text the model wrote.

  This is a constraint on how the agent writes, not only on what the
  renderer strips. A web address written as a markdown link loses the
  address entirely and leaves the owner holding the label -- so an
  address that matters is written out as plain text, where it survives
  and can be read aloud, copied, or typed.
- **Streaming is visible.** Tokens append as they arrive -- no spinner,
  no "typing..." placeholder. Before the first token the orb shifts to
  its thinking rhythm (per Motion); while tokens flow it speaks; idle
  when done. The orb's state IS the status indicator; nothing textual
  duplicates it.
- **Work is visible, quietly.** When the device does something rather
  than just answering -- reads a file, runs a command, searches -- it
  says so on one `--text-secondary` line in the flow: "Looked through
  your files." One line per action, in the owner's language, expandable
  to the detail for anyone who wants it and silent about it for everyone
  else.

  This is not a progress indicator and must never become one. The orb
  already says the device is working; these lines say *what* it did, which
  is a different question and the one that earns trust. A device that goes
  quiet for twenty seconds and then produces an answer is asking to be
  taken on faith. Collapsed by default, because the owner asked a
  question, not for a log.
- **Permission is asked in the conversation.** When the device needs
  consent before doing something consequential, the request appears in
  the flow where the exchange is happening -- not as a modal over it,
  and not as a toast.

  Plain language first: what it is about to do and why, in a sentence the
  owner can actually judge. The literal command sits behind a disclosure
  for anyone who wants it; constitution.md forbids surfacing internals
  unasked, and a shell command presented as the question is a decision
  the owner has no way to make. The choices are written the same way --
  "Just this once", "Yes, for now", "Always allow this", "No" -- and the
  device only ever offers the ones actually available for that request.

  Once answered, the card collapses to one quiet line recording what the
  owner chose. A permission that vanishes without trace leaves them with
  no way to know what they agreed to.
- **A real choice is answerable by tapping, and it should be rare.**
  When the device offers a genuine choice -- between two folders it has
  already found, or between going ahead and not -- the answers appear as
  chips under the reply: at most four, the one it would pick itself first
  and visibly so, and always the option of ignoring them and typing
  instead. Tapping sends that answer as the owner's message, so it
  becomes an ordinary turn in the conversation rather than a form field.

  The restraint is the design, not a limitation of it. Chips on every
  question turn a conversation into a form, and this surface exists
  precisely because a form is the wrong shape for what the device does. A
  question with its answers pinned underneath also stops being a
  question: the owner reads the list as what the device is willing to
  hear and picks the nearest item instead of saying what is actually
  true, which costs more than the typing ever saved. So the rule is that
  the device asks plainly by default, and names the answers only when
  they *are* the question, or when the owner has visibly stalled on it.
  Where that judgement lives is `brain/chat-protocol.md`, because it is a
  question about how the device speaks.

  They appear only once the reply is complete; a half-arrived question
  flickering into buttons is worse than waiting a beat for them.
- **One input, one action.** A single quiet input bar pinned at the
  bottom. Enter sends; Shift+Enter starts a new line, so a list or a
  pasted paragraph can be written as it will be read. No toolbar, no
  attach button, no file picker, no model picker -- routing is the
  device's decision (constitution.md discloses it only on request).

  The bar itself is still one control. In the conversation pane and in
  setup it grows with the words, up to about six lines, then scrolls.
  The floating pill keeps the same keys but stays one line tall -- extra
  lines scroll inside the bar rather than growing the window, because
  the pill is meant to sit on top of other work, not become a second
  conversation pane. The naming screen stays a single line: a name is
  one line.
- **Context comes from selection, not from attaching.** Selecting a file
  places one quiet `--surface` chip directly above the input naming what
  was selected, with a single dismiss control. The next message carries
  it; sending clears it, because the question was about what they were
  looking at then and a chip left standing would suggest otherwise.

  Always exactly one chip. Selecting several collapses to a count rather
  than stacking -- a column of chips would push the conversation off
  screen, and the owner can already see their whole selection on the
  other side of the window. Dismissing the chip deselects those rows
  too: two views of one selection must never disagree.

  What reaches the assistant is the selection written from the owner's
  own files downward -- `Documents/Invoices/june.xlsx` -- and the folder
  they are currently looking at, on every turn. Never an absolute path:
  nothing above the owner's home is the device's business to mention, and
  it is not somewhere they can navigate to anyway.

  An earlier version of this rule sent bare filenames and no path at all,
  reasoning that a path in the model's context reliably comes back out in
  a reply and constitution.md forbids showing the owner one. The
  observation was right; the remedy was aimed at the wrong layer. Names
  alone leave the device unable to act on what the owner just pointed at
  -- two files called `invoice.xlsx` in different folders are one
  question it cannot answer -- so it guesses, which is the failure this
  system cares about most. The device now knows which thing is meant, and
  the rule it must follow is that it never *says* a path back. That
  belongs in constitution.md, where behaviour is specified, rather than
  being enforced by keeping the device ignorant.

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

### Voice

The owner speaks to the device and hears it answer. On a front desk or a
shop counter this is the interaction that actually fits the room --
typing is what you fall back to when you cannot speak, not the other way
round. It is a layer over the conversation, never a second conversation:
what is said aloud is written down, and what is written is what would
have been written had it been typed.

- **Mic mode is a mode the owner turns on**, from a control beside the
  composer, and it stays on until they turn it off. Not a per-message
  decision: someone standing at a counter with their hands full should
  not have to re-arm the device for every question.
- **Hold to talk, release to send.** One control, one gesture, and
  nothing happens while it is not held. A device that listens
  continuously is a device that has to be *trusted* not to; a device
  that only listens while a button is physically down is one the owner
  can see the state of. It also keeps a noisy shop from talking to it by
  accident, which is the failure that would make the whole feature get
  switched off.
- **The layer covers the input, never the transcript.** Mic mode raises
  a layer over the composer, the context chip and the model chip -- the
  bottom of the pane. Everything above it keeps streaming exactly as it
  does when typing: turns appear, the reply types itself out, and the
  owner watches the device answer while it is speaking. Turning mic mode
  off gives the composer back with the whole exchange already in place.
  Covering the conversation to show that a conversation is happening is
  the mistake this rule exists to prevent.
- **The orb carries the state**, as it does everywhere else: `listening`
  while the control is held, `thinking` between release and the first
  token, `speaking` while it talks. Rhythm only -- the layer structure
  never changes, and the orb is on screen in every voice state.
- **One line of state, in the owner's words.** "Hold to talk",
  "Listening…", "One moment…". Never a level meter, never a waveform:
  those are instruments for someone tuning a recording, and this owner
  is asking a question.
- **A spoken reply is a spoken reply.** Two sentences, no tables, no
  bullet lists, no code read aloud. Anything longer is something to
  *look* at, which is what a view is for. The device says what it built
  and offers it, rather than reading it out.
- **What is missing is spoken, not hidden.** No microphone plugged in,
  no voice service configured, no network -- each says so in the layer,
  in owner language, and the device stays usable by typing. "I can't
  hear anything plugged in" is the message; the engine's own refusal
  goes to the log. Errors here follow the same rule as everywhere else
  on this surface: never a toast, and never the system's own words.


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

This is where the owner reaches their files. A real desktop does run
underneath and it has a file manager of its own, but reaching for it
means leaving the assistant, and the whole product claim is that they
never have to. Every affordance they need must exist here, and every one
they don't need is weight they carry.

- **Either half can give way to the other.** The conversation collapses
  to its orb rail when the owner wants the files; the file view collapses
  to a rail of its own when they want the conversation. Whichever pane is
  left takes the whole width rather than sitting at its usual measure
  beside an empty gap -- a reader who asked for room and got white space
  instead has been told no politely.

  Collapsing is symmetric and reversible from the rail itself, the same
  gesture in both directions, and it is remembered across restarts: how
  the owner arranged their screen is a decision they made once. Both
  never collapse at once. The file rail carries a folder mark, not a
  chevron pointing at nothing, and the orb stays on screen either way.

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

## The canvas

The right-hand pane is **what the owner is looking at**, and their files
are only its first answer. When the device builds something to be looked
at -- a month of takings, tomorrow's appointments, a price list to hand
across a counter -- it appears here, and the conversation carries on
beside it unchanged.

Folders and Views are **tabs on the one pane**, not a third column. The
device is the assistant beside the owner's work; permanently narrowing
that work to hold a second surface costs more than it gives. Both tabs
collapse together, because the pane collapses as one thing.

- **A view is a page the device made, not a document the owner wrote.**
  That distinction has to be legible without being announced: a view
  says what question it answers and where its figures came from, in the
  same quiet `--text-secondary` line the file view uses for a count. A
  page of numbers with no stated source is the device asking to be taken
  on faith, on the one surface where faith is least appropriate --
  nobody proofreads a bar they can see.

- **The device made it; the owner owns it.** A view is a real folder in
  the owner's own filesystem, alongside their documents, not an entry in
  something only this app can open. They can copy it to a memory stick,
  send it to their accountant, or delete it, and none of that is a
  feature the device grants them.

  They must never meet its parts. In the file view a view is **one row**
  that opens the view -- not a folder to walk into and find `index.html`
  staring back. The owner of this device has never opened a text editor
  and should never learn what one is for.

- **Nothing in a view is fetched, and nothing in a view runs.** No
  network, no scripts. Static markup, the device's own stylesheets, and
  drawings computed before the page existed.

  Both halves are enforced, not merely asked for, and by different
  mechanisms because they are different holes. The frame is sandboxed,
  which stops scripts. The page is served with a policy refusing every
  outbound request, which the sandbox does not do -- and that is the one
  that matters most, because a view is markup written from the owner's
  own documents, one of which may have been written by somebody with an
  interest in the others. An image source needs no script to carry a
  figure off the device.

  It is also why a view prints correctly and opens instantly on a device
  that may never see a network.

- **A view is light by default, and the owner can turn it dark.** This is
  the one surface that departs from dark-first, and the reason is what a
  view *is*: the rest of the product is an ambient presence on a counter,
  which is what dark-first is for. A view is a document. It gets read
  closely, and it gets printed and handed to someone — and paper is
  white, so a view that is dark on screen and light on paper is two
  different documents wearing one name.

  Light is the default because it is the one that matches the printer and
  the one a front desk reads under strip lighting. Dark is one control
  away for an owner who prefers it, and the choice is remembered.

  The chrome around the view does not change. The conversation, the tab
  strip and the file view stay dark whichever way the page is set: the
  device is still the device, and the page inside it is the thing that
  behaves like paper.

- **Views look like the device, not like whatever made them.** They link
  two stylesheets the device ships and nothing else. A view written last
  month picks up a change made to either of them today. Nothing here
  invents a colour, and a view that could be mistaken for a web page
  from somewhere else is a bug.

  The first is a vendored copy of a common CSS framework, present for
  one reason: the assistant writing a view already knows its class
  names, so asking for two columns produces two columns rather than an
  invention. The second is this system's, and it exists to undo the
  first one's appearance -- it overrides that framework's own variables
  with the tokens above, so a familiar class still behaves the way the
  assistant expects while arriving in the device's colours. A framework
  page that still looks like a framework page has failed at the only job
  this layer has.

- **A view is for what will not fit in a sentence.** "How many invoices
  this month?" is answered by saying the number. Building a page for it
  is the same failure as putting three buttons under a question that
  needed none: it dresses an answer up as an event. The restraint is the
  design, not a limitation of it.

- **Printing is a first-class act.** These devices sit on counters and
  front desks, where things get printed and handed to people. A view
  that prints as a mess is a view that failed at the last step.

- **The tab appears when there is something in it.** A device on its
  first day has made nothing, and shipping every unit with an empty tab
  advertises a feature instead of offering one.

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
choice tied to the language requirement: the device needs to render
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
  selected persona, the owner-given agent name, which window mode the
  device was last in and where the pill was sitting). Deliberately NOT
  used for the OpenRouter API key -- that needs OS-keyring-backed
  storage, not a plain JSON file.
