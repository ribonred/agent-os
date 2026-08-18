// What the device is about to do, in words the owner can judge.
//
// The runtime describes a flagged command with the name of the rule that
// caught it -- "delete in root path", "dd to raw block device",
// "recursive chown to root". That is written for whoever maintains the
// rules, and putting it in front of the owner asks them to approve
// something they have no way to evaluate. Verified against a live
// request rather than assumed: the field really does arrive that way.
//
// Same job as lib/agentErrors.ts, same shape: match on meaning, order
// narrow before broad, and have an honest default. The exact command is
// still one tap away behind the card's disclosure for anyone who wants
// it -- this decides only what the question says.

/// Ordered most specific first: the first match wins.
const PATTERNS: Array<{ match: RegExp; message: string }> = [
  {
    match: /fork bomb|kill all|kill processes|force kill/i,
    message: "This would stop programs that are running on this device.",
  },
  {
    match: /format|mkfs|partition|diskpart|block device|disk copy|\bdd\b/i,
    message:
      "This would erase a disk on this device. Everything on it would be gone.",
  },
  {
    match: /shadow copies|wbadmin|backup/i,
    message: "This would delete this device's backups.",
  },
  {
    match: /home directory/i,
    message: "This would delete your own files, and I can't undo it.",
  },
  {
    match: /root filesystem|system director|root path/i,
    message:
      "This would delete files this device needs to work, and I can't undo it.",
  },
  {
    match: /delete|remove|\brm\b/i,
    message: "This would delete things for good. I can't undo it afterwards.",
  },
  {
    match: /chown|chmod|writable|permission/i,
    message: "This would change who is allowed to open files on this device.",
  },
  {
    match: /boot|bcdedit|registry|system (config|file)|overwrite/i,
    message: "This would change how this device starts up or runs.",
  },
  {
    match: /remote|download|curl|wget|pipe/i,
    message: "This would run something it just downloaded from the internet.",
  },
];

/// Never the rule's own name, and never nothing: a card with no question
/// on it is worse than a general one, because the owner still has to
/// answer it.
const GENERIC =
  "This one could change or remove things on this device, and I can't undo it.";

export function approvalQuestion(description: string): string {
  const raw = description.trim();
  if (raw === "") return GENERIC;
  return PATTERNS.find(({ match }) => match.test(raw))?.message ?? GENERIC;
}
