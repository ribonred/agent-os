# Onboarding

Governs the first real conversation with a new user: how you learn who
they are and what they need, before any domain knowledge exists yet. This
is the highest-risk conversation you will ever have — whatever you get
wrong here becomes the foundation everything after it is built on, and it
often runs on the smallest model available on the device. Treat accuracy
here as more important than speed or coverage.

## Mandatory device setup, before the guided conversation

Two choices happen first, as direct UI selection in the Tauri shell —
not as LLM-generated conversational questions. They're device-level
setup, not business-context discovery, and they're deterministic enough
that asking a model to phrase them adds risk (translation/interpretation
drift) for no benefit.

**Language.** Presented as a plain list, no scrolling through a long
global list — scope is Asia-first plus English, per the actual market
this device ships into:

- Bahasa Indonesia (primary market, listed first, not alphabetically)
- English
- Mandarin Chinese (Simplified)
- Japanese
- Korean
- Vietnamese
- Thai
- Malay
- Filipino (Tagalog)
- Hindi

Whatever's selected here is the language for everything after, including
the business-onboarding conversation below.

**Name.** The device ships nameless — the assistant has no built-in
name (constitution.md), and the owner christens it here. A direct
free-text input, not an LLM question, for the same determinism reason
as language: this is the one answer that must survive verbatim,
in any script the owner types. The owner's choice is final until they
change it. Naming is the moment the device stops being "a box" and
becomes *theirs* — the screen should feel like that, not like a form
field (see design/DESIGN.md, "Naming screen").

Fresh devices use the **Balanced** voice from constitution.md during setup.
How the owner prefers to be addressed — formality, detail, and pace — is one
of the five unknowns learned in the guided conversation, not another menu.
Devices upgraded from an earlier release may already carry a persona choice;
keep applying it rather than forcing the owner through setup again.

### How the two choices reach the agent

The selections do not edit the soul file. constitution.md ships
verbatim as the agent runtime's identity slot (SOUL.md) and stays
device-generic; the shell applies the owner's choices as a small
overlay appended on top of it for every conversation turn. The overlay
shifts language and carries the name — it does not
change any Core Behavior rule (confirm-before-consequential, never
fabricate, etc.). A legacy persona, when present, only shifts register.

The canonical overlay texts live here; the shell
(`ui/src-tauri/src/agent.rs`) mirrors them verbatim, same contract as
the option lists above. Change this file first, then the mirror. These
three stay in the shell because each is a template with a runtime value
substituted into it — the owner's chosen name, their language — rather
than fixed prose.

Onboarding progress is **shell state**, not model memory. The checklist,
answers, and completion flag live in the setup store (`onboardingState`);
Hermes only phrases the current open step. The product contract for that
driver is `brain/onboarding-protocol.md`. Opening/resume shell directives
are `brain/onboarding-start.md` and `brain/onboarding-resume.md`.

- **Identity** (only when a name is set): "Your owner has named you
  {name}. That is your name — use it naturally when you introduce
  yourself or when asked, and never claim a different name or
  identity."
- **Language**: "Reply in {language} by default; follow the user's
  lead if they switch languages."
- **Balanced / no stored persona**: no overlay — it *is* the baseline
  defined in constitution.md's Tone section.
- **Warm & Patient**: "Voice: be warm and patient. Offer more
  encouragement and more explanation per answer, at a slower pace.
  Never rush the user or assume familiarity with technology."
- **Straight & Efficient**: "Voice: be brief and efficient. Minimal
  small talk, lead with the answer, keep sentences short. The user is
  busy — every extra sentence costs them time."
- **Formal & Precise**: "Voice: keep a measured, professional
  register. Precise wording, no casual phrasing, no exclamation
  marks. Warmth shows through care and accuracy, not informality."

## The first question is what to call them

The owner names the device on the naming screen. The device then opens
the conversation by introducing itself and asking nothing back about who
it is talking to — it goes on to learn their role, their work and their
vocabulary, and never learns their name. That is an odd thing for a
personal assistant not to know, and odder still when they have just
given it one of its own.

So the opening exchange is a mutual introduction: the device says what it
is called and asks what to call them. This comes before the unknowns
below, and it happens **once**. The shell locks `owner_name` after the
first answer and will not make that step current again — never re-asked,
never "confirmed" by asking again, never re-opened by a fresh
self-introduction.

What is wanted is what they want to be **called** — a first name, a
nickname, whatever they answer with. Not a legal name, and not a title
unless they offer one. Record it exactly as given: never correct its
spelling, expand it, translate it, or convert it to another script. The
same rule the device's own name gets, for the same reason — a name that
comes back altered reads as not having been listened to.

It is a fair question to decline, and some devices have no single answer:
a clinic front desk or a shop counter may be several people across a day.
If they decline, or say it is shared, take that as the answer, note it,
and move on. A device that asks again is worse than one that says "you".

Never invent a name, and never infer one from an email address, a
business name, or anything else already on the device.
records this as the `owner_name` step. It earns the slot: the
answer is used in every conversation afterwards, which is more than most
of the others can say.

## Shell-owned checklist; Hermes only phrases

The guided conversation is a fixed checklist owned by the shell, not a
free-form interview the model steers:

1. `owner_name` — what to call them
2. `role` — who they are and what this device is for
3. `needs` — concrete tasks they want help with
4. `vocabulary` — day-to-day terms and important entities
5. `boundaries` — sensitivities / off-limits
6. `communication` — formality, detail, pace
7. `confirm` — summarize known facts; owner accepts

Hermes receives only the **current** open step plus locked facts already
answered. It phrases one question for that step and stops. It does not
choose the next step, mark steps done, run tools, check services, search
sessions, or write memory during setup.

**One question per reply, and the reply ends at the question mark.**

**Prefer questions the owner can answer with yes or no** when the step
allows it. Use an open question only where yes/no cannot get there
(vocabulary is the usual case).

`chat-protocol.md` still applies so a stalled owner can get tappable
likely answers — not so every step becomes a form.

A short, thin, declined, or "not sure" answer still completes the current
step in shell state. Unresolved is valid; circling is not.

## The shell learns the device silently

Before the greeting, the shell runs live Postgres and Redis checks and
writes successful facts to Hermes `MEMORY.md`. Hermes is not asked to
discover the device during onboarding. Failures stay silent and do not
block learning about the owner.

Postgres is durable relational storage. Redis is ephemeral cache, queue,
and session state. Never put the durable owner profile in Redis.

## Bounds

There is one shell turn per open step, plus confirm. No 5–15 free
discovery budget. The interview ends when the owner accepts the confirm
summary (or the shell has nothing left to ask and confirm runs with
unresolved fields marked unresolved)ark that unknown done, and don't
  force them back to the original one.

## Why small models hallucinate here, and how you avoid it

Onboarding often runs on the smallest model on the device. Small models
are most likely to fail exactly here: filling gaps with plausible-sounding
inferences, drifting during paraphrase, or losing track across turns.
Concrete safeguards, all mandatory:

**Never infer what wasn't said.** If the user says "I run a skincare
shop," that's it — you don't know if they do facials, sell products,
have staff, or take walk-ins until they tell you. An unresolved unknown
is a valid, expected outcome. A guessed one is not.

**Confirm before you commit anything to memory.** After extracting what
you think you learned from an answer, reflect it back in plain language
("So it's just you, mainly facials and skin consultations — is that
right?") and get explicit confirmation before writing it into persistent
context. This is the single most important safeguard: it catches a
misunderstanding the moment it happens, while it's cheap to fix, instead
of after it's already shaped every later interaction.

**Extract into fixed fields, not free paraphrase.** Don't let the model
freely summarize a long answer into prose — map it into the specific
unknown it resolves. Free paraphrase is where small models drift furthest
from what was actually said. Every field can hold "not yet known"; that
is always a legitimate value, never a reason to guess instead.

**Store the user's own words, not just your interpretation of them.**
Keep what they actually said as the source of truth alongside any
structured field you derived from it, so a later review can trace any
fact back to the specific statement that produced it.

**Cross-check with a stronger model when one is reachable.** If the
device has connectivity and the routing policy allows it, running the
final structured extraction past a larger model catches mistakes a small
local model made — this is exactly the kind of task the online/offline
routing policy should weigh toward "use the bigger model" for, even on a
device that stays offline for everything else, because getting onboarding
wrong is costly and it's a one-time, low-frequency task. When offline,
the confirm-before-commit step above is the safety net instead.

## Handoff

When every discovery step is done, the shell moves to `confirm`. Hermes
summarizes the shell's known facts (including what to call them) and asks
for acceptance with yes/no options. Corrections stay on confirm until the
owner accepts.

After explicit acceptance, the **shell**:

1. writes `USER.md` from the structured checklist
2. sets `onboardingComplete`
3. streams a fixed closing line (language-aware; includes what to call
   them and the assistant name) — not a free-form Hermes goodbye
4. drops the onboarding session

Unresolved fields stay "not yet known". Do not duplicate the profile in
Postgres. Setup is complete when the write succeeds and
`onboardingComplete` is set — never when the model merely says it is done.

Start normal conversation in a fresh Hermes session after the write so the
new `USER.md` and `MEMORY.md` snapshots are present in the system prompt.
