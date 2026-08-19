//! Shell-owned onboarding state machine.
//!
//! Progress is not left to the model. The shell keeps a fixed checklist,
//! records each answer, and only asks Hermes to phrase the current open
//! step. Device service checks run here once, silently, before the owner
//! is greeted.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};
use tauri_plugin_store::StoreExt;

pub const STATE_KEY: &str = "onboardingState";

const STEP_ORDER: &[StepId] = &[
    StepId::OwnerName,
    StepId::Role,
    StepId::Needs,
    StepId::Vocabulary,
    StepId::Boundaries,
    StepId::Communication,
    StepId::Confirm,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StepId {
    OwnerName,
    Role,
    Needs,
    Vocabulary,
    Boundaries,
    Communication,
    Confirm,
}

impl StepId {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::OwnerName => "owner_name",
            Self::Role => "role",
            Self::Needs => "needs",
            Self::Vocabulary => "vocabulary",
            Self::Boundaries => "boundaries",
            Self::Communication => "communication",
            Self::Confirm => "confirm",
        }
    }

    fn title(self) -> &'static str {
        match self {
            Self::OwnerName => "what to call the owner",
            Self::Role => "role and context",
            Self::Needs => "concrete needs",
            Self::Vocabulary => "vocabulary and important entities",
            Self::Boundaries => "boundaries and sensitivities",
            Self::Communication => "communication preference",
            Self::Confirm => "summary confirmation",
        }
    }

    fn ask_hint(self) -> &'static str {
        match self {
            Self::OwnerName => {
                "Introduce yourself by the owner-given name, ask what they would like you to call them, and stop. One question only."
            }
            Self::Role => {
                "Ask one short question about who they are and what this device is for. Prefer yes/no when it fits."
            }
            Self::Needs => {
                "Ask one short question about the specific tasks they want help with. Prefer yes/no when it fits."
            }
            Self::Vocabulary => {
                "Ask one concrete question about the words or records that matter in their work (what they call customers, files, day-to-day terms). One thing only."
            }
            Self::Boundaries => {
                "Ask one yes/no question about anything sensitive, off-limits, or needing extra care."
            }
            Self::Communication => {
                "Ask one short question about how they want to be talked to (brief vs detailed, formal vs casual)."
            }
            Self::Confirm => {
                "Summarize every known fact below in plain language, including what to call them. Ask whether that summary is correct. Do not ask any new discovery question. End with tappable answers using exactly: <options>Yes|No</options>"
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StepStatus {
    Open,
    Done,
    Skipped,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Step {
    pub id: StepId,
    pub status: StepStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub answer: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DeviceChecks {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub postgres: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub redis: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub checked_at: Option<String>,
    #[serde(default)]
    pub attempted: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OnboardingState {
    pub version: u32,
    pub steps: Vec<Step>,
    pub current: StepId,
    #[serde(default)]
    pub device_checks: DeviceChecks,
    #[serde(default)]
    pub profile_written: bool,
}

impl OnboardingState {
    pub fn fresh() -> Self {
        Self {
            version: 1,
            steps: STEP_ORDER
                .iter()
                .map(|&id| Step {
                    id,
                    status: StepStatus::Open,
                    answer: None,
                })
                .collect(),
            current: StepId::OwnerName,
            device_checks: DeviceChecks::default(),
            profile_written: false,
        }
    }

    pub fn step(&self, id: StepId) -> Option<&Step> {
        self.steps.iter().find(|step| step.id == id)
    }

    fn step_mut(&mut self, id: StepId) -> Option<&mut Step> {
        self.steps.iter_mut().find(|step| step.id == id)
    }

    pub fn done_count(&self) -> u8 {
        self.steps
            .iter()
            .filter(|step| {
                step.id != StepId::Confirm
                    && matches!(step.status, StepStatus::Done | StepStatus::Skipped)
            })
            .count() as u8
    }

    pub fn known_facts(&self) -> Vec<(StepId, String)> {
        self.steps
            .iter()
            .filter(|step| step.id != StepId::Confirm)
            .filter_map(|step| {
                let answer = step.answer.as_ref()?.trim();
                if answer.is_empty() {
                    None
                } else {
                    Some((step.id, answer.to_string()))
                }
            })
            .collect()
    }

    /// Record the owner's reply for the current step and advance.
    pub fn apply_owner_reply(&mut self, raw: &str) -> ApplyOutcome {
        let answer = normalize_answer(raw);
        if answer.is_empty() {
            return ApplyOutcome::NeedQuestion;
        }

        match self.current {
            StepId::Confirm => {
                if is_acceptance(&answer) {
                    if let Some(step) = self.step_mut(StepId::Confirm) {
                        step.answer = Some(answer);
                        step.status = StepStatus::Done;
                    }
                    ApplyOutcome::Accepted
                } else if is_rejection(&answer) {
                    if let Some(step) = self.step_mut(StepId::Confirm) {
                        step.answer = Some(answer);
                        step.status = StepStatus::Open;
                    }
                    ApplyOutcome::NeedQuestion
                } else {
                    if let Some(step) = self.step_mut(StepId::Confirm) {
                        step.answer = Some(answer);
                        step.status = StepStatus::Open;
                    }
                    ApplyOutcome::NeedQuestion
                }
            }
            step_id => {
                if let Some(step) = self.step_mut(step_id) {
                    if matches!(step.status, StepStatus::Done | StepStatus::Skipped)
                        && step.answer.is_some()
                    {
                        // locked
                    } else {
                        step.answer = Some(answer);
                        step.status = StepStatus::Done;
                    }
                }
                self.current = self.next_open_after(step_id).unwrap_or(StepId::Confirm);
                if self.current == StepId::Confirm {
                    if let Some(step) = self.step_mut(StepId::Confirm) {
                        step.status = StepStatus::Open;
                    }
                }
                ApplyOutcome::NeedQuestion
            }
        }
    }

    fn next_open_after(&self, after: StepId) -> Option<StepId> {
        let mut seen = false;
        for &id in STEP_ORDER {
            if id == after {
                seen = true;
                continue;
            }
            if !seen {
                continue;
            }
            let status = self
                .step(id)
                .map(|step| step.status)
                .unwrap_or(StepStatus::Open);
            if matches!(status, StepStatus::Open) || id == StepId::Confirm {
                return Some(id);
            }
        }
        Some(StepId::Confirm)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApplyOutcome {
    NeedQuestion,
    Accepted,
}

fn normalize_answer(raw: &str) -> String {
    raw.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn is_acceptance(text: &str) -> bool {
    let t = text.trim().to_lowercase();
    matches!(
        t.as_str(),
        "yes"
            | "y"
            | "yeah"
            | "yep"
            | "ok"
            | "okay"
            | "correct"
            | "right"
            | "sure"
            | "confirm"
            | "confirmed"
            | "looks good"
            | "good"
            | "perfect"
            | "ya"
            | "iya"
            | "iyo"
            | "benar"
            | "betul"
            | "setuju"
            | "sip"
            | "oke"
            | "okey"
            | "sudah benar"
            | "sudah oke"
            | "that's right"
            | "thats right"
            | "all good"
    ) || t.starts_with("yes ")
        || t.starts_with("ya ")
        || t.starts_with("iya ")
}

fn is_rejection(text: &str) -> bool {
    let t = text.trim().to_lowercase();
    matches!(
        t.as_str(),
        "no" | "n" | "nope" | "wrong" | "fix" | "tidak" | "salah" | "belum" | "change"
            | "edit"
    ) || t.starts_with("no ")
        || t.starts_with("tidak ")
}

pub fn load_state(app: &tauri::AppHandle) -> OnboardingState {
    let Ok(store) = app.store("settings.json") else {
        return OnboardingState::fresh();
    };
    store
        .get(STATE_KEY)
        .and_then(|value| serde_json::from_value(value).ok())
        .unwrap_or_else(OnboardingState::fresh)
}

pub fn save_state(app: &tauri::AppHandle, state: &OnboardingState) -> Result<(), String> {
    let store = app
        .store("settings.json")
        .map_err(|e| format!("could not open setup store: {e}"))?;
    let value = serde_json::to_value(state).map_err(|e| format!("encode onboarding state: {e}"))?;
    store.set(STATE_KEY, value);
    store
        .save()
        .map_err(|e| format!("could not save setup store: {e}"))
}

pub fn mark_setup_complete(app: &tauri::AppHandle) -> Result<(), String> {
    let store = app
        .store("settings.json")
        .map_err(|e| format!("could not open setup store: {e}"))?;
    store.set("onboardingComplete", true);
    store.set(
        "onboardingQuestionCount",
        serde_json::json!(STEP_ORDER.len().saturating_sub(1)),
    );
    store
        .save()
        .map_err(|e| format!("could not save setup store: {e}"))
}

pub fn hermes_home() -> PathBuf {
    if let Ok(value) = std::env::var("HERMES_HOME") {
        let path = PathBuf::from(value);
        if !path.as_os_str().is_empty() {
            return path;
        }
    }
    dirs_home()
        .map(|home| home.join(".hermes"))
        .unwrap_or_else(|| PathBuf::from(".hermes"))
}

fn dirs_home() -> Option<PathBuf> {
    std::env::var_os("HOME").map(PathBuf::from)
}

fn utc_stamp() -> String {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    format!("unix:{secs}")
}

/// Live Postgres/Redis checks. Never blocks the owner on failure; a miss
/// is recorded as attempted and left empty.
pub fn ensure_device_checks(state: &mut OnboardingState) {
    if state.device_checks.attempted {
        return;
    }
    state.device_checks.attempted = true;
    state.device_checks.checked_at = Some(utc_stamp());

    if let Some(version) = probe_postgres() {
        state.device_checks.postgres = Some(version);
    }
    if let Some(version) = probe_redis() {
        state.device_checks.redis = Some(version);
    }

    if let Err(error) = write_device_memory(&state.device_checks) {
        log::warn!("could not write device checks to Hermes memory: {error}");
    }
}

fn probe_postgres() -> Option<String> {
    let output = Command::new("sudo")
        .args([
            "-n",
            "-u",
            "postgres",
            "psql",
            "-XAtqc",
            "SELECT current_setting('server_version')",
        ])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
    (!text.is_empty()).then_some(text)
}

fn probe_redis() -> Option<String> {
    let ping = Command::new("redis-cli")
        .args(["--raw", "PING"])
        .output()
        .ok()?;
    if !ping.status.success() {
        return None;
    }
    if String::from_utf8_lossy(&ping.stdout).trim() != "PONG" {
        return None;
    }
    let info = Command::new("redis-cli")
        .args(["--raw", "INFO", "server"])
        .output()
        .ok()?;
    if !info.status.success() {
        return None;
    }
    let body = String::from_utf8_lossy(&info.stdout);
    body.lines().find_map(|line| {
        line.strip_prefix("redis_version:")
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
    })
}

fn write_device_memory(checks: &DeviceChecks) -> Result<(), String> {
    let mut parts = Vec::new();
    if let Some(pg) = checks.postgres.as_deref() {
        parts.push(format!("PostgreSQL {pg}"));
    }
    if let Some(redis) = checks.redis.as_deref() {
        parts.push(format!("Redis {redis} (PING=PONG)"));
    }
    if parts.is_empty() {
        return Ok(());
    }
    let stamp = checks.checked_at.as_deref().unwrap_or("unknown");
    let line = format!(
        "Device services verified live by the shell: {} (checked {stamp}).",
        parts.join(", ")
    );

    let home = hermes_home();
    let memories = home.join("memories");
    std::fs::create_dir_all(&memories).map_err(|e| format!("create memories dir: {e}"))?;
    let path = memories.join("MEMORY.md");
    let existing = std::fs::read_to_string(&path).unwrap_or_default();
    let rewritten = upsert_memory_line(&existing, "Device services verified live", &line);
    std::fs::write(&path, rewritten).map_err(|e| format!("write MEMORY.md: {e}"))?;
    Ok(())
}

fn upsert_memory_line(existing: &str, needle: &str, line: &str) -> String {
    let mut kept = Vec::new();
    for entry in existing.lines() {
        if entry.contains(needle) {
            continue;
        }
        if !entry.trim().is_empty() {
            kept.push(entry.to_string());
        }
    }
    kept.push(line.to_string());
    let mut out = kept.join("\n");
    if !out.ends_with('\n') {
        out.push('\n');
    }
    out
}

pub fn compose_user_profile(state: &OnboardingState, agent_name: Option<&str>) -> String {
    let mut lines = Vec::new();
    lines.push("# Owner profile".to_string());
    lines.push(String::new());
    lines.push(
        "Written by the device shell after the owner confirmed onboarding. \
         Facts below are owner-given; unresolved means not yet known."
            .to_string(),
    );
    lines.push(String::new());
    if let Some(name) = agent_name.map(str::trim).filter(|value| !value.is_empty()) {
        lines.push(format!("- Assistant name: {name}"));
    }

    let label = |id: StepId| -> &'static str {
        match id {
            StepId::OwnerName => "Call the owner",
            StepId::Role => "Role and context",
            StepId::Needs => "Concrete needs",
            StepId::Vocabulary => "Vocabulary and entities",
            StepId::Boundaries => "Boundaries",
            StepId::Communication => "Communication preference",
            StepId::Confirm => "Confirmation note",
        }
    };

    for &id in STEP_ORDER {
        if id == StepId::Confirm {
            continue;
        }
        let value = state
            .step(id)
            .and_then(|step| step.answer.as_deref())
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .unwrap_or("not yet known");
        lines.push(format!("- {}: {value}", label(id)));
    }
    lines.push(String::new());
    lines.join("\n")
}

pub fn write_user_profile(
    state: &OnboardingState,
    agent_name: Option<&str>,
) -> Result<PathBuf, String> {
    let home = hermes_home();
    std::fs::create_dir_all(&home).map_err(|e| format!("create hermes home: {e}"))?;
    let path = home.join("USER.md");
    let body = compose_user_profile(state, agent_name);
    std::fs::write(&path, body).map_err(|e| format!("write USER.md: {e}"))?;
    Ok(path)
}

/// Per-turn instructions: only the current step, plus locked facts.
pub fn step_overlay(state: &OnboardingState) -> String {
    let mut parts = Vec::new();
    parts.push(
        "You are in device onboarding. The shell owns the checklist. \
         You only phrase the current step. Do not run tools, do not load \
         skills, do not check services, do not search sessions, and do not \
         write memory unless the shell has already finished and this is \
         ordinary chat."
            .to_string(),
    );
    parts.push(
        "Send exactly one question per reply, then stop. End at the \
         question mark. Nothing follows it."
            .to_string(),
    );

    let facts = state.known_facts();
    if !facts.is_empty() {
        let mut block = String::from("Already known (do not re-ask):\n");
        for (id, answer) in facts {
            block.push_str(&format!("- {}: {}\n", id.title(), answer));
        }
        parts.push(block);
    }

    let current = state.current;
    parts.push(format!(
        "Current step: {} ({}).\n{}",
        current.as_str(),
        current.title(),
        current.ask_hint()
    ));

    if current != StepId::OwnerName {
        parts.push(
            "The name question is done if listed above. Never ask what to \
             call the owner again."
                .to_string(),
        );
    }

    if current == StepId::Confirm {
        if let Some(note) = state
            .step(StepId::Confirm)
            .and_then(|step| step.answer.as_ref())
        {
            parts.push(format!(
                "The owner replied to the summary with: \"{note}\". \
                 Revise the summary if needed, then ask for confirmation again."
            ));
        }
    }

    parts.join("\n\n")
}

pub fn turn_user_message(state: &OnboardingState, _owner_input: Option<&str>) -> String {
    match state.current {
        StepId::OwnerName => {
            "Begin onboarding now. Introduce yourself by the owner-given name, then ask what you should call them, and stop. One question only."
                .to_string()
        }
        StepId::Confirm => {
            "Continue onboarding. Summarize the known facts and ask the owner to confirm. One question only. End with <options>Yes|No</options>."
                .to_string()
        }
        other => format!(
            "Continue onboarding. Ask only the current open step ({}). One question only.",
            other.title()
        ),
    }
}

pub fn agent_name_from_store(app: &tauri::AppHandle) -> Option<String> {
    let store = app.store("settings.json").ok()?;
    store
        .get("agentName")
        .and_then(|value| value.as_str().map(str::to_string))
        .map(|name| name.trim().to_string())
        .filter(|name| !name.is_empty())
}

pub fn language_from_store(app: &tauri::AppHandle) -> Option<String> {
    let store = app.store("settings.json").ok()?;
    store
        .get("language")
        .and_then(|value| value.as_str().map(str::to_string))
        .map(|code| code.trim().to_string())
        .filter(|code| !code.is_empty())
}

/// Deterministic closing line after the owner accepts. The shell owns this
/// text so finish is never a blank turn or an improvised goodbye.
pub fn completion_message(
    state: &OnboardingState,
    agent_name: Option<&str>,
    language: Option<&str>,
) -> String {
    let owner = state
        .step(StepId::OwnerName)
        .and_then(|step| step.answer.as_deref())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .filter(|value| !is_declined_name(value));
    let agent = agent_name
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("I");

    let id = language.map(str::trim).unwrap_or("en");
    match id {
        "id" => match owner {
            Some(name) => format!(
                "Siap, {name}. Setup sudah selesai — saya {agent}, dan saya siap membantu."
            ),
            None => {
                format!("Siap. Setup sudah selesai — saya {agent}, dan saya siap membantu.")
            }
        },
        _ => match owner {
            Some(name) => format!(
                "All set, {name}. Setup is complete — I'm {agent}, and I'm ready to help."
            ),
            None => {
                format!("All set. Setup is complete — I'm {agent}, and I'm ready to help.")
            }
        },
    }
}

fn is_declined_name(value: &str) -> bool {
    let t = value.trim().to_lowercase();
    matches!(
        t.as_str(),
        "you"
            | "skip"
            | "no"
            | "nope"
            | "none"
            | "shared"
            | "tidak"
            | "skip saja"
            | "no name"
            | "don't"
            | "dont"
    ) || t.contains("shared")
        || t.contains("no name")
        || t.contains("jangan")
}

/// Prompt shape if a future path asks Hermes to echo the shell closing
/// line. Production finish streams `completion_message` directly.
#[cfg(test)]
pub fn completion_overlay(message: &str) -> String {
    format!(
        "Onboarding is finished. The shell already saved the owner profile.\n\
         Reply with EXACTLY the following text and nothing else — no tools, \
         no questions, no extra sentences, no options trailer:\n\n{message}"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fresh_state_starts_at_owner_name() {
        let state = OnboardingState::fresh();
        assert_eq!(state.current, StepId::OwnerName);
        assert_eq!(state.steps.len(), STEP_ORDER.len());
        assert_eq!(state.done_count(), 0);
    }

    #[test]
    fn name_locks_and_advances() {
        let mut state = OnboardingState::fresh();
        let outcome = state.apply_owner_reply("saya john");
        assert_eq!(outcome, ApplyOutcome::NeedQuestion);
        assert_eq!(
            state.step(StepId::OwnerName).unwrap().answer.as_deref(),
            Some("saya john")
        );
        assert_eq!(
            state.step(StepId::OwnerName).unwrap().status,
            StepStatus::Done
        );
        assert_eq!(state.current, StepId::Role);
        assert_eq!(state.done_count(), 1);
    }

    #[test]
    fn full_path_reaches_confirm_then_accept() {
        let mut state = OnboardingState::fresh();
        for answer in [
            "John",
            "school staff",
            "index school documents",
            "regulations, events, lessons",
            "no special boundaries",
            "brief and direct",
        ] {
            assert_eq!(state.apply_owner_reply(answer), ApplyOutcome::NeedQuestion);
        }
        assert_eq!(state.current, StepId::Confirm);
        assert_eq!(state.done_count(), 6);
        assert_eq!(state.apply_owner_reply("yes"), ApplyOutcome::Accepted);
        assert_eq!(
            state.step(StepId::Confirm).unwrap().status,
            StepStatus::Done
        );
    }

    #[test]
    fn profile_lists_unresolved_when_thin() {
        let mut state = OnboardingState::fresh();
        state.apply_owner_reply("John");
        let profile = compose_user_profile(&state, Some("Brian"));
        assert!(profile.contains("Assistant name: Brian"));
        assert!(profile.contains("Call the owner: John"));
        assert!(profile.contains("Role and context: not yet known"));
    }

    #[test]
    fn overlay_forbids_tools_and_names_current_step() {
        let mut state = OnboardingState::fresh();
        state.apply_owner_reply("John");
        let overlay = step_overlay(&state);
        assert!(
            overlay.to_lowercase().contains("do not run tools"),
            "overlay should forbid tools: {overlay}"
        );
        assert!(overlay.contains("Current step: role"));
        assert!(overlay.contains("John"));
        assert!(overlay.contains("Never ask what to call the owner again"));
    }

    #[test]
    fn memory_line_is_upserted() {
        let existing = "Other fact stays.\nDevice services verified live by the shell: old.\n";
        let next = upsert_memory_line(
            existing,
            "Device services verified live",
            "Device services verified live by the shell: new.",
        );
        assert!(next.contains("Other fact stays."));
        assert!(next.contains("Device services verified live by the shell: new."));
        assert!(!next.contains(": old."));
    }

    #[test]
    fn acceptance_words_cover_id_and_en() {
        assert!(is_acceptance("ya"));
        assert!(is_acceptance("Yes"));
        assert!(is_acceptance("benar"));
        assert!(!is_acceptance("school staff"));
    }

    #[test]
    fn completion_message_is_deterministic() {
        let mut state = OnboardingState::fresh();
        state.apply_owner_reply("John");
        let en = completion_message(&state, Some("Brian"), Some("en"));
        assert_eq!(
            en,
            "All set, John. Setup is complete — I'm Brian, and I'm ready to help."
        );
        let id = completion_message(&state, Some("Brian"), Some("id"));
        assert_eq!(
            id,
            "Siap, John. Setup sudah selesai — saya Brian, dan saya siap membantu."
        );
        let overlay = completion_overlay(&en);
        assert!(overlay.contains("EXACTLY the following text"));
        assert!(overlay.contains(&en));
    }
}
