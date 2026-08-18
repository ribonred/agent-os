// What the device did, in the owner's language.
//
// The runtime names its tools for the people who wrote them --
// "web_extract", "code_execution" -- and constitution.md forbids putting
// that vocabulary in front of the owner. This is the same translation
// job lib/agentErrors.ts does for failures: one table, applied at the
// edge, so no surface has to remember to do it.
//
// An unknown tool gets a true but unspecific sentence rather than its
// raw name. New tools appear when the runtime is upgraded, and a
// device that leaks "acp_bridge" onto the screen the day that happens
// is worse than one that says it did something.

const SUMMARIES: Record<string, string> = {
  read_file: "Read one of your files",
  write_file: "Saved a file",
  patch: "Edited a file",
  search_files: "Looked through your files",
  terminal: "Ran something on this device",
  process: "Checked what this device is running",
  web_search: "Searched the web",
  web_extract: "Read a page from the web",
  browser: "Used the browser",
  code_execution: "Worked something out",
  vision: "Looked at a picture",
  image_gen: "Made a picture",
  memory: "Made a note to remember",
  clarify: "Asked you something",
};

export function toolSummary(name: string): string {
  const known = SUMMARIES[name];
  if (known) return known;
  // Anything browser-shaped is still the browser, whichever of the
  // dozen browser tools it was.
  if (name.startsWith("browser")) return SUMMARIES.browser!;
  return "Did something on this device";
}
