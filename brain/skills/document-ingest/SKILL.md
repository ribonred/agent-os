---
name: document-ingest
description: >
  The owner hands over a spreadsheet or document (xlsx, docx, csv) so
  the assistant can learn their work from it. Use when the owner shares
  a file and expects it to be read and remembered.
version: 1.0.0
platforms: [linux]
metadata:
  hermes:
    tags: [files, documents, spreadsheet, csv, docx, xlsx, ingest]
---

# Document Ingest

When the owner hands over a file — a spreadsheet of stock, a document of
prices or policies, a list of contacts — they expect the assistant to
read it and carry what matters into its knowledge of their work. This
skill is how that happens safely.

Use it when the owner **shares a file (xlsx, docx, or csv)** and expects
the assistant to learn from it.

## What the assistant can read today

- **Spreadsheets** (xlsx)
- **Documents** (docx)
- **Comma-separated lists** (csv)

**Not yet: PDF or scanned images.** If the owner hands over a scan or a
PDF, say plainly that the assistant cannot read that kind of file yet
rather than pretending to. Do not attempt to parse what is not supported.

## Rules

- **Only extract what is written.** Read the words and numbers actually
  in the file. Do not infer what was not written, fill gaps with guesses,
  or invent meaning.
- **Confirm before you remember.** Reflect back, in plain language, what
  you took from the file and get an explicit yes before committing it as
  something you will remember.
- **Turn it into ordinary records.** What you learn from the file is
  stored as the same kind of confirmed, traced facts as anything the
  owner tells you in conversation — it feeds the same memory of their
  work.

## How to do it

1. Accept the file the owner shared.
2. Read it with the right tool for its kind (spreadsheet, document, or
   comma-separated list).
3. Pull out only what the file actually says.
4. Show the owner a plain-language summary of what you found.
5. Get explicit confirmation before committing it as remembered facts.

## When to use

- The owner drops in a spreadsheet of their stock, inventory, or records.
- The owner shares a document with prices, policies, or instructions.
- The owner shares a list and wants the assistant to know the contents.