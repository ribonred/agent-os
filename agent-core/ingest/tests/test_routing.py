from pathlib import Path

import pytest

from routing import (
    RoutingDecisionError,
    choose_extraction_backend,
    extract,
    get_routing_decision,
)

HW_PROBE_BINARY = (
    Path(__file__).parent.parent.parent / "hw-probe" / "target" / "debug" / "hw-probe"
)


def test_local_routing_chooses_ollama() -> None:
    assert choose_extraction_backend({"default_routing": "local"}) == "ollama"


def test_cloud_routing_chooses_openrouter() -> None:
    assert choose_extraction_backend({"default_routing": "cloud"}) == "openrouter"


def test_unrecognized_routing_value_raises() -> None:
    with pytest.raises(RoutingDecisionError, match="unrecognized default_routing"):
        choose_extraction_backend({"default_routing": "sideways"})


def test_missing_routing_key_raises() -> None:
    with pytest.raises(RoutingDecisionError, match="unrecognized default_routing"):
        choose_extraction_backend({})


def test_nonexistent_hw_probe_binary_raises_explicitly() -> None:
    with pytest.raises(RoutingDecisionError, match="hw-probe failed to run"):
        get_routing_decision(Path("/nonexistent/hw-probe"))


@pytest.mark.skipif(
    not HW_PROBE_BINARY.exists(),
    reason="hw-probe not built -- run `cargo build` in agent-core/hw-probe first",
)
def test_real_hw_probe_binary_produces_a_valid_routing_decision() -> None:
    # Genuine end-to-end check: runs the actual compiled Rust binary, not
    # a fabricated JSON string. This is the first real cross-language
    # integration point in the project.
    result = get_routing_decision(HW_PROBE_BINARY)

    assert "default_routing" in result
    assert result["default_routing"] in ("local", "cloud")
    assert isinstance(result["logical_cores"], int)
    # Real backend choice actually resolves without raising.
    backend = choose_extraction_backend(result)
    assert backend in ("ollama", "openrouter")


def test_extract_dispatches_to_ollama_when_routing_is_local(monkeypatch) -> None:
    # Probe result fabricated/controlled here -- the dispatch logic is
    # what's under test, not hw-probe itself (that's covered separately
    # by test_real_hw_probe_binary_produces_a_valid_routing_decision).
    # The extraction call is faked too, since neither backend is live in
    # this sandbox (see extraction.py's own docstrings for that gap).
    monkeypatch.setattr(
        "routing.get_routing_decision", lambda _binary: {"default_routing": "local"}
    )
    calls = []
    monkeypatch.setattr(
        "routing.call_ollama_extract",
        lambda text, model: calls.append((text, model)) or [],
    )

    result = extract("some text", Path("/irrelevant"))

    assert calls == [("some text", "hermes3:3b")]
    assert result == []


def test_extract_dispatches_to_openrouter_when_routing_is_cloud(monkeypatch) -> None:
    monkeypatch.setattr(
        "routing.get_routing_decision", lambda _binary: {"default_routing": "cloud"}
    )
    calls = []
    monkeypatch.setattr(
        "routing.call_openrouter_extract",
        lambda text, api_key, model=None: calls.append((text, api_key)) or [],
    )

    result = extract("some text", Path("/irrelevant"), openrouter_api_key="test-key")

    assert calls == [("some text", "test-key")]
    assert result == []


def test_extract_raises_when_cloud_chosen_without_api_key(monkeypatch) -> None:
    monkeypatch.setattr(
        "routing.get_routing_decision", lambda _binary: {"default_routing": "cloud"}
    )

    with pytest.raises(RoutingDecisionError, match="no OpenRouter API key"):
        extract("some text", Path("/irrelevant"))
