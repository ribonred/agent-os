import { describe, expect, it } from "bun:test";
import { ALLOWED_TAGS, toHtml } from "./markdown";

describe("the reply's markup", () => {
  it("renders the structure a reply actually uses", () => {
    expect(toHtml("- one\n- two")).toContain("<li>one</li>");
    expect(toHtml("**paid**")).toContain("<strong>paid</strong>");
    expect(toHtml("`june.xlsx`")).toContain("<code>june.xlsx</code>");
    expect(toHtml("```\nls\n```")).toContain("<pre><code>");
  });

  // Everything the agent is told it may write in brain/chat-protocol.md,
  // held against what the parser actually produces. The prompt promising
  // a mark the renderer drops is a device that formats an answer into
  // nothing, and it fails silently on the owner's screen.
  it("renders every mark the agent is told it can use", () => {
    const emitted = (markdown: string) =>
      [...toHtml(markdown).matchAll(/<([a-z][a-z0-9]*)/g)].map((m) => m[1]!);

    const cases: Array<[string, string]> = [
      ["## Heading", "h2"],
      ["> quoted", "blockquote"],
      ["1. one", "ol"],
      ["a\n\n---\n\nb", "hr"],
      ["*maybe*", "em"],
      ["~~gone~~", "s"],
      ["| a | b |\n| --- | --- |\n| 1 | 2 |", "table"],
    ];

    for (const [markdown, tag] of cases) {
      const tags = emitted(markdown);
      expect(tags).toContain(tag);
      // Producing it is half the promise; surviving the sanitizer is the
      // other half, and only the second one reaches the owner.
      for (const produced of tags) expect(ALLOWED_TAGS).toContain(produced);
    }
  });

  it("keeps a table whole, header row and all", () => {
    const rendered = toHtml("| Item | Amount |\n| --- | --- |\n| Invoice | 12 |");
    expect(rendered).toContain("<th>Item</th>");
    expect(rendered).toContain("<td>Invoice</td>");
  });

  // The agent is told to write an address as plain text for exactly this
  // reason: the tag carrying it is not one a reply may contain, so the
  // owner would be left holding the label with nowhere to go.
  it("has no tag that could carry a link or an image", () => {
    expect(ALLOWED_TAGS).not.toContain("a");
    expect(ALLOWED_TAGS).not.toContain("img");
  });

  it("shows HTML in a reply as text instead of running it", () => {
    // The model's output is not a trusted input.
    const rendered = toHtml('<img src=x onerror="alert(1)">');
    expect(rendered).not.toContain("<img");
    expect(rendered).toContain("&lt;img");

    const script = toHtml("<script>alert(1)</script>");
    expect(script).not.toContain("<script>");
  });

  it("never produces an image, because nothing is fetched at runtime", () => {
    expect(toHtml("![a picture](photo.png)")).not.toContain("<img");
  });

  it("leaves a filename alone instead of making it a link", () => {
    const rendered = toHtml("Open price-list-2026.xlsx when you can.");
    expect(rendered).not.toContain("<a ");
    expect(rendered).toContain("price-list-2026.xlsx");
  });
});
