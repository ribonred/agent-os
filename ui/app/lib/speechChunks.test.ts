import { describe, expect, it } from "bun:test";
import { nextSpeech, speakableText } from "./speechChunks";

/// Speaking a reply as it is written means every one of these decisions
/// is made on a half-finished string, where getting it wrong is audible:
/// a sentence said twice, a price read as two sentences, or a table read
/// out as a run of numbers.

describe("speakableText", () => {
  it("leaves an ordinary reply alone", () => {
    const reply = "I moved it into Invoices for you.";
    expect(speakableText(reply)).toBe(reply);
  });

  it("never reads a table out", () => {
    const reply =
      "Here is June.\n\n| Item | Amount |\n| --- | --- |\n| Invoice 118 | 2.400.000 |\n\nThat is everything.";
    const spoken = speakableText(reply);
    expect(spoken).not.toContain("|");
    expect(spoken).not.toContain("Invoice 118");
    expect(spoken).toContain("Here is June.");
    expect(spoken).toContain("That is everything.");
  });

  it("keeps the words of a list and drops the bullets", () => {
    const spoken = speakableText("Two things:\n- the first one\n- the second one");
    expect(spoken).toContain("the first one");
    expect(spoken).not.toContain("- ");
  });

  it("says nothing that is inside a code block", () => {
    const spoken = speakableText("Run this.\n\n```\nrm -rf everything\n```\n\nThen tell me.");
    expect(spoken).not.toContain("rm -rf");
    expect(spoken).toContain("Run this.");
    expect(spoken).toContain("Then tell me.");
  });

  it("treats a block that is still being written as running to the end", () => {
    // Held rather than read: the next characters may be a command.
    const spoken = speakableText("Try this.\n\n```\nsudo something");
    expect(spoken).not.toContain("sudo");
    expect(spoken).toContain("Try this.");
  });

  it("strips the marks but keeps the words", () => {
    expect(speakableText("It is **already** there.")).toBe("It is already there.");
    expect(speakableText("The `invoices` folder.")).toBe("The invoices folder.");
    expect(speakableText("## Your June\n\nAll paid.")).toBe("Your June\n\nAll paid.");
    expect(speakableText("See [the form](https://example.com/f).")).toBe("See the form.");
  });
});

describe("nextSpeech", () => {
  it("says a sentence as soon as it is finished, and never again", () => {
    const first = nextSpeech("Your June invoices are all paid. I checked", 0, false);
    expect(first.chunks).toEqual(["Your June invoices are all paid."]);

    const second = nextSpeech(
      "Your June invoices are all paid. I checked every one of them.",
      first.cursor,
      true,
    );
    expect(second.chunks).toEqual(["I checked every one of them."]);
  });

  it("holds a sentence that has not ended yet", () => {
    expect(nextSpeech("I had a look at", 0, false).chunks).toEqual([]);
  });

  it("does not end a sentence in the middle of a price", () => {
    // The device's first market writes money this way, and reading it as
    // three sentences is how it would be heard.
    const spoken = nextSpeech("That comes to 2.400.000 rupiah altogether. Shall I file it?", 0, true);
    expect(spoken.chunks[0]).toBe("That comes to 2.400.000 rupiah altogether.");
    expect(spoken.chunks[1]).toBe("Shall I file it?");
  });

  it("keeps a decimal in one piece", () => {
    const spoken = nextSpeech("It went up by 3.5 percent since May. Not much.", 0, true);
    expect(spoken.chunks[0]).toBe("It went up by 3.5 percent since May.");
  });

  it("merges a sentence too short to be worth its own clip", () => {
    // "Yes." alone, then a pause while the next one is fetched, sounds
    // like the device stopped working.
    const spoken = nextSpeech("Yes. I filed it under Invoices this morning.", 0, true);
    expect(spoken.chunks).toEqual(["Yes. I filed it under Invoices this morning."]);
  });

  it("treats a run of terminators as one ending", () => {
    const spoken = nextSpeech("Are you quite sure about that?! I can undo it.", 0, true);
    expect(spoken.chunks[0]).toBe("Are you quite sure about that?!");
  });

  it("ends a sentence written in a full-width script", () => {
    const spoken = nextSpeech("请稍等一下，我看一看你的文件。好了，都在这里。", 0, true);
    expect(spoken.chunks.length).toBe(2);
    expect(spoken.chunks[0]).toBe("请稍等一下，我看一看你的文件。");
  });

  it("says the rest when the turn ends without a full stop", () => {
    const spoken = nextSpeech("All paid. And one still open", 0, true);
    expect(spoken.chunks).toEqual(["All paid. And one still open"]);
  });

  it("never speaks the answers trailer", () => {
    // The trailer is stripped before this by the reply parser, but a
    // half-written one arriving mid-stream must not be read either.
    const spoken = nextSpeech("Would you like me to file it? <opt", 0, false);
    expect(spoken.chunks.join(" ")).not.toContain("opt");
  });

  it("does not repeat itself as the reply grows", () => {
    const full = "First thing done here. Second thing done as well. Third and last one.";
    let cursor = 0;
    const said: string[] = [];
    for (let i = 1; i <= full.length; i += 7) {
      const step = nextSpeech(full.slice(0, i), cursor, false);
      said.push(...step.chunks);
      cursor = step.cursor;
    }
    said.push(...nextSpeech(full, cursor, true).chunks);

    // Every word said exactly once, in order.
    expect(said.join(" ").replace(/\s+/g, " ")).toBe(full);
  });

  it("starts over when the reply is replaced rather than extended", () => {
    // Reopening a conversation swaps the text under the cursor; the
    // alternative is a device that has gone permanently silent.
    const step = nextSpeech("A completely different reply.", 500, true);
    expect(step.chunks).toEqual(["A completely different reply."]);
  });
});
