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
- [x] Postgres schema for the knowledge store: entities/facts/source-quote
      tables, proportionate to one small business's documents, not a full
      graph engine
      -- agent-core/knowledge-store/schema.sql. Normalized 3-table design:
      entities (real identity, not repeated strings), sources (document-
      level provenance), facts (entity_id + field + value, nullable value
      = "not yet known", source_quote for traceability, confirmed bool).
      Append-only by design, not upsert -- business details change, and
      onboarding.md says treat everything as provisional; "current value"
      is a query (latest confirmed row), not a destructive overwrite.
      Validated for real: this box only has psql client tools installed,
      no server (confirmed via dpkg, not assumed) and no running cluster
      (pg_lsclusters empty). Pulled a real postgresql_18 via `nix shell`
      instead of sudo apt-installing a system package, ran a throwaway
      cluster in /tmp, applied the schema, and ran a realistic scenario
      (onboarding fact + docx price fact + a price change) confirming
      entity/source joins, NULL-as-unresolved querying, and history
      preservation all work -- then tore the cluster down, no system
      state left behind.
- [~] Fixed-field extraction step reusing onboarding.md's safeguards
      (structured fields incl. "not yet known", confirm-back before
      commit, routed through hw-probe's tier/online output to decide
      local vs cloud model for the extraction call)
      -- extraction.py built and split the same way hw-probe split GPU
      detection: pure logic (build_extraction_request,
      parse_extraction_response) fully unit-tested with fabricated model
      responses (7 tests: well-formed, null-value-is-valid, empty facts,
      malformed JSON, missing keys -- all explicit errors via
      ExtractionParseError, never a silent empty result). Uses Ollama's
      schema-constrained `format` field (verified against current docs,
      not assumed) so the model structurally cannot return free prose --
      this is the concrete mechanism behind onboarding.md's "fixed
      fields, not free paraphrase" rule.
      Cloud provider decided: OpenRouter, not a direct Claude/OpenAI/
      Gemini integration -- one API key, one client, and OpenRouter
      itself carries Hermes (nousresearch/hermes-4-70b, our default),
      giving behavioral consistency with the local Ollama tier instead of
      a personality shift when routing flips. Checked OpenRouter's terms
      before committing: commercial use unrestricted, pass-through
      provider pricing plus a flat 5.5% credit fee, no found restriction
      on embedding in a third-party device. Real tradeoff disclosed, not
      hidden: OpenRouter sits in the request path as a third party (their
      own provider-logging policy applies), and an OpenRouter-side outage
      takes down the whole cloud tier even if the underlying provider is
      fine -- narrower blast radius than a direct integration's failure
      mode, but not zero.
      build_openrouter_extraction_request() built alongside the Ollama
      version, sharing the same EXTRACTION_SCHEMA (verified identical via
      a dedicated test) and the same prompt text -- confirmed against
      OpenRouter's actual /api/v1/chat/completions + response_format.
      json_schema contract, not assumed. call_openrouter_extract() is
      unvalidated for the same reason as call_ollama_extract: no API key
      exists anywhere in this project yet (that's the GUI piece, deferred
      -- see below).
      Routing wired: agent-core/ingest/routing.py shells out to the real
      compiled hw-probe binary (two runtimes, one source of truth --
      routing logic is not reimplemented in Python), parses its JSON, and
      dispatches to call_ollama_extract or call_openrouter_extract based
      on default_routing. First real cross-language integration point in
      the project, validated end-to-end: a dedicated test actually runs
      the compiled Rust binary and confirms the JSON round-trips into a
      resolvable backend choice, not fabricated input. Dispatch logic
      itself tested separately with controlled/monkeypatched probe
      results (local->ollama, cloud->openrouter, cloud-without-api-key
      raises explicitly rather than silently picking a default). 9 new
      tests, 28 total in the ingest package, all passing.
      Live validation DONE for both backends (the previously recorded
      gap, closed once tasks 016+008 produced a real key and a real
      routing owner). Local: full pipeline through the real compiled
      hw-probe binary -> default_routing=local -> live Ollama hermes3:3b
      with schema-constrained decoding -> 3 CandidateFacts with source
      quotes from a fabricated skincare-shop menu; the deliberately
      ambiguous line ("walk-ins maybe accepted") was correctly not
      extracted. Cloud: call_openrouter_extract against live OpenRouter
      hermes-4-70b -> 5 facts (caught the durations the 3b local model
      missed, correct entity attribution, same ambiguity correctly
      omitted). Model-ID drift found and fixed during this: the
      orchestrator daemon's default cloud model said
      hermes-3-llama-3.1-70b while this task had deliberately chosen
      hermes-4-70b -- both verified live against OpenRouter's /models,
      hermes-4-70b wins (the documented decision), orchestrator default
      updated, lockstep comments added in both runtimes.
      Still NOT done: confirm-back-before-commit flow doesn't exist, and
      extracted facts are not yet written into the knowledge-store
      schema (extraction ends at CandidateFact objects). Design note for
      the next step: now that the orchestrator daemon owns routing and
      key resolution, ingest should likely call the daemon (e.g. a
      POST /extract endpoint) instead of shelling to hw-probe and
      holding an API key itself -- one routing implementation, and the
      key never enters the Python runtime. Decide deliberately before
      building the confirm-back flow on the current shape.
- [x] Google Drive ingestion -- decided closed, not built. OAuth + Drive
      sync is a solved problem with mature existing tools (rclone, the
      gdrive CLI) -- no reason to hand-roll custom OAuth/API code for it.
      When Drive support is actually needed, wire one of those in rather
      than building bespoke integration; whatever gets pulled down still
      lands as a normal file and flows through the same docx/xlsx/csv
      parsers already built, no new parsing path required. Still
      correctly gated on connectivity by nature (can't sync Drive
      offline), that part was never in question.

## Acceptance Criteria

- A real docx, xlsx, and csv file can each be parsed into structured text
  without any network access
- Extraction never silently guesses -- unresolved fields are stored as
  unresolved, matching onboarding.md's rules
- Nothing in the offline path imports or shells out to graphify or any
  cloud API
