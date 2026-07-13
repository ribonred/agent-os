# agentic-os

Base OS + agent runtime for MSI-bundled AI-assistant devices (NixOS track).
This is commercial product code that ships to customers, not a personal
dev project — treat every file as something a customer's device could
expose or a future team member could read with no shared context.

## Commercial-code hygiene (non-negotiable)

- **No personal names, usernames, or identifying info** anywhere in code,
  config, or comments. System accounts get generic names (`admin`, not a
  developer's username). No email addresses, no "my machine" references.
- **No references to internal tooling/process by name** in code comments
  — e.g. don't cite an internal task-tracker's ticket IDs, or name an
  internal engineering-process/skill doc as the source of a rule. If a
  rule matters enough to justify in a comment, restate the reasoning
  itself in plain terms so it stands on its own to an outside reader.
- **Explaining the "why" is encouraged, not discouraged.** The rule above
  is about who/what gets named, not about removing rationale. A comment
  like "unix socket only -- nothing on this box needs network pg yet" is
  exactly right. A comment like "see red-mindset" or "tracked as task 002"
  is not, because it leaks internal context and won't mean anything once
  this ships.
- Project-management artifacts (the `tasks/` directory, taskmd) are
  internal and not part of what ships — task IDs are fine *within that
  directory*, just don't bleed them into source comments elsewhere.

Apply this on every edit, not just when asked.
