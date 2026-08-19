// Turning the device's conversations into the list the owner reads.
//
// Ordering and naming are the gateway's job and are already done by the
// time rows get here. What is left is a question about the owner's day
// rather than about storage: people remember roughly when they asked
// something far better than they remember what the conversation ended up
// being called, so the list is grouped by that and never by a date they
// have to read off each row.

export type Conversation = {
  id: string;
  title: string;
  preview: string;
  /// Seconds since the epoch, as the gateway reports it.
  lastActive: number;
  messageCount: number;
  kept: boolean;
};

export type ConversationGroup = {
  label: string;
  conversations: Conversation[];
};

/// What a conversation is called before the device has named it. Titles
/// are written from the owner's opening words and land a second or two
/// after a conversation starts, so this is what the newest row says for
/// that moment -- never an id, and never a timestamp standing in for a
/// name.
export const UNNAMED = "New conversation";

export function conversationName(conversation: Conversation): string {
  const title = conversation.title.trim();
  return title === "" ? UNNAMED : title;
}

/// Midnight at the start of the day `date` falls in, in the device's own
/// timezone. "Yesterday" has to mean the owner's yesterday, so the
/// boundaries are calendar days rather than 24-hour windows -- something
/// said at 00:30 was said today even though it was ten minutes ago.
function startOfDay(date: Date): number {
  const midnight = new Date(date);
  midnight.setHours(0, 0, 0, 0);
  return midnight.getTime();
}

const DAY_MS = 86_400_000;

/// Groups conversations for display, newest first within each group.
///
/// `now` is passed in rather than read here so this is a pure function
/// that can be tested against a fixed clock.
export function groupConversations(
  conversations: Conversation[],
  now: Date = new Date(),
): ConversationGroup[] {
  const today = startOfDay(now);
  const yesterday = today - DAY_MS;
  const week = today - 6 * DAY_MS;

  const kept: Conversation[] = [];
  const buckets: Record<string, Conversation[]> = {
    Today: [],
    Yesterday: [],
    "Earlier this week": [],
    "Before that": [],
  };

  for (const conversation of conversations) {
    // A session the gateway opened for a turn that then failed is not a
    // conversation, and offering it would open an empty pane.
    if (conversation.messageCount === 0) continue;
    if (conversation.kept) {
      kept.push(conversation);
      continue;
    }
    const at = conversation.lastActive * 1000;
    if (at >= today) buckets.Today!.push(conversation);
    else if (at >= yesterday) buckets.Yesterday!.push(conversation);
    else if (at >= week) buckets["Earlier this week"]!.push(conversation);
    else buckets["Before that"]!.push(conversation);
  }

  const groups: ConversationGroup[] = [];
  // Kept ones sit above the days: the owner said these must stay
  // reachable, and burying one under a date heading is the opposite of
  // what they asked for.
  if (kept.length > 0) groups.push({ label: "Kept", conversations: kept });
  for (const label of Object.keys(buckets)) {
    const conversations = buckets[label]!;
    if (conversations.length > 0) groups.push({ label, conversations });
  }
  return groups;
}

/// Narrowing the list by what a conversation is called.
///
/// Only what is already loaded, and only titles: the runtime can search
/// its own transcripts but does not offer that over this interface, and
/// a box that looks like it searches everything while quietly searching
/// one page is worse than no box at all.
export function filterConversations(
  conversations: Conversation[],
  query: string,
): Conversation[] {
  const needle = query.trim().toLowerCase();
  if (needle === "") return conversations;
  return conversations.filter((conversation) =>
    conversationName(conversation).toLowerCase().includes(needle),
  );
}
