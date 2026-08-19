import { describe, expect, it } from "bun:test";
import {
  conversationName,
  filterConversations,
  groupConversations,
  UNNAMED,
  type Conversation,
} from "./sessionList";

const NOW = new Date("2026-08-19T14:00:00");

function at(iso: string, extra: Partial<Conversation> = {}): Conversation {
  return {
    id: iso,
    title: `said at ${iso}`,
    preview: "",
    lastActive: new Date(iso).getTime() / 1000,
    messageCount: 2,
    kept: false,
    ...extra,
  };
}

function labels(conversations: Conversation[]) {
  return groupConversations(conversations, NOW).map((group) => group.label);
}

describe("groupConversations", () => {
  it("puts a conversation in the day the owner had it", () => {
    expect(
      labels([
        at("2026-08-19T09:30:00"),
        at("2026-08-18T22:10:00"),
        at("2026-08-15T11:00:00"),
        at("2026-06-02T11:00:00"),
      ]),
    ).toEqual(["Today", "Yesterday", "Earlier this week", "Before that"]);
  });

  it("counts days from midnight, not from the hour it is now", () => {
    // Half past midnight is today even though it is thirteen hours ago,
    // and 23:00 the night before is yesterday even though it is closer.
    expect(labels([at("2026-08-19T00:30:00")])).toEqual(["Today"]);
    expect(labels([at("2026-08-18T23:00:00")])).toEqual(["Yesterday"]);
  });

  it("hoists kept conversations above the days, however old", () => {
    const groups = groupConversations(
      [at("2026-08-19T09:00:00"), at("2025-01-04T09:00:00", { kept: true })],
      NOW,
    );
    expect(groups.map((group) => group.label)).toEqual(["Kept", "Today"]);
    expect(groups[0]!.conversations).toHaveLength(1);
  });

  it("leaves out a group nothing falls into", () => {
    expect(labels([at("2026-08-19T09:00:00")])).toEqual(["Today"]);
  });

  it("drops a session nothing was ever said in", () => {
    // The gateway opens one before the first turn, so a turn that failed
    // leaves a row behind. Offering it would open an empty pane.
    expect(labels([at("2026-08-19T09:00:00", { messageCount: 0 })])).toEqual([]);
  });

  it("keeps the order it was given inside a group", () => {
    const groups = groupConversations(
      [at("2026-08-19T11:00:00"), at("2026-08-19T09:00:00")],
      NOW,
    );
    expect(groups[0]!.conversations.map((c) => c.id)).toEqual([
      "2026-08-19T11:00:00",
      "2026-08-19T09:00:00",
    ]);
  });
});

describe("conversationName", () => {
  it("uses what the device called it", () => {
    expect(conversationName(at("2026-08-19T09:00:00", { title: "Track invoices" })))
      .toBe("Track invoices");
  });

  it("never falls back to an id or a date", () => {
    // A title arrives a second or two after a conversation starts. What
    // stands in for it until then must still read like a name.
    for (const title of ["", "   "]) {
      const named = conversationName(at("2026-08-19T09:00:00", { title }));
      expect(named).toBe(UNNAMED);
      expect(named).not.toContain("2026");
    }
  });
});

describe("filterConversations", () => {
  const rows = [
    at("2026-08-19T09:00:00", { title: "Track invoices" }),
    at("2026-08-18T09:00:00", { title: "Copy the school zip" }),
    at("2026-08-17T09:00:00", { title: "" }),
  ];

  it("matches any part of the name, whatever the case", () => {
    expect(filterConversations(rows, "INVOIC").map((c) => c.title)).toEqual([
      "Track invoices",
    ]);
  });

  it("returns everything when nothing was typed", () => {
    expect(filterConversations(rows, "   ")).toHaveLength(3);
  });

  it("searches the name an unnamed conversation is shown under", () => {
    // Otherwise the one row the owner can see by that name is the one
    // row typing it cannot find.
    expect(filterConversations(rows, "new conv")).toHaveLength(1);
  });
});
