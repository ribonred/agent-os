---
id: "007"
title: "Decide graphify port strategy for docx/xlsx/csv/Drive ingestion into agent knowledge space"
status: completed
priority: low
effort: medium
phase: agent-core
dependencies: []
tags: ["graphify", "knowledge"]
created_at: 2026-07-13
completed_at: 2026-07-13
---

# Decide graphify port strategy for docx/xlsx/csv/Drive ingestion into agent knowledge space

## Objective

Decide how document ingestion (docx/xlsx/csv/Drive) into the agent's
knowledge space relates to graphify, given graphify today is a standalone
package (`graphifyy`) with pluggable extraction backends -- but checked its
actual backend list and confirmed it is cloud-only (gemini/kimi/openai/
deepseek/claude-cli), with no local/Ollama option. That's a hard conflict
with offline capability, so a full port was never viable as-is.

## Decision

- Do not port graphify's full pipeline (community detection, Neo4j export,
  Obsidian vault) -- built for developer-scale codebase graphing, more
  machinery than a small business's own documents need.
- docx/xlsx/csv parsing is built directly in agentic-os, independent of
  graphify -- deterministic format parsing needs no LLM and always works
  offline.
- Extraction reuses the onboarding safeguards (fixed-field extraction,
  confirm-back before committing, "not yet known" as a valid value,
  cross-check with a bigger/cloud model when connectivity allows) rather
  than graphify's pipeline.
- Storage rides on the Postgres already in the tool registry (entities/
  facts/source-quote tables) rather than adding Neo4j as a new tool-registry
  entry -- disproportionate resource cost for this scale of knowledge.
- Google Drive ingestion is inherently online-only (OAuth + API), gated on
  connectivity regardless of extraction path.
- Separately (outside this repo): add a local/Ollama backend to graphify
  itself, upstream in the graphifyy project. Not part of agentic-os's
  codebase or task list -- tracked in that project instead. Once it exists,
  capable hardware (DGX Spark tier, or any online-connected device) can
  optionally run graphify's richer pipeline instead of the lightweight
  extractor, but agentic-os's own ingestion does not depend on it existing.

## Tasks

- [x] Confirm graphify's actual backend support (read the skill/package,
      don't assume)
- [x] Decide storage target (Postgres vs. a dedicated graph DB)
- [x] Decide whether the local/Ollama backend work belongs in this repo

## Acceptance Criteria

- Ingestion pipeline design (follow-up task) does not assume graphify is
  available or reachable -- offline ingestion must work with zero
  dependency on graphify or any cloud API
