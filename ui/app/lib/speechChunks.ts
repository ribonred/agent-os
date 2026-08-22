// Turning a reply that is being written into sentences that can be
// spoken while the rest of it is still arriving.
//
// The device speaks a sentence at a time rather than waiting for the
// whole answer, because the wait is the entire difference between a
// device that responds and one that thinks about it first. That means
// deciding, on a half-finished reply, which part of it is finished
// enough to say out loud -- and never speaking a sentence twice.
//
// Two separate jobs, in order:
//
//   1. `speakableText` -- what a listener should hear at all. A reply is
//      written to be *read*, and brain/chat-protocol.md tells the agent
//      to drop the reading-only shapes when a turn is spoken; this is
//      the safety net for when it doesn't. A table read aloud is a
//      stream of numbers with nothing holding them apart, and a bulleted
//      list is the word "dash" over and over.
//   2. `nextSpeech` -- which complete sentences of that are new since
//      last time.
//
// Both are pure, so the awkward cases are settled in a test file rather
// than by listening to the device get them wrong.

/// Sentence terminators, including the full-width ones -- several of the
/// supported languages do not use the ASCII stop.
const TERMINATORS = ".!?…。！？";

/// The full-width ones end a sentence on their own. Their scripts are
/// written without spaces between words, so the space that tells a Latin
/// sentence apart from an abbreviation is never there to look for -- and
/// waiting for one means a whole reply is read out as a single breath.
const FULL_WIDTH_TERMINATORS = "。！？";

/// Below this, a sentence is merged with the one after it. "Yes." spoken
/// entirely on its own, then a pause while the next clip is fetched,
/// sounds like the device stopped working. The wait it costs is shorter
/// than the pause it removes.
///
/// Measured in the weights below rather than in characters, because a
/// character is not the same amount of speech in every script the device
/// supports.
const MIN_CHUNK_WEIGHT = 24;

/// Scripts written without spaces, where one character is a whole
/// syllable or word rather than a letter. Counting their characters the
/// same as Latin ones would treat a complete Mandarin sentence as a
/// fragment and glue it to the next one -- roughly a paragraph before
/// the device said anything at all.
const DENSE_SCRIPT =
  /[\u3040-\u30ff\u3400-\u4dbf\u4e00-\u9fff\uac00-\ud7af\uff00-\uff9f]/;

/// Roughly how much speech a string is, in Latin characters.
function spokenWeight(text: string): number {
  let weight = 0;
  for (const char of text.trim()) {
    weight += DENSE_SCRIPT.test(char) ? 3 : 1;
  }
  return weight;
}

/// A fence the model has only started writing. Held back rather than
/// spoken, in case the next character makes it a code block.
const PARTIAL_FENCE = /(?:^|\n)[ \t]*`{1,2}$/;

/// What a listener should hear: the reply with the shapes that only mean
/// anything on a screen taken out.
///
/// The result grows only at its end as more of the reply arrives, which
/// is what lets a cursor into it stay valid across calls. An unterminated
/// code fence is treated as running to the end of the text -- the same
/// thing a renderer does with one -- so the part before it stays stable
/// as the block fills in.
export function speakableText(text: string): string {
  let out = text;

  // Fenced blocks, closed or not. Nothing inside one is ever read out.
  out = out.replace(/(^|\n)[ \t]*(```|~~~)[\s\S]*?(\n[ \t]*\2[^\n]*(?=\n|$)|$)/g, "$1");

  // A table is the worst thing this could read aloud, and it is the one
  // shape the agent is most likely to reach for. Any line that is a row
  // goes, header rules included.
  out = out
    .split("\n")
    .filter((line) => !/^[ \t]*\|/.test(line))
    .join("\n");

  // Horizontal rules, and the heading's own marks -- the words of a
  // heading are worth hearing, the hashes are not.
  out = out.replace(/(^|\n)[ \t]*(?:-{3,}|\*{3,}|_{3,})[ \t]*(?=\n|$)/g, "$1");
  out = out.replace(/(^|\n)[ \t]*#{1,6}[ \t]+/g, "$1");

  // List markers. The item is a sentence; the bullet is punctuation the
  // ear cannot hear anyway.
  out = out.replace(/(^|\n)[ \t]*(?:[-*+]|\d+[.)])[ \t]+/g, "$1");
  // Quoted lines keep their words.
  out = out.replace(/(^|\n)[ \t]*>[ \t]?/g, "$1");

  // A link's words are the part that was worth saying; the address is
  // not something anyone can act on by ear.
  out = out.replace(/\[([^\]]*)\]\([^)]*\)/g, "$1");

  // Emphasis and inline code: the marks go, the words stay.
  out = out.replace(/`([^`]*)`/g, "$1");
  out = out.replace(/(\*\*|__|~~)(.*?)\1/g, "$2");
  out = out.replace(/(?<![\w*])\*(?!\s)([^*\n]+?)(?<!\s)\*(?![\w*])/g, "$1");

  // Blank runs left behind by everything above would otherwise be heard
  // as hesitation.
  out = out.replace(/[ \t]+\n/g, "\n").replace(/\n{3,}/g, "\n\n");

  return out;
}

/// True where a `.` is part of a number rather than the end of a
/// sentence. Both conventions matter here: `3.14`, and a thousands
/// separator like `2.400.000`, which is how the device's first market
/// writes money.
function insideNumber(text: string, index: number): boolean {
  const before = text[index - 1];
  const after = text[index + 1];
  return (
    before !== undefined &&
    after !== undefined &&
    before >= "0" &&
    before <= "9" &&
    after >= "0" &&
    after <= "9"
  );
}

/// The end of the sentence starting at `from`, or -1 if the text does
/// not contain a finished one yet.
///
/// A terminator only ends a sentence when something follows it that is
/// not more sentence -- whitespace, or the end of a finished reply.
/// Mid-stream the end of the string is not an ending: the reply may be
/// about to continue the same sentence.
function sentenceEnd(text: string, from: number, done: boolean): number {
  for (let i = from; i < text.length; i += 1) {
    const char = text[i] as string;
    if (!TERMINATORS.includes(char)) continue;
    if (char === "." && insideNumber(text, i)) continue;

    // "?!" and "..." are one ending, not two or three.
    let end = i;
    while (end + 1 < text.length && TERMINATORS.includes(text[end + 1] as string)) {
      end += 1;
    }

    if (end + 1 >= text.length) return done ? end + 1 : -1;
    if (FULL_WIDTH_TERMINATORS.includes(text[end] as string)) return end + 1;
    if (/\s/.test(text[end + 1] as string)) return end + 1;
    // A terminator with a word pressed against it -- a file name, an
    // address -- is not an ending.
    i = end;
  }
  return -1;
}

export type Speech = {
  /** Sentences to say now, in order. Empty when nothing is ready. */
  chunks: string[];
  /** How far into the speakable text has now been handed over. */
  cursor: number;
};

/// The sentences of `text` that have finished since `cursor`.
///
/// `text` is the reply as written so far, and `cursor` is what a previous
/// call returned -- an index into the *speakable* projection of it, not
/// into the raw markdown. Call it again with the same cursor and a
/// longer reply and it will not repeat itself.
///
/// `done` marks the turn as finished, which flushes whatever is left
/// even if the agent never wrote a final full stop.
export function nextSpeech(text: string, cursor: number, done: boolean): Speech {
  const speakable = speakableText(text);
  // The reply was replaced rather than extended -- a new turn, or a
  // conversation reopened. Start again rather than slicing from a cursor
  // that now means nothing.
  const from = cursor > speakable.length ? 0 : cursor;

  let pending = speakable.slice(from);
  if (!done && PARTIAL_FENCE.test(pending)) {
    return { chunks: [], cursor: from };
  }

  const chunks: string[] = [];
  let taken = 0;
  let held = "";

  for (;;) {
    const rest = pending.slice(taken);
    if (rest.trim() === "") break;

    const end = sentenceEnd(rest, 0, done);
    if (end === -1) break;

    held = `${held}${rest.slice(0, end)}`;
    taken += end;

    // Too short to be worth a clip of its own: keep it, and let whatever
    // comes next be said in the same breath. The exception is a short
    // sentence that is the last thing in a finished reply -- there is
    // nothing left to merge it with, and it still has to be said.
    if (spokenWeight(held) < MIN_CHUNK_WEIGHT) {
      const more = pending.slice(taken).trim() !== "";
      if (more || !done) continue;
    }

    chunks.push(held.trim());
    held = "";
  }

  // A finished turn says the rest whatever shape it is in. An unfinished
  // one keeps it: the sentence may not be over.
  if (done) {
    const rest = `${held}${pending.slice(taken)}`.trim();
    if (rest !== "") chunks.push(rest);
    return { chunks: chunks.filter(Boolean), cursor: speakable.length };
  }

  // Whatever was held back for being too short is not spoken yet, so the
  // cursor must not move past it either.
  return {
    chunks: chunks.filter(Boolean),
    cursor: from + taken - held.length,
  };
}
