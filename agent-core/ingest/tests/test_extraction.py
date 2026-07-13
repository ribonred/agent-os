import json

import pytest

from extraction import (
    DEFAULT_OPENROUTER_MODEL,
    EXTRACTION_SCHEMA,
    CandidateFact,
    ExtractionParseError,
    build_extraction_request,
    build_openrouter_extraction_request,
    parse_extraction_response,
)


def test_build_extraction_request_shape() -> None:
    req = build_extraction_request("We are a skincare shop.", "hermes3:3b")

    assert req["model"] == "hermes3:3b"
    assert req["stream"] is False
    assert "We are a skincare shop." in req["prompt"]
    assert req["format"]["required"] == ["facts"]


def test_build_openrouter_extraction_request_shape() -> None:
    req = build_openrouter_extraction_request("We are a skincare shop.", "some-model")

    assert req["model"] == "some-model"
    assert len(req["messages"]) == 1
    assert req["messages"][0]["role"] == "user"
    assert "We are a skincare shop." in req["messages"][0]["content"]
    assert req["response_format"]["type"] == "json_schema"
    assert req["response_format"]["json_schema"]["strict"] is True
    assert req["response_format"]["json_schema"]["schema"] == EXTRACTION_SCHEMA


def test_openrouter_and_ollama_share_the_same_extraction_schema() -> None:
    ollama_req = build_extraction_request("text", "m")
    openrouter_req = build_openrouter_extraction_request("text", "m")

    assert ollama_req["format"] == openrouter_req["response_format"]["json_schema"]["schema"]


def test_default_openrouter_model_is_hermes() -> None:
    # Product decision: cloud escalations default to Hermes via
    # OpenRouter, not a frontier model -- consistency with the local
    # Ollama tier, lower cost for what should be routine escalations.
    assert "hermes" in DEFAULT_OPENROUTER_MODEL.lower()


def test_parses_well_formed_response() -> None:
    raw = json.dumps(
        {
            "facts": [
                {
                    "entity": "business",
                    "field": "business_type",
                    "value": "skincare practice",
                    "source_quote": "We are a skincare shop.",
                },
                {
                    "entity": "business",
                    "field": "staff_count",
                    "value": None,
                    "source_quote": None,
                },
            ]
        }
    )

    facts = parse_extraction_response(raw)

    assert facts == [
        CandidateFact(
            entity="business",
            field="business_type",
            value="skincare practice",
            source_quote="We are a skincare shop.",
        ),
        CandidateFact(
            entity="business", field="staff_count", value=None, source_quote=None
        ),
    ]


def test_null_value_means_not_yet_known_not_an_error() -> None:
    # This is the concrete check that onboarding.md's "not yet known is a
    # legitimate value" rule actually round-trips through parsing.
    raw = json.dumps(
        {"facts": [{"entity": "business", "field": "x", "value": None, "source_quote": None}]}
    )

    facts = parse_extraction_response(raw)

    assert facts[0].value is None


def test_empty_facts_list_is_valid_not_an_error() -> None:
    assert parse_extraction_response(json.dumps({"facts": []})) == []


def test_invalid_json_raises_extraction_parse_error() -> None:
    with pytest.raises(ExtractionParseError, match="not valid JSON"):
        parse_extraction_response("this is not json")


def test_missing_facts_key_raises_extraction_parse_error() -> None:
    with pytest.raises(ExtractionParseError, match="missing a 'facts' array"):
        parse_extraction_response(json.dumps({"oops": []}))


def test_fact_missing_required_key_raises_extraction_parse_error() -> None:
    raw = json.dumps({"facts": [{"entity": "business", "field": "x"}]})

    with pytest.raises(ExtractionParseError, match="missing required key"):
        parse_extraction_response(raw)
