---
id: "012"
title: "Build local ingestion pipeline: docx/xlsx/csv parsing + fixed-field extraction into Postgres knowledge store"
status: in-progress
priority: medium
effort: large
phase: agent-core
dependencies: ["003", "011"]
tags: ["knowledge", "ingestion", "postgres"]
created_at: 2026-07-13
---

# Build local ingestion pipeline: docx/xlsx/csv parsing + fixed-field extraction into Postgres knowledge store

## Objective

Let a non-technical user teach the device their business by handing it
documents (docx/xlsx/csv, later Drive) instead of typing everything into
onboarding. Per task 007's decision: no dependency on graphify or any
cloud API for the offline path, storage in the Postgres already in the
tool registry (not a dedicated graph DB), and extraction reuses the same
fixed-field + confirm-back safeguards as onboarding.md rather than free
paraphrase.

Language choice, decided here rather than defaulting to Rust for
consistency with hw-probe: Python via uv, not Rust. Document parsing
(docx/xlsx/csv) is a data-wrangling problem where Python's libraries
(python-docx, openpyxl, csv) are far more mature than Rust's equivalents
-- red-mindset's own guidance is "complex domain, use a library," and uv
is already a first-class, mandated part of this project's toolchain, not
a new dependency being introduced. hw-probe was Rust because it needed
low-level sysfs access; this is a different kind of problem and doesn't
need to match that choice for its own sake.

## Tasks

- [x] Scaffold a uv-managed Python project at agent-core/ingest/
- [x] CSV parsing (stdlib, no dependency) -- smallest piece, validate
      against a real fabricated test file before anything else
      -- parsers/csv_parser.py, 3 pytest cases against real fabricated
      CSV files (header+rows, header-only/empty, values stay strings not
      coerced). All passing.
- [x] xlsx parsing (openpyxl) -- parsers/xlsx_parser.py, 3 pytest cases
      including a real write-then-read round trip. Deliberately keeps
      openpyxl's native types (int/float) unlike parse_csv's all-strings
      -- documented as an intentional difference, not an inconsistency.
- [x] docx parsing (python-docx) -- parsers/docx_parser.py, 3 pytest
      cases. Paragraphs and tables both extracted; tables stay as raw
      rows of strings (not header-keyed dicts like csv/xlsx) since a
      docx table isn't reliably a data export the way a spreadsheet is --
      assuming row 0 is a header would often be wrong.
- [ ] Postgres schema for the knowledge store: entities/facts/source-quote
      tables, proportionate to one small business's documents, not a full
      graph engine
- [ ] Fixed-field extraction step reusing onboarding.md's safeguards
      (structured fields incl. "not yet known", confirm-back before
      commit, routed through hw-probe's tier/online output to decide
      local vs cloud model for the extraction call)
- [ ] Google Drive ingestion -- explicitly gated on connectivity, separate
      from the always-offline docx/xlsx/csv path

## Acceptance Criteria

- A real docx, xlsx, and csv file can each be parsed into structured text
  without any network access
- Extraction never silently guesses -- unresolved fields are stored as
  unresolved, matching onboarding.md's rules
- Nothing in the offline path imports or shells out to graphify or any
  cloud API
