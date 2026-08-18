---
name: product-ux
description: >
  Design or critique this product's screens using design/DESIGN.md.
  Use for the orb, setup, conversation pane, file shelf, pill window,
  owner-facing copy, empty/error/permission states, motion, or when a
  change might look like a generic chatbot or SaaS dashboard.
---

# Product UX

Adapted from Anthropic's public `frontend-design` skill (restraint,
anti-template, copy is design) and applied to *this* appliance — not
a marketing site and not "make it look like Linear."

Source of truth: `design/DESIGN.md`. Change the doc, then
`ui/app/assets/css/main.css` and components. Never invent hex in a
Vue file.

## When to Use

- New or changed owner-visible UI
- "Does this feel right", empty states, errors, permissions
- Orb, naming, onboarding, two-pane shell, pill
- Copy the owner will read

Don't use for: standalone HTML mockups, cloning Stripe/Linear,
Google DESIGN.md token files. This product already has a visual
identity.

## Surface (name it before tokens)

This is mostly **Command / Inspect** (talk to the device) sitting
beside **Explore** (their files). Setup screens are **Configure**.

It is not Decide/Learn. Do not add a hero, three feature cards, or
a product tour.

## Locked identity (do not restyle)

From DESIGN.md, already in `:root`:

```
--bg #0B0E14    --surface #12151C    --surface-raised #1A1E27
--text-primary #E8EAED    --text-secondary #8B93A1
--accent #3DDCFF    --accent-warm #F6B673
--success #4ADE80    --danger #FF6B6B
```

Two accents only. Orb-only extras (`--orb-violet`, `--orb-deep`)
never appear on buttons, text, or borders.

The orb is the other party. No assistant avatar, name badge, or
assistant-side bubble. Assistant text is bare `--text-primary` on
the canvas. User messages are quiet right-aligned `--surface` pills.

A screen with no orb reads as powered off. Keep it on every state,
including the pill.

## Conversation rules (easy to break)

- Persistent left pane, not a chat route you navigate away from.
  Replies keep streaming while the owner browses files (`shell.vue`
  is a layout so it does not remount).
- Streaming = tokens appending. No spinner, no "typing…". Orb state
  is the only status: idle / thinking / speaking.
- Work the device did: one `--text-secondary` line in the owner's
  language ("Looked through your files."), collapsed, never a
  progress bar or log.
- Permission lives in the conversation, not a modal. Plain language
  first; the shell command behind a disclosure. Choices like "Just
  this once" / "No" — only ones that are real for that request.
  Afterward, collapse to one line of what they chose.
- Answer chips: at most four, recommended first, after the reply
  has finished. Typing always still works.
- One input, send on Enter. No toolbar, attach button, file picker,
  or model picker.
- Context is a selection chip above the input, cleared on send.

## Setup

Language (deterministic list, Indonesia first) → name (free text,
max 60, no suggested names) → guided conversation. No persona
screen on a fresh device.

Naming is ceremony, not a form. Onboarding is still setup: orb at
setup size, no "Ask me anything", no back control. Profile review
is one spoken summary, not five cards.

## Window

Full = maximized undecorated two-pane (system bar still reachable).
Minimized = one floating pill, same conversation, grows only while
there is something to read. Transition is a resize, not a new
window.

## Copy

Write from the owner's side of the screen. They have used a phone,
not Ubuntu.

- Name the outcome, not the mechanism. "I couldn't reach my
  assistant" not "connection refused :8642".
- Errors go through `ui/app/lib/agentErrors.ts`.
- Failure and emptiness give a next step, no apology theater.
- Active voice, sentence case, no filler. Same verb on the button
  and the confirmation.

## Anti-slop (this product)

Score before fixing. These tells are failures here:

1. Chat-app skin (avatars, bubbles both sides, composer toolbar)
2. SaaS dashboard (sidebar nav, metric cards, "Insights")
3. Extra accent colors or orb-violet on chrome
4. Hero / feature-tile grid on a setup or home screen
5. Spinner that duplicates the orb
6. Modal permission dialog
7. Inter / "AI font" swap away from the system stack (CJK must
   keep working; fonts come from the image)
8. Glassmorphism, gradient wash, emoji chrome
9. Showing apt, systemd, paths, or model ids
10. Invented claims or fake metrics

Compositional tells (1, 2, 4, 10) need a re-layout, not a recolor.

Motion: atmosphere breathe, orb state changes, greeting fade.
Respect `prefers-reduced-motion`. No looping decoration.

Hit targets ≥ 44px. Keyboard focus visible. Contrast against
`--bg` / `--surface` for `--text-primary` and `--accent`.

## Procedure

1. Read the relevant DESIGN.md section and the existing Vue file.
2. Name the surface. List which locked rules apply.
3. Change DESIGN.md first if the visual system is moving.
4. Implement with existing tokens and components
   (`PresenceOrb`, `ConversationPane`, `ContextChip`, …).
5. Verify visually per `agentic-ui` (screenshot, not typecheck).
6. Run the anti-slop list. Cut one accessory.

## Gotchas

- System font stack is required. Several setup languages are
  non-Latin; a bundled webfont would miss them.
- Do not put the owner's chosen name in the header. It lives in
  the agent's voice only.
- `popular-web-designs` / generic `claude-design` one-off HTML
  will fight this identity. Stay in `ui/`.
