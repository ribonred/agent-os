-- Knowledge store schema. Backs both onboarding.md's conversational
-- profile and the docx/xlsx/csv ingestion pipeline -- both are just
-- different sources feeding the same three tables. Deliberately flat
-- and proportionate to one small business's own records, not a general
-- graph engine (see task 007's decision).

CREATE TABLE entities (
    id SERIAL PRIMARY KEY,
    label TEXT NOT NULL UNIQUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
-- A real thing the business talks about: "business" itself, a product,
-- a service, a staff member. Facts link to an entity by id, not by
-- repeating its label as a string, so the same entity accumulates facts
-- consistently instead of drifting across near-duplicate spellings.

CREATE TABLE sources (
    id SERIAL PRIMARY KEY,
    source_type TEXT NOT NULL
        CHECK (source_type IN ('docx', 'xlsx', 'csv', 'onboarding_conversation', 'drive')),
    origin TEXT NOT NULL, -- filename, or a conversation/session identifier
    ingested_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
-- One row per ingested document or onboarding session -- provenance at
-- the document level, not per fact.

CREATE TABLE facts (
    id SERIAL PRIMARY KEY,
    entity_id INTEGER NOT NULL REFERENCES entities(id),
    field TEXT NOT NULL,
    value TEXT, -- NULL means "not yet known" -- a legitimate, expected
                -- state per onboarding.md's rules, never a reason to guess
    source_quote TEXT, -- the user's own words this was derived from, so
                        -- any fact can be traced back to what was actually
                        -- said, not just the model's interpretation of it
    source_id INTEGER REFERENCES sources(id),
    confirmed BOOLEAN NOT NULL DEFAULT false, -- confirm-back before a
                                               -- fact is trusted, per
                                               -- onboarding.md
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
-- Append-only, not upsert-on-conflict: business details change (a price
-- goes up, hours change), and onboarding.md says to treat everything the
-- user teaches as provisional. Keeping history means "what's current" is
-- a query (latest confirmed row per entity+field), not a destructive
-- overwrite that loses what used to be true.

CREATE INDEX idx_facts_entity_id ON facts(entity_id);
CREATE INDEX idx_facts_source_id ON facts(source_id);
