import { describe, expect, it } from "bun:test";
import { agentErrorMessage } from "./agentErrors";

// The inputs below are the real strings the Rust layer produces, copied
// from src-tauri/src/agent.rs rather than invented -- a translation layer
// tested against imagined input proves nothing about what an owner will
// actually be shown.

describe("agentErrorMessage", () => {
  it("never passes the system's own words through", () => {
    const raw = [
      "agent gateway unreachable: tcp connect error: Connection refused (os error 111)",
      "no Hermes API key found: set AGENTIC_OS_HERMES_KEY, or provide API_SERVER_KEY in /etc/agentic-os/hermes.env (device) or ~/.hermes/.env (dev)",
      "agent gateway sent invalid JSON: expected value at line 1 column 1",
      "could not open setup store: Permission denied (os error 13)",
      "agent gateway rejected chat (401): unauthorized",
      "chat stream failed mid-response: connection reset",
      "bad gateway URL: invalid port number",
    ];

    for (const text of raw) {
      const shown = agentErrorMessage("chat", text);
      // Nothing that identifies the machine may survive translation.
      expect(shown).not.toMatch(/\//); // no paths
      expect(shown).not.toMatch(/[A-Z]{2,}_[A-Z_]+/); // no env var names
      expect(shown).not.toMatch(/os error|errno|\bhttp\b|gateway|json|tcp/i);
      expect(shown).not.toMatch(/\b\d{3}\b/); // no status codes
      expect(shown.length).toBeGreaterThan(0);
    }
  });

  it("says the assistant is still starting when it cannot be reached", () => {
    const shown = agentErrorMessage(
      "chat",
      "agent gateway unreachable: tcp connect error: Connection refused (os error 111)",
    );
    expect(shown).toBe("My assistant isn't responding yet. It may still be starting up.");
  });

  it("distinguishes a device misconfiguration from a transient outage", () => {
    // The owner cannot fix a missing key by waiting, so this must not
    // read as "try again in a moment".
    const shown = agentErrorMessage(
      "chat",
      "no Hermes API key found: set AGENTIC_OS_HERMES_KEY, or provide API_SERVER_KEY in /etc/agentic-os/hermes.env (device)",
    );
    expect(shown).toMatch(/needs someone to look at it/);
  });

  it("treats a rejected session or chat as a credential problem", () => {
    for (const text of [
      "agent gateway refused session (403): forbidden",
      "agent gateway rejected chat (401): unauthorized",
    ]) {
      expect(agentErrorMessage("chat", text)).toMatch(/needs someone to look at it/);
    }
  });

  it("tells the owner a key was rejected, because that one they can fix", () => {
    const shown = agentErrorMessage("cloudKey", "invalid api key provided");
    expect(shown).toMatch(/wasn't accepted/);
  });

  it("says an answer was cut off rather than that nothing arrived", () => {
    // The owner is looking at half a reply; "I couldn't reach my
    // assistant" would contradict what is on their screen.
    const shown = agentErrorMessage(
      "chat",
      "chat stream failed mid-response: connection reset",
    );
    expect(shown).toMatch(/stopped part-way/);
  });

  it("separates a fault inside the assistant from one reaching it", () => {
    expect(agentErrorMessage("chat", "agent gateway returned 500: internal error")).toMatch(
      /went wrong inside/,
    );
  });

  it("does not mistake an unrelated number for a server fault", () => {
    // The 5xx pattern must not fire on a port, a byte count, or a year.
    expect(agentErrorMessage("chat", "bad gateway URL: invalid port 5000")).not.toMatch(
      /went wrong inside/,
    );
  });

  it("falls back to a per-surface sentence for anything unrecognised", () => {
    expect(agentErrorMessage("chat", "frame error")).toBe(
      "I couldn't reach my assistant just now.",
    );
    expect(agentErrorMessage("setup", "frame error")).toMatch(/Setup can continue/);
    expect(agentErrorMessage("cloudKey", "frame error")).toBe(
      "I couldn't save that just now.",
    );
  });

  it("accepts an Error object as well as a string", () => {
    const shown = agentErrorMessage(
      "chat",
      new Error("agent gateway unreachable: connection refused"),
    );
    expect(shown).toMatch(/still be starting up/);
  });
});
