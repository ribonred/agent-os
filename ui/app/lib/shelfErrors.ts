// The owner never reads the system's own words. Every failure that
// reaches a screen is written for them; the raw string goes to the log.
//
// This exists because passing an error straight through is the path of
// least resistance and leaks exactly what the device should not surface.

export function shelfErrorMessage(raw: unknown): string {
  // Worth keeping, but only where an engineer reads it.
  console.error("[files]", raw);

  // The Rust side already phrases its failures for the owner ("I can't
  // open that"). Anything else -- a panic, a serialisation fault -- gets
  // the generic line rather than its own text.
  const text = typeof raw === "string" ? raw : String(raw);
  const known = ["I can't open that", "I can't find that", "that isn't somewhere I can look"];
  const match = known.find((k) => text.includes(k));
  return match ? match.charAt(0).toUpperCase() + match.slice(1) : "I can't open that.";
}

/// How the owner would say when something changed -- not a timestamp.
export function relativeDate(ms: number, now = Date.now()): string {
  if (!Number.isFinite(ms) || ms <= 0) return "";

  const then = new Date(ms);
  const startOfToday = new Date(now);
  startOfToday.setHours(0, 0, 0, 0);
  const startOfThen = new Date(then);
  startOfThen.setHours(0, 0, 0, 0);

  const days = Math.round(
    (startOfToday.getTime() - startOfThen.getTime()) / 86_400_000,
  );

  if (days <= 0) return "Today";
  if (days === 1) return "Yesterday";
  if (days < 7) return `${days} days ago`;

  const sameYear = then.getFullYear() === new Date(now).getFullYear();
  return then.toLocaleDateString(undefined, {
    day: "numeric",
    month: "short",
    ...(sameYear ? {} : { year: "numeric" }),
  });
}

/// Sizes in units a person uses. Directories say what they hold instead.
export function fileSize(bytes: number): string {
  if (!Number.isFinite(bytes) || bytes <= 0) return "";
  const units = ["B", "KB", "MB", "GB", "TB"];
  let value = bytes;
  let unit = 0;
  while (value >= 1024 && unit < units.length - 1) {
    value /= 1024;
    unit += 1;
  }
  // No decimal on bytes and KB: "1.4 KB" is noise, "1 KB" is enough.
  const rounded = unit >= 2 ? value.toFixed(1) : Math.round(value).toString();
  return `${rounded} ${units[unit]}`;
}

export function itemCount(count: number): string {
  if (count === 0) return "empty";
  return count === 1 ? "1 item" : `${count} items`;
}
