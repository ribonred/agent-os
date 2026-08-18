import { describe, expect, it } from "bun:test";
import { toHtml } from "./markdown";

describe("the reply's markup", () => {
  it("renders the structure a reply actually uses", () => {
    expect(toHtml("- one\n- two")).toContain("<li>one</li>");
    expect(toHtml("**paid**")).toContain("<strong>paid</strong>");
    expect(toHtml("`june.xlsx`")).toContain("<code>june.xlsx</code>");
    expect(toHtml("```\nls\n```")).toContain("<pre><code>");
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
