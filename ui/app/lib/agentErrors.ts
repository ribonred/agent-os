// The owner never reads the system's own words.
//
// The backend's error strings are written for whoever is debugging the
// device: they name gateway URLs, config file paths, environment
// variables and OS error numbers. Rendering one of those into the
// interface breaks constitution.md's rule that the operating system is
// part of the appliance, not something the owner has to think about --
// and it tells someone at a shop counter nothing they can act on.
//
// So every failure that reaches a screen is translated here, and the raw
// text goes to the log instead. The mapping is by *meaning*, not one
// blanket sentence: "the assistant isn't running" and "the assistant is
// running but rejected us" call for different things from the owner, and
// flattening them would trade one unhelpful message for another.

type Domain = "chat" | "setup" | "cloudKey" | "voice";

/// What the owner sees, and -- where there is one -- what they can do.
const GENERIC: Record<Domain, string> = {
  chat: "I couldn't reach my assistant just now.",
  setup: "I couldn't reach my assistant just now. Setup can continue once it's back.",
  cloudKey: "I couldn't save that just now.",
  voice: "I couldn't speak just now. You can still type to me.",
};

/// Ordered most specific first: the first match wins, so a narrow
/// signature is never shadowed by a broader one.
const PATTERNS: Array<{ match: RegExp; message: string }> = [
  // Speech is optional, so its absence is a state to explain rather than
  // a fault to apologise for -- and it has a fix the owner can carry out.
  {
    match: /no speech key configured/i,
    message: "I need a speech connection before I can talk. You'll find it in Settings.",
  },
  // The recording arrived but there was nothing in it. Said as a fact,
  // not as a failure: a shop counter is a loud room.
  {
    match: /recording was empty|nothing to say|returned no audio/i,
    message: "I didn't catch that.",
  },
  // The gateway is not running, or not listening yet. The most common
  // real failure, and the one with an honest thing to say about it.
  {
    match: /unreachable|connection refused|connect error|timed out|timeout/i,
    message: "My assistant isn't responding yet. It may still be starting up.",
  },
  // Running, but refusing us: a credential problem on the device. The
  // owner cannot fix this, so the message says so rather than implying
  // they should retry forever.
  {
    match: /no hermes api key|unauthorized|401|403|forbidden/i,
    message:
      "My assistant isn't set up correctly on this device. This needs someone to look at it.",
  },
  // The key exists but the provider rejected it.
  {
    match: /invalid api key|invalid_api_key|authentication/i,
    message: "That key wasn't accepted. Check it and try again.",
  },
  // Out of credit or rate limited upstream.
  {
    match: /rate limit|quota|insufficient|payment|429/i,
    message: "My assistant has run out of its allowance for now.",
  },
  // The reply started and then stopped. Worth its own wording: the owner
  // is looking at half an answer and needs to know it is not the whole
  // one, which "I couldn't reach my assistant" would not tell them.
  {
    match: /mid-response|stream failed/i,
    message: "My assistant stopped part-way through that answer.",
  },
  // The assistant is reachable and authorised but broke on its own side.
  {
    match: /\b5\d{2}\b|internal error/i,
    message: "Something went wrong inside my assistant.",
  },
  // The gateway answered with something we couldn't parse.
  {
    match: /invalid json|response failed|could not encode|could not build/i,
    message: "My assistant answered in a way I couldn't understand.",
  },
  // Local preference storage.
  {
    match: /setup store/i,
    message: "I couldn't remember that setting.",
  },
  // The keyring is locked or unavailable.
  {
    match: /keyring|secret service|no such interface/i,
    message: "I couldn't get to this device's secure storage.",
  },
];

export function agentErrorMessage(domain: Domain, raw: unknown): string {
  // Worth keeping in full, but only where an engineer reads it.
  console.error(`[${domain}]`, raw);

  const text =
    raw instanceof Error ? raw.message : typeof raw === "string" ? raw : String(raw);

  const hit = PATTERNS.find((p) => p.match.test(text));
  return hit ? hit.message : GENERIC[domain];
}

/// Anything that went wrong while listening or speaking.
///
/// Voice is the one part of the device that can fail while everything
/// else still works, so its wording always leaves the owner the way
/// round: they can type.
export function voiceErrorMessage(raw: unknown): string {
  return agentErrorMessage("voice", raw);
}

/// Errors that arrive as a stream event rather than a thrown value. Same
/// translation -- the gateway's wording is no more suitable for the owner
/// just because it came over a channel.
export function streamErrorMessage(raw: string): string {
  return agentErrorMessage("chat", raw);
}
