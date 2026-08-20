// The answers the agent offers alongside a question, so the owner can
// pick one instead of typing.
//
// The convention is the shell's, not the runtime's. Hermes has a real
// tool for structured questions, but its delivery path is not
// implemented for the HTTP gateway this device talks to -- the call
// fails and the model is told so mid-turn, after which it guesses. So
// the agent is taught (brain/chat-protocol.md) to end such a reply with
// a trailer, and this parses it back out. The shape deliberately mirrors
// the tool's -- at most four, best first, always answerable by typing --
// so it can be swapped for the real thing if that path ever exists.

const TRAILER = /<options>([\s\S]*?)<\/options>/gi;
/// The view the reply is about, when the device built or updated one.
/// Same shape as the options trailer and parsed by the same rules, so a
/// reply that explains the convention does not trip it.
const VIEW_TRAILER = /<view>([\s\S]*?)<\/view>/gi;
/// Exactly what `agentic-view` accepts as a name. A trailer naming
/// anything else is dropped rather than turned into a control that
/// opens nothing.
const VIEW_NAME = /^[a-z0-9]+(?:-[a-z0-9]+)*$/;
/// A trailer the model has only started writing. Stripped while a reply
/// streams so the owner never watches markup type itself out.
const PARTIAL_TRAILER =
  /<(?:o(?:p(?:t(?:i(?:o(?:n(?:s(?:>[^<]*)?)?)?)?)?)?)?|v(?:i(?:e(?:w(?:>[^<]*)?)?)?)?)?$/i;

/// Long enough for a real answer, short enough that a paragraph the
/// model mislabelled as a choice is rejected rather than rendered as an
/// unreadable button.
const MAX_OPTION_LENGTH = 200;
/// The runtime's own ceiling for the equivalent tool. More than four
/// answers is a list, and a list is something to read, not tap.
const MAX_OPTIONS = 4;

export type ParsedReply = {
  /** The reply as the owner should see it, trailers removed. */
  text: string;
  /** Empty when there was no usable trailer -- never dead buttons. */
  options: string[];
  /** The view this reply built or updated, if it named one. */
  view: string | null;
};

/// Where fenced code blocks sit, so a reply that *explains* the
/// convention does not have its example eaten as a real question.
function fencedRanges(text: string): Array<[number, number]> {
  const ranges: Array<[number, number]> = [];
  const fence = /^[ \t]*(```|~~~)/gm;
  let open: number | null = null;
  let match: RegExpExecArray | null;
  while ((match = fence.exec(text)) !== null) {
    if (open === null) {
      open = match.index;
    } else {
      ranges.push([open, fence.lastIndex]);
      open = null;
    }
  }
  // An unclosed fence runs to the end -- that is what a renderer does
  // with it too.
  if (open !== null) ranges.push([open, text.length]);
  return ranges;
}

export function normalizeOptions(raw: string[]): string[] {
  const seen = new Set<string>();
  const kept: string[] = [];
  for (const candidate of raw) {
    const option = candidate.trim();
    if (!option) continue;
    if (option.length > MAX_OPTION_LENGTH) continue;
    if (/[\n\r]/.test(option)) continue;
    const key = option.toLowerCase();
    if (seen.has(key)) continue;
    seen.add(key);
    kept.push(option);
    if (kept.length === MAX_OPTIONS) break;
  }
  // One option is not a choice: there is nothing to prefer it over, and
  // a lone button reads as the only allowed answer when the owner is
  // free to say anything.
  return kept.length >= 2 ? kept : [];
}

/// The last trailer of one kind that is not inside a fenced block, and
/// the text with it removed. Both trailers come out whether or not their
/// contents survived validation: leaving markup on screen because the
/// payload was unusable shows the owner the machinery, which is the one
/// thing it must never do.
function takeTrailer(
  text: string,
  pattern: RegExp,
  inCode: (index: number) => boolean,
): { text: string; body: string | null } {
  let chosen: RegExpExecArray | null = null;
  let match: RegExpExecArray | null;
  pattern.lastIndex = 0;
  while ((match = pattern.exec(text)) !== null) {
    if (!inCode(match.index)) chosen = match;
  }
  if (!chosen) return { text, body: null };

  return {
    text: text.slice(0, chosen.index) + text.slice(chosen.index + chosen[0].length),
    body: chosen[1] ?? "",
  };
}

export function parseReply(text: string): ParsedReply {
  const fenced = fencedRanges(text);
  const inCode = (index: number) =>
    fenced.some(([start, end]) => index >= start && index < end);

  const withOptions = takeTrailer(text, TRAILER, inCode);
  const withView = takeTrailer(withOptions.text, VIEW_TRAILER, inCode);

  const name = withView.body?.trim() ?? "";
  return {
    text: withView.text.trimEnd(),
    options: normalizeOptions((withOptions.body ?? "").split("|")),
    view: VIEW_NAME.test(name) ? name : null,
  };
}

/// The reply as it should look mid-stream: no trailer, and no half of
/// one either.
export function streamingText(text: string): string {
  return parseReply(text).text.replace(PARTIAL_TRAILER, "").trimEnd();
}
