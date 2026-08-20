---
name: owner-views
description: >
  Build a page for the owner to look at — a chart, a schedule, a table of
  figures, a price list to print. Use when an answer is too much to say
  in a sentence or two, or when the owner asks to see, chart, plot,
  summarise or print something.
version: 1.0.0
platforms: [linux]
metadata:
  hermes:
    tags: [views, charts, reports, print, summarise]
---

# Views

Some answers do not fit in a reply. Three months of takings, tomorrow's
appointments, what is in five files, a price list to hand across a
counter — read as prose these are a paragraph nobody finishes, and the
owner has to hold every number in their head while they read.

A **view** is a page you build for them. It appears beside the
conversation, and it prints.

## Build it with the command, never by hand

```
agentic-view new <name> --title "June takings" \
    --asked "how much did we take each week in June" \
    --from "Documents/june-sales.csv"
```

The name is lowercase words joined by dashes: `june-takings`. The command
prints where it put the folder.

**Do not create the folder or its files yourself.** The assistant surface
recognises one exact shape, and a folder that is almost that shape fails
silently — the files get written, everything looks like it worked, and
the owner sees nothing at all. The command is what makes the shape right.

Then edit the `index.html` it wrote. It arrives as a worked example of
every shape the page can set: replace what is below the heading and keep
the two stylesheet links exactly as they are.

**Bootstrap is on the device and already linked.** Its grid, utilities
and table classes all work -- `row`, `col`, `d-flex`, `text-end`,
`table` -- so write the layout the way you already know how rather than
inventing CSS. The second stylesheet turns the result into this device's
colours and makes it print, which is why the order of the two links
matters and why neither should be touched.

Do not link a stylesheet, font or script from the internet. It will not
load: a view is refused the network outright, so an external link is a
page that renders wrong on a device that may have no network anyway.

## One view per subject, updated

Before making anything, look at what is already there:

```
agentic-view list
```

If the owner asks about a subject you have already built a view for,
**edit that view**. Do not make a second one. `agentic-view new` will
refuse a name that exists, and that refusal is telling you to go and
edit. A folder holding four nearly identical pages is worse for the owner
than having no view at all, because now they have to work out which one
is current.

## Charts come from the numbers, not from you

Never hand-write an `<svg>`. Write the numbers to a small JSON file and
let the command draw it:

```
agentic-view chart june-takings --data /tmp/weeks.json --kind bar \
    --title "Takings by week"
```

where the data is `{"labels": [...], "values": [...]}`.

This matters more than it looks. Drawing a bar means choosing its height,
and a height chosen by you is a number you made up — it will look
authoritative and be wrong. Give the tool the figures you actually read
and it works out the geometry.

Paste the contents of the file it writes **inline** inside a `<figure>`,
rather than linking to it. Inlined, the drawing follows the page's
colours and turns to ink when the owner prints it.

## Every figure comes from something you read

`--from` names the files the numbers came from, and the surface shows
that to the owner underneath the page. Name them the way they do — the
file and the folder it is in, never a full path.

If you did not read it, it does not go in the view. A number you inferred,
estimated or remembered is the one thing a view must never contain: prose
can hedge, and a table cannot. If something is genuinely an estimate, say
so in the page in words.

Do not add your own line about where the figures came from. The surface
already shows it, and saying it twice reads as a page that does not trust
itself.

## Nothing in a view runs

No `<script>`, ever, in any form. Scripts are ignored rather than
refused, so one would look like it worked and quietly do nothing. No
images or fonts fetched from the internet either — this device may have
no network, and a broken-image icon in the middle of an answer is worse
than the text you would have written.

## Most answers are not views

A view is for what will not fit in a sentence. "How many invoices this
month?" is answered by saying the number. Building a page for it dresses
an ordinary answer up as an event, and an owner who gets a page every
time stops looking at any of them.

Build one when there are several numbers to compare, a list with more
than a few rows, something with a shape worth seeing, or something the
owner said they want to print or keep.

## When to use

- "Show me…", "chart…", "how did each week compare?"
- Anything the owner asks to print, pin up, or hand to someone.
- A summary they will want again next month — build it once, update it.
- Never for a single number, a yes or no, or a short explanation.
