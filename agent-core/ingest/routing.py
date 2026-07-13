"""Reads hw-probe's routing decision and dispatches an extraction call
to the right backend. Two runtimes, one source of truth: this does not
reimplement hardware/routing detection in Python, it shells out to the
real compiled Rust binary rather than duplicating that logic.
"""

import json
import subprocess
from pathlib import Path

from extraction import CandidateFact, call_ollama_extract, call_openrouter_extract


class RoutingDecisionError(Exception):
    """hw-probe couldn't be run, its output was unparseable, or the
    routing decision couldn't be acted on (e.g. cloud chosen but no
    API key available). Always explicit, never a silent fallback to
    a default backend -- see red-mindset: no silent fallback.
    """


def get_routing_decision(hw_probe_binary: Path) -> dict:
    """Real I/O -- runs the actual hw-probe binary and parses its JSON."""
    try:
        result = subprocess.run(
            [str(hw_probe_binary)],
            capture_output=True,
            text=True,
            timeout=10,
            check=True,
        )
    except (subprocess.CalledProcessError, subprocess.TimeoutExpired, FileNotFoundError) as e:
        raise RoutingDecisionError(f"hw-probe failed to run: {e}") from e

    try:
        return json.loads(result.stdout)
    except json.JSONDecodeError as e:
        raise RoutingDecisionError(f"hw-probe output was not valid JSON: {e}") from e


def choose_extraction_backend(probe_result: dict) -> str:
    """Pure -- maps hw-probe's default_routing field to a backend name.
    No I/O, fully testable with a fabricated probe_result dict.
    """
    routing = probe_result.get("default_routing")
    if routing == "local":
        return "ollama"
    if routing == "cloud":
        return "openrouter"
    raise RoutingDecisionError(f"unrecognized default_routing value: {routing!r}")


def extract(
    text: str,
    hw_probe_binary: Path,
    *,
    ollama_model: str = "hermes3:3b",
    openrouter_api_key: str | None = None,
) -> list[CandidateFact]:
    """Orchestrates probe -> decide -> call. The probe step is real I/O
    against the actual hw-probe binary; the extraction call itself is
    only as validated as call_ollama_extract/call_openrouter_extract are
    (neither has touched a live backend in this sandbox -- see their own
    docstrings).
    """
    probe_result = get_routing_decision(hw_probe_binary)
    backend = choose_extraction_backend(probe_result)

    if backend == "ollama":
        return call_ollama_extract(text, model=ollama_model)

    if openrouter_api_key is None:
        raise RoutingDecisionError(
            "routing decided 'cloud' but no OpenRouter API key was provided"
        )
    return call_openrouter_extract(text, api_key=openrouter_api_key)
