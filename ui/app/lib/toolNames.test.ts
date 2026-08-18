import { describe, expect, it } from "bun:test";
import { toolSummary } from "./toolNames";

describe("toolSummary", () => {
  it("says what happened in the owner's words", () => {
    expect(toolSummary("search_files")).toBe("Looked through your files");
    expect(toolSummary("terminal")).toBe("Ran something on this device");
  });

  it("never puts a tool's own name on the screen", () => {
    // A runtime upgrade adds tools this table has never heard of, and
    // the day that happens must not be the day "acp_bridge" appears in
    // front of the owner.
    for (const unknown of ["acp_bridge", "doc_extract", "some_new_tool", ""]) {
      const summary = toolSummary(unknown);
      expect(summary).not.toContain("_");
      if (unknown) expect(summary).not.toContain(unknown);
      expect(summary.length).toBeGreaterThan(0);
    }
  });

  it("treats the whole browser family as the browser", () => {
    expect(toolSummary("browser_navigate")).toBe(toolSummary("browser"));
    expect(toolSummary("browser_click")).toBe(toolSummary("browser"));
  });
});
