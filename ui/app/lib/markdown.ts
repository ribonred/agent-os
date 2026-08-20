// Assistant replies are markdown. A reply that arrives as "- one" and
// renders as a literal hyphen tells the owner they are looking at raw
// machine output, which is the opposite of what this device claims to
// be.
//
// Two independent limits, on purpose. The parser is configured to
// produce only the subset design/DESIGN.md names, and the sanitizer then
// allows only that same subset through. Either one alone would be
// enough on a good day; the model's output is not a trusted input and
// this is the last place it becomes markup.

import DOMPurify from "dompurify";
import MarkdownIt from "markdown-it";

// html: false is the important one -- anything HTML-shaped in a reply is
// shown as the text it is rather than becoming part of the page.
// linkify would turn every bare filename with a dot in it into a link,
// on a surface whose whole subject is filenames.
const md = new MarkdownIt({
  html: false,
  linkify: false,
  typographer: false,
  breaks: true,
});

// Nothing is ever fetched at runtime -- the device may never see a
// network, and a reply that renders as a broken-image icon is worse than
// one that renders as the text the model wrote.
md.disable(["image"]);

/// Exactly what a reply may contain. Anything else is a request to
/// revisit DESIGN.md, not a tag to quietly add.
///
/// Exported so a test can hold it against what the agent is told it may
/// write (brain/chat-protocol.md). The two drifting apart is the failure
/// worth catching: a device that offers the owner a table it then strips
/// on the way to the screen, or renders a mark it never uses.
export const ALLOWED_TAGS = [
  "p", "br", "hr",
  // The parser emits <s> for ~~struck through~~, never <del> -- an
  // earlier list allowed only <del>, so strikethrough was stripped back
  // to bare text and the mark silently did nothing. Both are listed
  // because either spelling means the same thing to a reader.
  "strong", "em", "s", "del",
  "ul", "ol", "li",
  "h1", "h2", "h3", "h4", "h5", "h6",
  "blockquote",
  "code", "pre",
  "table", "thead", "tbody", "tr", "th", "td",
];

/// The markup, before sanitizing. Separated so the parser's
/// configuration can be tested for what it does and does not produce,
/// which is a string question and not a DOM one.
export function toHtml(text: string): string {
  return md.render(text);
}

export function renderMarkdown(text: string): string {
  return DOMPurify.sanitize(toHtml(text), {
    ALLOWED_TAGS,
    // No attributes at all: there is no styling, no id, and no href the
    // model has any business setting on this surface.
    ALLOWED_ATTR: [],
  });
}

/// The text of one code block, for the copy control. Reads from the
/// rendered DOM rather than re-parsing the reply, so what is copied is
/// exactly what is on screen.
export function codeBlockText(element: Element): string {
  return element.querySelector("code")?.textContent ?? element.textContent ?? "";
}
