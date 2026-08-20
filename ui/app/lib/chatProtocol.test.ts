import { describe, expect, it } from "bun:test";
import { normalizeOptions, parseReply, streamingText } from "./chatProtocol";

describe("parseReply", () => {
  it("takes the answers off the end and leaves the question", () => {
    const parsed = parseReply(
      "Do you handle appointments for other people?\n<options>Yes|No|Not sure</options>",
    );
    expect(parsed.text).toBe("Do you handle appointments for other people?");
    expect(parsed.options).toEqual(["Yes", "No", "Not sure"]);
  });

  it("leaves an ordinary reply exactly as it was", () => {
    const parsed = parseReply("I moved it into Invoices for you.");
    expect(parsed.text).toBe("I moved it into Invoices for you.");
    expect(parsed.options).toEqual([]);
  });

  it("never leaves the markup on screen, even when nothing usable is in it", () => {
    // Dead buttons are worse than none, but showing the owner the
    // machinery is worse than either.
    for (const raw of [
      "Which one?\n<options></options>",
      "Which one?\n<options>   |  </options>",
      "Which one?\n<options>only one</options>",
    ]) {
      const parsed = parseReply(raw);
      expect(parsed.text).toBe("Which one?");
      expect(parsed.options).toEqual([]);
    }
  });

  it("ignores an example inside a code block", () => {
    const explaining =
      "Write it like this:\n\n```\n<options>Yes|No</options>\n```\n\nThat is the whole trick.";
    const parsed = parseReply(explaining);
    expect(parsed.options).toEqual([]);
    expect(parsed.text).toContain("<options>Yes|No</options>");
  });

  it("uses the last trailer when a reply somehow carries two", () => {
    const parsed = parseReply(
      "First?\n<options>A|B</options>\nActually, second?\n<options>C|D</options>",
    );
    expect(parsed.options).toEqual(["C", "D"]);
  });
});

describe("normalizeOptions", () => {
  it("drops what cannot be a button and keeps the order", () => {
    expect(normalizeOptions(["  Yes  ", "", "No"])).toEqual(["Yes", "No"]);
    expect(normalizeOptions(["Yes", "line\nbreak", "No"])).toEqual(["Yes", "No"]);
    expect(normalizeOptions(["Yes", "x".repeat(201), "No"])).toEqual(["Yes", "No"]);
  });

  it("never offers more than four", () => {
    expect(normalizeOptions(["a", "b", "c", "d", "e"])).toEqual(["a", "b", "c", "d"]);
  });

  it("treats the same answer twice as one answer", () => {
    // Which then leaves a single option, and a single option is not a
    // choice at all.
    expect(normalizeOptions(["Yes", "yes"])).toEqual([]);
    expect(normalizeOptions(["Yes", "YES", "No"])).toEqual(["Yes", "No"]);
  });
});

describe("streamingText", () => {
  it("hides a trailer the model has only started writing", () => {
    expect(streamingText("Shall I?\n<opt")).toBe("Shall I?");
    expect(streamingText("Shall I?\n<options>Ye")).toBe("Shall I?");
    expect(streamingText("Shall I?\n<options>Yes|No</options>")).toBe("Shall I?");
  });

  it("does not eat a less-than the owner is meant to read", () => {
    expect(streamingText("Use a < b to compare")).toBe("Use a < b to compare");
    expect(streamingText("The total is < 40")).toBe("The total is < 40");
  });
});

describe("the view a reply points at", () => {
  it("takes the name out of the reply and offers it", () => {
    const parsed = parseReply("I've put June's takings on screen.\n<view>june-takings</view>");
    expect(parsed.view).toBe("june-takings");
    // The owner must never see the markup that carried it.
    expect(parsed.text).toBe("I've put June's takings on screen.");
  });

  it("carries options and a view in the same reply", () => {
    const parsed = parseReply(
      "Here it is. Want last month too?\n<view>june-takings</view>\n<options>Yes|No</options>",
    );
    expect(parsed.view).toBe("june-takings");
    expect(parsed.options).toEqual(["Yes", "No"]);
    expect(parsed.text).toBe("Here it is. Want last month too?");
  });

  it("refuses a name the command could never have made", () => {
    // A control that opens nothing is worse than no control: the owner
    // taps it, nothing happens, and they learn the device is unreliable.
    for (const bad of ["../etc/passwd", "June Takings", "", "a/b", "UPPER"]) {
      expect(parseReply(`text\n<view>${bad}</view>`).view).toBeNull();
    }
  });

  it("still strips a trailer whose name was unusable", () => {
    const parsed = parseReply("Done.\n<view>Not A Slug</view>");
    expect(parsed.view).toBeNull();
    expect(parsed.text).toBe("Done.");
  });

  it("leaves an example inside a fence alone", () => {
    const reply = "Write it like this:\n\n```\n<view>june-takings</view>\n```";
    const parsed = parseReply(reply);
    expect(parsed.view).toBeNull();
    expect(parsed.text).toContain("<view>june-takings</view>");
  });

  it("never types half a trailer onto the screen mid-stream", () => {
    expect(streamingText("On screen now.\n<vi")).toBe("On screen now.");
    expect(streamingText("On screen now.\n<view>june-tak")).toBe("On screen now.");
    expect(streamingText("On screen now.\n<view>june-takings</view>")).toBe(
      "On screen now.",
    );
  });
});
