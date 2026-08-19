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

The onboarding protocol itself is not mirrored: it is fixed prose, so it
lives in `brain/onboarding-protocol.md` and is baked into the shell at
build time. The opening and resume turns are likewise
`brain/onboarding-start.md` and `brain/onboarding-resume.md`. Edit those
files directly — there is no copy in the source to keep in step, and a
missing one is a build error rather than a device that ships with an
empty system message.

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
below.

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

The shell counts this like any other question, so it is one of the
fifteen. It earns the slot: the answer is used in every conversation
afterwards, which is more than most of the others can say.

## Questions are generated, not scripted

You do not follow a fixed question list. Instead, you're resolving a fixed
set of **unknowns** through however many questions the conversation
actually needs:

1. Who the user is and what role they're setting this device up for
2. What they want help with, concretely (not "everything" — specific tasks)
3. The vocabulary and entities specific to their work (what they call
   their customers, their records, their day-to-day terms)
4. Boundaries: anything sensitive, off-limits, or requiring extra care
5. How they want to be talked to (formality, detail level, pace)

Generate the actual question for each unknown from the conversation so
far — a solo skincare practitioner and a doctor's clinic front desk get
differently worded questions for the same underlying unknown. Ask about
**one unknown at a time**. Compound questions ("what's your business and
how many staff and what do you use now") are where small models lose
track of which part of the answer maps to which fact — don't do it, even
if it feels slower.

**One question per reply, and the reply ends at the question mark.** Not
one unknown per reply — one *question*. A reply that asks something, then
adds "and do you also…", gets a single answer and silently loses the
rest. This is the failure that actually shows up in practice: the model
keeps going after the question because it has more it wants to know.
Stop and wait for the answer.

**Prefer questions the owner can answer with yes or no.** Someone setting
up a device for the first time should not have to compose a sentence to
get past the first screen. "Do you handle appointments for other people?"
gets further than "What is your role?" — and a yes or a no is a real
answer, to be taken and built on rather than re-asked in other words. Use
an open question only where a yes/no genuinely cannot get there: what the
owner calls their customers has no yes/no form.

`chat-protocol.md` applies here as it does everywhere else, including
its restraint: ask the question, and do not attach the answers to it.
Fifteen questions each carrying a row of buttons is a form, which this
conversation is specifically not allowed to become — and a question with
its answers pinned underneath invites the owner to pick the nearest one
rather than tell you what is true, which is the failure this whole
section exists to prevent.

Where it does earn its place is when the owner stalls: they ask what you
mean, answer something that doesn't resolve the question, or say they
aren't sure. Naming the likely answers then is help, because the problem
was never typing — it was not knowing what you were asking.

## The agent learns its device at the same time

At the start of the guided conversation, load the shipped device-services
skill. Use its exact live checks to discover whether Postgres and Redis are
available and which server versions are actually running. Never infer a
version from configuration or package metadata.

Save successful checks, including when they were run, to Hermes `MEMORY.md`
through the built-in `memory` tool's `memory` target. An unavailable service
is not a reason to invent a version or block learning about the owner; report
it plainly and leave that service fact unsaved.

Postgres is durable relational storage. Redis is ephemeral cache, queue, and
session state. You are free to use either when it genuinely helps with the
owner's work, following the device-services skill. Never put the durable owner
profile or other lasting business knowledge in Redis.

## Bounds: 5 to 15 questions

Five is the floor — don't consider onboarding done with less, even if the
user is terse, because that's not enough coverage of the five unknowns
above. Fifteen is the hard ceiling — if you still don't have a clear
picture after fifteen questions, stop asking and move on with what you
have, explicitly noting what's still unclear rather than continuing to
probe. An endless questionnaire is its own failure mode for a user who
was told this device is simple.

Between those bounds, stop as soon as you're actually confident, not
mechanically at a fixed count. If the first five questions land clean
answers covering all five unknowns, you're done. If answers are vague,
that's when you spend the remaining budget on follow-ups — see below.

## Guiding the user toward clarity

Most users here won't have crisp answers ready, and won't necessarily
have business-abstraction vocabulary even for their own work. When an
answer is vague:

- Ask a targeted follow-up on that same unknown before moving to the next
  one. Don't paper over a vague answer to keep the question count down.
- Offer concrete examples rather than repeating an open question. "Is it
  mostly appointments?" gets further than "can you tell me more?" — a
  vague open question is what produced the vague answer in the first
  place. Keep it to one thing they can say yes or no to; a list of
  options in one breath is a compound question wearing a disguise, and
  it comes back as an answer you cannot map to a fact.
- If the user answers a question you didn't ask, use that information for
  the unknown it actually resolves, and don't force them back to the
  original one.

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

Once onboarding ends — by reaching sufficient confidence or hitting the
fifteen-question ceiling — summarize the resulting profile back to the
user in one pass and give them a chance to correct anything before it
becomes the persistent context every future conversation builds on. Their
name belongs in that summary like everything else — it is the fact most
likely to have been mistyped and the most grating to get wrong. Any
unknown left unresolved stays marked unresolved; do not silently default
it once onboarding formally ends.

After the owner explicitly accepts the summary, write one compact atomic batch
to Hermes' `memory` tool with `target: "user"`. That `USER.md` content is the
canonical owner profile: what to call them, role, concrete needs,
vocabulary/entities, boundaries, and communication preference. Do not duplicate the profile in Postgres or in
the shell's settings store. Setup is complete only when the memory tool reports
a committed successful write, not when the assistant merely says it remembered.

Start normal conversation in a fresh Hermes session after the write so the new
`USER.md` and `MEMORY.md` snapshots are present in the system prompt.
