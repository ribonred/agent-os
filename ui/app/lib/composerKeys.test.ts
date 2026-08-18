import { describe, expect, it } from "bun:test";
import { shouldSubmitComposer } from "./composerKeys";

function event(
  partial: Partial<{
    key: string;
    shiftKey: boolean;
    isComposing: boolean;
    keyCode: number;
  }>,
) {
  return {
    key: "Enter",
    shiftKey: false,
    isComposing: false,
    keyCode: 13,
    ...partial,
  };
}

describe("shouldSubmitComposer", () => {
  it("sends on Enter", () => {
    expect(shouldSubmitComposer(event({}))).toBe(true);
  });

  it("starts a new line on Shift+Enter", () => {
    expect(shouldSubmitComposer(event({ shiftKey: true }))).toBe(false);
  });

  it("ignores keys that are not Enter", () => {
    expect(shouldSubmitComposer(event({ key: "Tab" }))).toBe(false);
    expect(shouldSubmitComposer(event({ key: "a" }))).toBe(false);
  });

  it("does not send while an IME is composing", () => {
    expect(shouldSubmitComposer(event({ isComposing: true }))).toBe(false);
    expect(shouldSubmitComposer(event({ keyCode: 229 }))).toBe(false);
  });
});
