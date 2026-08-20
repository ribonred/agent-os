//! Which model the device thinks with, and letting the owner change it.
//!
//! There is exactly one answer to "which model", and it lives in the
//! agent runtime's `config.yaml`. The shell used to carry its own copy
//! as a literal, with a comment saying the two had to be kept in step.
//! They were not: every session the shell opened was stamped with the
//! shell's copy while the runtime's own default went unused for weeks,
//! and nothing anywhere reported the disagreement. So the copy is gone
//! and this reads the runtime's file -- and when that file does not name
//! a model, this is an error rather than a fallback. A fallback is what
//! made the first drift invisible.
//!
//! The catalogue is Hermes' own. `GET /api/model/options` exists for
//! precisely this: its docstring says it is there "so external clients
//! using the API server can sync to the user's configured Hermes provider
//! catalog instead of scraping the single OpenAI-compatible /v1/models
//! alias." It returns models Hermes has already filtered to the ones an
//! agent can actually drive -- which matters, because a model that cannot
//! call tools would leave a device that chats but cannot read a file,
//! open a folder or build a view.

use http_body_util::{BodyExt, Full};
use hyper::body::Bytes;
use serde::Serialize;

use crate::agent::{api_key, base_url, http_client, stored_session, CHAT_SESSION_KEY};
use crate::hermes_config::{read_key, write_key};

const BLOCK: &str = "model";
const KEY: &str = "default";

/// The aggregator whose own catalogue is wider than the runtime's.
const AGGREGATOR: &str = "openrouter";
const AGGREGATOR_CATALOGUE: &str = "https://openrouter.ai/api/v1/models";

/// The aggregator's catalogue, kept for the life of the process.
///
/// It is a few hundred entries that change every few days, and the owner
/// opens this screen far more often than that. `refresh` clears it.
static CATALOGUE: std::sync::Mutex<Option<Vec<String>>> = std::sync::Mutex::new(None);

/// Every model the aggregator serves that can call tools.
///
/// Hermes' own inventory comes from a curated catalogue that covers a
/// fraction of what the aggregator actually serves -- 36 models against
/// 348, and for some makers a single entry where the aggregator offers
/// fifteen. Both lists are worth having: the runtime's is what it knows
/// how to route, and this is what the owner can actually reach.
///
/// The tool filter is the same rule Hermes applies to its own list, for
/// the same reason: a model that cannot call tools leaves a device that
/// chats but cannot read a file, open a folder or build a view.
///
/// A device with no network gets `None` and simply keeps the runtime's
/// list. Nothing here is allowed to fail a screen.
async fn aggregator_catalogue(refresh: bool) -> Option<Vec<String>> {
    if refresh {
        if let Ok(mut held) = CATALOGUE.lock() {
            *held = None;
        }
    } else if let Ok(held) = CATALOGUE.lock() {
        if let Some(cached) = held.as_ref() {
            return Some(cached.clone());
        }
    }

    let tls = hyper_rustls::HttpsConnectorBuilder::new()
        .with_webpki_roots()
        .https_only()
        .enable_http1()
        .build();
    let client: hyper_util::client::legacy::Client<_, Full<Bytes>> =
        hyper_util::client::legacy::Client::builder(hyper_util::rt::TokioExecutor::new())
            .build(tls);

    let request = hyper::Request::get(AGGREGATOR_CATALOGUE)
        .header("accept", "application/json")
        .body(Full::new(Bytes::new()))
        .ok()?;

    let response = client.request(request).await.ok()?;
    if !response.status().is_success() {
        return None;
    }
    let body = response.into_body().collect().await.ok()?.to_bytes();
    let payload: serde_json::Value = serde_json::from_slice(&body).ok()?;

    let ids: Vec<String> = payload
        .get("data")?
        .as_array()?
        .iter()
        .filter(|model| {
            model
                .get("supported_parameters")
                .and_then(|v| v.as_array())
                .is_some_and(|params| params.iter().any(|p| p.as_str() == Some("tools")))
        })
        .filter_map(|model| model.get("id")?.as_str().map(str::to_string))
        .collect();

    if ids.is_empty() {
        return None;
    }
    if let Ok(mut held) = CATALOGUE.lock() {
        *held = Some(ids.clone());
    }
    Some(ids)
}

/// The runtime's list for this provider, widened by the aggregator's own.
///
/// A union rather than a replacement: the runtime may know a route the
/// catalogue does not name, and losing one would be worse than the
/// duplication -- which there is none of, because ids are exact.
fn widen(existing: Vec<String>, catalogue: &[String]) -> Vec<String> {
    let mut seen: std::collections::HashSet<&str> =
        existing.iter().map(String::as_str).collect();
    let mut out = existing.clone();
    for id in catalogue {
        if seen.insert(id.as_str()) {
            out.push(id.clone());
        }
    }
    out.sort();
    out
}

/// The model the device is set to think with, and what to call it.
///
/// Deliberately cheap: one file read, no gateway and no network. The
/// label beside the composer asks this on every start, and a label has no
/// business waiting on the internet to render -- the full inventory is
/// fetched only when the owner actually opens the picker.
pub(crate) fn configured_model() -> Result<String, String> {
    read_key(BLOCK, KEY)
}

#[tauri::command]
pub fn model_current() -> Result<CurrentModel, String> {
    let id = configured_model()?;
    Ok(CurrentModel {
        name: model_name(&id),
        id,
    })
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CurrentModel {
    pub id: String,
    pub name: String,
}

/// The maker a model belongs to, from the segment before the slash.
///
/// Two things have to be normalised or one maker ends up under two
/// headings. The aggregator publishes floating aliases with a leading
/// tilde -- `~z-ai/glm-latest` is the same lab as `z-ai/glm-5.3` -- and
/// it carries both `meta` and `meta-llama` for Meta. The id keeps
/// whatever spelling it has; only the grouping is canonical.
fn canonical_family(raw: &str) -> String {
    let slug = raw.trim_start_matches('~').to_lowercase();
    match slug.as_str() {
        "meta-llama" => "meta".to_string(),
        other => other.to_string(),
    }
}

/// Providers whose name is not simply their slug capitalised. Only the
/// ones that actually look wrong are listed; anything else is title-cased
/// from its slug, so a provider added upstream tomorrow still reads
/// properly without shipping a new build.
fn family_name(slug: &str) -> String {
    match slug {
        "openai" => "OpenAI".into(),
        "x-ai" => "xAI".into(),
        "z-ai" => "Z.AI".into(),
        "deepseek" => "DeepSeek".into(),
        "meta-llama" | "meta" => "Meta".into(),
        "mistralai" => "Mistral".into(),
        "moonshotai" => "Moonshot".into(),
        "nvidia" => "NVIDIA".into(),
        "stepfun" => "StepFun".into(),
        "minimax" => "MiniMax".into(),
        "bytedance-seed" => "ByteDance".into(),
        "ai21" => "AI21".into(),
        "hugging-face" => "Hugging Face".into(),
        other => other
            .split(['-', '_'])
            .map(|word| {
                let mut chars = word.chars();
                match chars.next() {
                    Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                    None => String::new(),
                }
            })
            .collect::<Vec<_>>()
            .join(" "),
    }
}

/// A readable name for one model, from its id. `anthropic/claude-opus-5`
/// reads as "Claude Opus 5"; the family is shown once as a heading rather
/// than repeated on every row under it.
fn model_name(id: &str) -> String {
    let tail = id.rsplit('/').next().unwrap_or(id);
    // A variant suffix is part of what the owner is choosing -- ":free"
    // and ":batch" behave differently -- so it is kept, set apart.
    let (base, variant) = match tail.split_once(':') {
        Some((base, variant)) => (base, Some(variant)),
        None => (tail, None),
    };
    // Split on dashes and underscores only. A dot is part of a version
    // number, and splitting there turned "claude-opus-4.8" into
    // "Claude Opus 4 8" -- which reads as two numbers and is not what
    // the owner would be choosing between.
    let words = base
        .split(['-', '_'])
        .map(|word| match word.to_lowercase().as_str() {
            // The handful that look wrong title-cased. Anything else is
            // capitalised, so a model released next month still reads
            // properly without a new build.
            "gpt" => "GPT".to_string(),
            "glm" => "GLM".to_string(),
            "qwq" => "QwQ".to_string(),
            "hy" => "HY".to_string(),
            _ => {
                let mut chars = word.chars();
                match chars.next() {
                    Some(first) if first.is_ascii_alphabetic() => {
                        first.to_uppercase().collect::<String>() + chars.as_str()
                    }
                    _ => word.to_string(),
                }
            }
        })
        .collect::<Vec<_>>()
        .join(" ");
    match variant {
        Some(variant) => format!("{words} ({variant})"),
        None => words,
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelEntry {
    /// What the runtime is told, exactly as it must be spelled.
    pub id: String,
    pub name: String,
    /// The family the id belongs to, and its heading.
    pub family: String,
    pub family_name: String,
    pub fast: bool,
    pub reasoning: bool,
    /// Hermes' own shortlist. Shown first, because 37 models is a list
    /// and six is a choice.
    pub featured: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderGroup {
    pub slug: String,
    pub name: String,
    pub is_current: bool,
    /// Whether the device holds a key for it. An unauthenticated provider
    /// is still shown -- "this needs a key" is a better answer for the
    /// owner than a silently shorter list.
    pub authenticated: bool,
    pub models: Vec<ModelEntry>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelOptions {
    /// What `config.yaml` names right now.
    pub current: String,
    pub providers: Vec<ProviderGroup>,
}

/// Turn Hermes' inventory payload into what the picker renders.
fn parse_options(
    payload: &serde_json::Value,
    current: String,
    catalogue: Option<&[String]>,
) -> ModelOptions {
    let featured_of = |provider: &serde_json::Value| -> Vec<String> {
        provider
            .get("featured_models")
            .and_then(|v| v.as_array())
            .map(|items| {
                items
                    .iter()
                    .filter_map(|i| i.as_str().map(str::to_string))
                    .collect()
            })
            .unwrap_or_default()
    };

    let mut providers: Vec<ProviderGroup> = payload
        .get("providers")
        .and_then(|v| v.as_array())
        .map(|items| items.as_slice())
        .unwrap_or(&[])
        .iter()
        .filter_map(|provider| {
            let mut ids: Vec<String> = provider
                .get("models")
                .and_then(|v| v.as_array())
                .map(|items| {
                    items
                        .iter()
                        .filter_map(|i| i.as_str().map(str::to_string))
                        .collect()
                })
                .unwrap_or_default();

            let slug_now = provider.get("slug").and_then(|v| v.as_str()).unwrap_or("");
            if slug_now == AGGREGATOR {
                if let Some(catalogue) = catalogue {
                    ids = widen(ids, catalogue);
                }
            }
            // A provider offering nothing is not a choice; it is a row
            // the owner can only be disappointed by.
            if ids.is_empty() {
                return None;
            }

            let featured = featured_of(provider);
            let capabilities = provider.get("capabilities");
            let flag = |id: &str, name: &str| -> bool {
                capabilities
                    .and_then(|c| c.get(id))
                    .and_then(|c| c.get(name))
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false)
            };

            let slug = provider
                .get("slug")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string();

            let models = ids
                .into_iter()
                .map(|id| {
                    // An aggregator names the maker in the id
                    // ("anthropic/claude-opus-5"); a provider the device
                    // talks to directly does not ("claude-fable-5"),
                    // because there is only one maker it could be. Read
                    // off the real gateway, where Anthropic's own models
                    // arrive bare and OpenRouter's arrive prefixed.
                    let family = match id.split_once('/') {
                        Some((family, _)) => canonical_family(family),
                        None => canonical_family(&slug),
                    };
                    ModelEntry {
                        name: model_name(&id),
                        family_name: family_name(&family),
                        fast: flag(&id, "fast"),
                        reasoning: flag(&id, "reasoning"),
                        featured: featured.contains(&id),
                        family,
                        id,
                    }
                })
                .collect();

            Some(ProviderGroup {
                slug,
                name: provider
                    .get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or_default()
                    .to_string(),
                is_current: provider
                    .get("is_current")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false),
                authenticated: provider
                    .get("authenticated")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false),
                models,
            })
        })
        .collect();

    // The provider in use first, then the ones the device can actually
    // reach, then the rest.
    providers.sort_by_key(|p| (!p.is_current, !p.authenticated, p.name.to_lowercase()));
    ModelOptions { current, providers }
}

/// Every model the device could think with, grouped the way Hermes
/// groups them.
#[tauri::command]
pub async fn model_options(refresh: Option<bool>) -> Result<ModelOptions, String> {
    let key = api_key()?;
    let current = configured_model()?;
    let refresh = refresh.unwrap_or(false);

    let uri: hyper::Uri = format!("{}/api/model/options?refresh={refresh}", base_url())
        .parse()
        .map_err(|e| format!("bad gateway URL: {e}"))?;
    let request = hyper::Request::get(uri)
        .header("authorization", format!("Bearer {key}"))
        .body(Full::new(Bytes::new()))
        .map_err(|e| format!("could not build the model request: {e}"))?;

    let response = http_client()
        .request(request)
        .await
        .map_err(|e| format!("agent gateway unreachable: {e}"))?;
    let status = response.status();
    let body = response
        .into_body()
        .collect()
        .await
        .map_err(|e| format!("model list response failed: {e}"))?
        .to_bytes();
    if !status.is_success() {
        return Err(format!("agent gateway refused the model list ({status})"));
    }
    let payload: serde_json::Value =
        serde_json::from_slice(&body).map_err(|e| format!("invalid model list JSON: {e}"))?;

    let catalogue = aggregator_catalogue(refresh).await;
    Ok(parse_options(&payload, current, catalogue.as_deref()))
}

/// Ask the gateway to lock the conversation in progress onto a model.
///
/// This is how Hermes itself changes model mid-conversation, so it is how
/// the shell does it: the owner's answer arrives from the model they just
/// chose, rather than the change quietly applying to some later
/// conversation they have to go and start.
async fn lock_session(key: &str, session_id: &str, id: &str) -> Result<(), String> {
    let uri: hyper::Uri = format!("{}/api/sessions/{session_id}/model", base_url())
        .parse()
        .map_err(|e| format!("bad gateway URL: {e}"))?;
    let payload = serde_json::json!({ "model": id }).to_string();
    let request = hyper::Request::post(uri)
        .header("authorization", format!("Bearer {key}"))
        .header("content-type", "application/json")
        .body(Full::new(Bytes::from(payload)))
        .map_err(|e| format!("could not build the model request: {e}"))?;

    let response = http_client()
        .request(request)
        .await
        .map_err(|e| format!("agent gateway unreachable: {e}"))?;
    let status = response.status();
    if !status.is_success() {
        return Err(format!("the gateway would not switch this conversation ({status})"));
    }
    Ok(())
}

/// Set the model the device thinks with.
///
/// Two writes, in this order. The config is the durable answer and every
/// new conversation reads it, so it goes first: if the second fails the
/// device is still correctly set, just not for the conversation already
/// open. The reverse order could leave one conversation on a model the
/// device does not believe it uses.
#[tauri::command]
pub async fn model_set(app: tauri::AppHandle, id: String) -> Result<(), String> {
    let id = id.trim().to_string();
    if id.is_empty() {
        return Err("no model was chosen".to_string());
    }

    // Only ids the gateway just offered. The webview is not trusted to
    // name a model, and a typo here would be written into the runtime's
    // own config and break every turn afterwards.
    let options = model_options(Some(false)).await?;
    let known = options
        .providers
        .iter()
        .any(|p| p.models.iter().any(|m| m.id == id));
    if !known {
        return Err("that isn't a model this device can use".to_string());
    }

    write_key(BLOCK, KEY, &id)?;

    if let Some(session_id) = stored_session(&app, CHAT_SESSION_KEY) {
        let key = api_key()?;
        if let Err(error) = lock_session(&key, &session_id, &id).await {
            // Said in the log, not to the owner: the choice was saved and
            // takes effect, and the only casualty is that it starts on
            // their next question rather than this one.
            log::warn!("could not switch the open conversation: {error}");
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_model_id_reads_as_a_name() {
        assert_eq!(model_name("anthropic/claude-opus-5"), "Claude Opus 5");
        assert_eq!(model_name("z-ai/glm-5.3"), "GLM 5.3");
        assert_eq!(model_name("openai/gpt-5.6-sol-pro"), "GPT 5.6 Sol Pro");
        // A version number is one number. Splitting on the dot produced
        // "Claude Opus 4 8", which reads as two.
        assert_eq!(model_name("anthropic/claude-opus-4.8"), "Claude Opus 4.8");
        // A variant changes what you get, so it stays visible.
        assert_eq!(
            model_name("nvidia/nemotron-3-ultra-550b-a55b:free"),
            "Nemotron 3 Ultra 550b A55b (free)"
        );
        // A bare id from a direct provider reads the same way.
        assert_eq!(model_name("claude-fable-5"), "Claude Fable 5");
    }

    #[test]
    fn a_direct_providers_bare_id_still_gets_a_family() {
        // Anthropic returns "claude-fable-5", not "anthropic/claude-fable-5".
        // Found against the real gateway; the fixture had not imagined it.
        let payload: serde_json::Value = serde_json::from_str(PAYLOAD).expect("valid");
        let options = parse_options(&payload, "z-ai/glm-5.3".to_string(), None);
        let anthropic = options
            .providers
            .iter()
            .find(|p| p.slug == "anthropic")
            .expect("anthropic");
        let bare = anthropic
            .models
            .iter()
            .find(|m| m.id == "claude-fable-5")
            .expect("the bare id");
        assert_eq!(bare.family, "anthropic");
        assert_eq!(bare.family_name, "Anthropic");
        assert_eq!(bare.name, "Claude Fable 5");
    }

    #[test]
    fn one_maker_never_gets_two_headings() {
        // The aggregator publishes floating aliases with a tilde and
        // carries Meta under two slugs. Left alone, the picker grew
        // headings for "~z Ai" and "~openai" beside the real ones.
        assert_eq!(canonical_family("~z-ai"), "z-ai");
        assert_eq!(canonical_family("~anthropic"), "anthropic");
        assert_eq!(canonical_family("meta-llama"), "meta");
        assert_eq!(canonical_family("meta"), "meta");
        assert_eq!(canonical_family("X-AI"), "x-ai");
        // And the tilde form reads as the lab it belongs to.
        assert_eq!(family_name(&canonical_family("~z-ai")), "Z.AI");
        assert_eq!(family_name(&canonical_family("~x-ai")), "xAI");
    }

    #[test]
    fn a_family_reads_as_its_makers_name() {
        assert_eq!(family_name("openai"), "OpenAI");
        assert_eq!(family_name("x-ai"), "xAI");
        assert_eq!(family_name("z-ai"), "Z.AI");
        assert_eq!(family_name("deepseek"), "DeepSeek");
        // Anything not listed is still presentable rather than raw.
        assert_eq!(family_name("anthropic"), "Anthropic");
        assert_eq!(family_name("moonshotai"), "Moonshot");
        assert_eq!(family_name("some-new-lab"), "Some New Lab");
    }

    /// Shaped exactly like the live gateway's reply, which was read off
    /// the running device rather than guessed at.
    const PAYLOAD: &str = r#"{
        "model": "z-ai/glm-5.3",
        "provider": "openrouter",
        "providers": [
          {"slug": "empty", "name": "Nothing Here", "models": [], "is_current": false,
           "authenticated": false},
          {"slug": "anthropic", "name": "Anthropic", "is_current": false,
           "authenticated": true, "models": ["anthropic/claude-opus-5", "claude-fable-5"],
           "featured_models": [], "capabilities": {}},
          {"slug": "openrouter", "name": "OpenRouter", "is_current": true,
           "authenticated": true,
           "models": ["z-ai/glm-5.3", "x-ai/grok-4.6", "openai/gpt-5.5",
                      "~z-ai/glm-latest", "meta-llama/llama-4", "meta/muse-1"],
           "featured_models": ["z-ai/glm-5.3"],
           "capabilities": {"x-ai/grok-4.6": {"fast": true, "reasoning": true}}}
        ]}"#;

    #[test]
    fn the_inventory_becomes_something_groupable() {
        let payload: serde_json::Value = serde_json::from_str(PAYLOAD).expect("valid");
        let options = parse_options(&payload, "z-ai/glm-5.3".to_string(), None);

        // A provider with no models is not a row the owner can use.
        assert_eq!(options.providers.len(), 2);
        // The one in use comes first.
        assert_eq!(options.providers[0].slug, "openrouter");
        assert!(options.providers[0].is_current);

        let models = &options.providers[0].models;
        assert_eq!(models.len(), 6);
        // Families come off the id, which is what the owner asked to
        // group by -- one provider, several makers under it.
        let families: Vec<&str> = models.iter().map(|m| m.family_name.as_str()).collect();
        assert_eq!(
            families,
            vec!["Z.AI", "xAI", "OpenAI", "Z.AI", "Meta", "Meta"]
        );
        // Same heading, same colour key -- one maker, one group.
        let zai: Vec<&str> = models
            .iter()
            .filter(|m| m.family_name == "Z.AI")
            .map(|m| m.family.as_str())
            .collect();
        assert_eq!(zai, vec!["z-ai", "z-ai"]);

        let grok = models.iter().find(|m| m.id == "x-ai/grok-4.6").unwrap();
        assert!(grok.fast && grok.reasoning);
        let glm = models.iter().find(|m| m.id == "z-ai/glm-5.3").unwrap();
        assert!(glm.featured);
        // Absent capability data is absent, never invented.
        assert!(!glm.fast);
    }

    /// Against the running gateway on this machine, whose inventory is
    /// shaped by the owner's own configured providers -- not something a
    /// fixture can guess at. Reads only; changes nothing.
    /// Opt-in: cargo test -- --ignored
    #[tokio::test]
    #[ignore]
    async fn the_real_gateway_offers_models_this_can_group() {
        let options = model_options(Some(false)).await.expect("no inventory");
        assert!(!options.current.is_empty(), "config names no model");
        assert!(!options.providers.is_empty(), "no provider offers anything");

        for provider in &options.providers {
            assert!(!provider.models.is_empty(), "{} is an empty row", provider.slug);
            for model in &provider.models {
                assert!(!model.id.is_empty());
                assert!(!model.name.is_empty(), "{} has no name", model.id);
                // Every model belongs under a heading, whether its id
                // named the maker or the provider had to stand in.
                assert!(!model.family_name.is_empty(), "{} has no family", model.id);
            }
        }

        // The model the config names must be one the owner could pick,
        // or the picker opens with nothing selected.
        let offered = options
            .providers
            .iter()
            .any(|p| p.models.iter().any(|m| m.id == options.current));
        assert!(offered, "{} is not in the inventory", options.current);
    }

    #[test]
    fn the_aggregators_row_is_widened_and_nothing_else_is() {
        // The runtime's catalogue knows 36 models where the aggregator
        // serves 348 -- one Qwen against forty-nine, one Grok against
        // five. Both lists matter, so this is a union.
        let payload: serde_json::Value = serde_json::from_str(PAYLOAD).expect("valid");
        let catalogue = vec![
            "qwen/qwen3.8-27b".to_string(),
            "x-ai/grok-4.5".to_string(),
            // Already in the runtime's list: it must not appear twice.
            "z-ai/glm-5.3".to_string(),
        ];
        let options = parse_options(&payload, "z-ai/glm-5.3".to_string(), Some(&catalogue));

        let aggregator = options
            .providers
            .iter()
            .find(|p| p.slug == "openrouter")
            .expect("openrouter");
        let ids: Vec<&str> = aggregator.models.iter().map(|m| m.id.as_str()).collect();
        assert!(ids.contains(&"qwen/qwen3.8-27b"));
        assert!(ids.contains(&"x-ai/grok-4.5"));
        // The runtime's own entries survive.
        assert!(ids.contains(&"openai/gpt-5.5"));
        assert_eq!(ids.iter().filter(|i| **i == "z-ai/glm-5.3").count(), 1);

        // A direct provider is left exactly as the runtime described it:
        // the catalogue describes the aggregator and nothing else.
        let direct = options
            .providers
            .iter()
            .find(|p| p.slug == "anthropic")
            .expect("anthropic");
        assert_eq!(direct.models.len(), 2);
    }

    #[test]
    fn an_offline_device_still_gets_the_runtimes_list() {
        // Nothing about reading a public catalogue is allowed to fail a
        // screen: a device may never see a network at all.
        let payload: serde_json::Value = serde_json::from_str(PAYLOAD).expect("valid");
        let options = parse_options(&payload, "z-ai/glm-5.3".to_string(), None);
        let aggregator = options
            .providers
            .iter()
            .find(|p| p.slug == "openrouter")
            .expect("openrouter");
        assert_eq!(aggregator.models.len(), 6);
    }

    #[test]
    fn the_current_model_is_the_configs_and_not_the_payloads() {
        // The payload carries a `model` too. The config is the answer;
        // trusting the payload here would reintroduce the second source
        // of truth this module exists to remove.
        let payload: serde_json::Value = serde_json::from_str(PAYLOAD).expect("valid");
        let options = parse_options(&payload, "anthropic/claude-opus-5".to_string(), None);
        assert_eq!(options.current, "anthropic/claude-opus-5");
    }
}
