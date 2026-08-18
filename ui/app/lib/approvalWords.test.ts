import { describe, expect, it } from "bun:test";
import { approvalQuestion } from "./approvalWords";

describe("approvalQuestion", () => {
  it("says what a real request means, in the owner's words", () => {
    // The exact strings the live gateway sends, taken from its own
    // rule table rather than invented.
    expect(approvalQuestion("delete in root path")).toContain("delete");
    expect(approvalQuestion("recursive delete of home directory")).toContain(
      "your own files",
    );
    expect(approvalQuestion("format filesystem (mkfs)")).toContain("erase a disk");
    expect(approvalQuestion("kill all processes")).toContain("stop programs");
    expect(approvalQuestion("recursive chown to root")).toContain("allowed to open");
    expect(approvalQuestion("pipe remote content to shell")).toContain("internet");
  });

  it("never shows the rule's own name", () => {
    const rules = [
      "delete in root path",
      "dd to raw block device",
      "recursive delete (flags after operands)",
      "modify boot configuration (bcdedit /set)",
      "delete volume shadow copies (vssadmin)",
      "execute remote script via process substitution",
      "some rule this build has never heard of",
      "",
    ];
    for (const rule of rules) {
      const shown = approvalQuestion(rule);
      expect(shown.length).toBeGreaterThan(0);
      // No engineering vocabulary survives.
      expect(shown).not.toMatch(/\bdd\b|mkfs|vssadmin|bcdedit|chown|recursive|shell/i);
      if (rule) expect(shown).not.toBe(rule);
    }
  });

  it("always leaves the owner a question they can answer", () => {
    expect(approvalQuestion("")).toContain("this device");
    expect(approvalQuestion("   ")).toContain("this device");
  });
});
