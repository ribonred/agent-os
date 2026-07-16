"""Turns parsed document text into candidate facts via a local LLM call.

Split deliberately into pure logic (testable here, no LLM needed) and a
thin I/O function (not validated in this sandbox -- see call_ollama_extract).
Mirrors how hw-probe split GPU/NPU detection from tier classification.
"""

import json
import urllib.request
from dataclasses import dataclass

# Schema-constrained decoding: Ollama's /api/generate accepts a JSON
# schema in `format` and restricts token sampling so the output is
# guaranteed to match it. This is the actual mechanism behind
# onboarding.md's "extract into fixed fields, not free paraphrase" rule --
# the model structurally cannot return prose instead of this shape.
EXTRACTION_SCHEMA = {
    "type": "object",
    "properties": {
        "facts": {
            "type": "array",
            "items": {
                "type": "object",
                "properties": {
                    "entity": {"type": "string"},
                    "field": {"type": "string"},
                    "value": {"type": ["string", "null"]},
                    "source_quote": {"type": ["string", "null"]},
                },
                "required": ["entity", "field", "value", "source_quote"],
            },
        }
    },
    "required": ["facts"],
}


@dataclass
class CandidateFact:
    entity: str
    field: str
    value: str | None  # None means "not yet known" -- see schema.sql
    source_quote: str | None


class ExtractionParseError(Exception):
    """The model's response didn't match EXTRACTION_SCHEMA.

    Deliberately a hard error, not a silent empty list -- a malformed
    response means something is wrong (bad model, bad prompt, schema
    drift) and swallowing it would hide that, not handle it gracefully.
    """


def _extraction_prompt(text: str) -> str:
    """Shared between both backends -- the instruction is provider-
    agnostic, only how it's wrapped into a request body differs."""
    return (
        "Extract factual claims from the following business document text. "
        "Only extract what is explicitly stated -- never infer or guess. "
        "If a claim is ambiguous, omit it rather than guess its meaning.\n\n"
        f"TEXT:\n{text}"
    )


def build_extraction_request(text: str, model: str) -> dict:
    """Pure -- constructs the Ollama /api/generate request payload. No
    I/O, no network."""
    return {
        "model": model,
        "prompt": _extraction_prompt(text),
        "stream": False,
        "format": EXTRACTION_SCHEMA,
    }


def build_openrouter_extraction_request(text: str, model: str) -> dict:
    """Pure -- constructs the OpenRouter /api/v1/chat/completions request
    payload. Different shape from Ollama's (OpenAI-compatible messages
    array + response_format.json_schema instead of prompt + format), but
    the same EXTRACTION_SCHEMA and the same underlying instruction --
    parse_extraction_response works on either backend's output once the
    response envelope is unwrapped, since the schema is provider-agnostic.
    """
    return {
        "model": model,
        "messages": [{"role": "user", "content": _extraction_prompt(text)}],
        "response_format": {
            "type": "json_schema",
            "json_schema": {
                "name": "extraction",
                "strict": True,
                "schema": EXTRACTION_SCHEMA,
            },
        },
    }


def parse_extraction_response(response_text: str) -> list[CandidateFact]:
    """Pure -- parses the model's response string (the value of Ollama's
    top-level "response" field) into CandidateFact objects. No I/O.
    """
    try:
        parsed = json.loads(response_text)
    except json.JSONDecodeError as e:
        raise ExtractionParseError(f"response was not valid JSON: {e}") from e

    if "facts" not in parsed or not isinstance(parsed["facts"], list):
        raise ExtractionParseError("response JSON missing a 'facts' array")

    facts = []
    for i, item in enumerate(parsed["facts"]):
        for required_key in ("entity", "field", "value", "source_quote"):
            if required_key not in item:
                raise ExtractionParseError(
                    f"fact at index {i} missing required key '{required_key}'"
                )
        facts.append(
            CandidateFact(
                entity=item["entity"],
                field=item["field"],
                value=item["value"],
                source_quote=item["source_quote"],
            )
        )
    return facts


def call_ollama_extract(
    text: str, model: str, host: str = "http://127.0.0.1:11434"
) -> list[CandidateFact]:
    """The actual I/O -- NOT validated in this sandbox. Ollama isn't
    installed or running here (confirmed: `which ollama` found nothing,
    curl to 127.0.0.1:11434 got no response), and pulling a real model
    just to test this one HTTP call would mean a multi-GB download for a
    throwaway sandbox check -- disproportionate to what's being verified.
    Written correctly against Ollama's documented /api/generate contract
    (format: JSON schema for constrained decoding), but must be exercised
    against a real running Ollama + Hermes before this is trusted.
    """
    request_body = build_extraction_request(text, model)
    req = urllib.request.Request(
        f"{host}/api/generate",
        data=json.dumps(request_body).encode("utf-8"),
        headers={"Content-Type": "application/json"},
        method="POST",
    )
    with urllib.request.urlopen(req, timeout=120) as resp:
        body = json.loads(resp.read().decode("utf-8"))
    return parse_extraction_response(body["response"])


# Default per the product decision: route cloud escalations to Hermes via
# OpenRouter, not a frontier model -- same model family as the local
# Ollama tier (consistent tool-calling/instruction-following behavior
# across the routing switch), and meaningfully cheaper than Claude/GPT/
# Gemini for what should mostly be routine escalations from weak
# hardware, not the hardest possible requests.
# Keep in lockstep with the orchestrator daemon's default cloud model
# (agent-core/orchestrator/src/main.rs) -- one product decision, two
# runtimes.
DEFAULT_OPENROUTER_MODEL = "nousresearch/hermes-4-70b"


def call_openrouter_extract(
    text: str,
    model: str = DEFAULT_OPENROUTER_MODEL,
    *,
    api_key: str,
    host: str = "https://openrouter.ai/api/v1",
) -> list[CandidateFact]:
    """The actual I/O -- NOT validated against a live OpenRouter account
    (no API key exists anywhere in this project yet; that's the GUI piece
    still to come). Written against OpenRouter's documented
    /api/v1/chat/completions contract (response_format.json_schema for
    constrained decoding, verified against current docs, not assumed).
    api_key is keyword-only and has no default -- there is no such thing
    as a safe default API key, this must always be supplied explicitly.
    """
    request_body = build_openrouter_extraction_request(text, model)
    req = urllib.request.Request(
        f"{host}/chat/completions",
        data=json.dumps(request_body).encode("utf-8"),
        headers={
            "Content-Type": "application/json",
            "Authorization": f"Bearer {api_key}",
        },
        method="POST",
    )
    with urllib.request.urlopen(req, timeout=120) as resp:
        body = json.loads(resp.read().decode("utf-8"))
    return parse_extraction_response(body["choices"][0]["message"]["content"])
