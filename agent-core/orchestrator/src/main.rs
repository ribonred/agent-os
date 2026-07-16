//! The agent orchestrator daemon: owns the routing decision (local Ollama
//! vs cloud OpenRouter) and fronts the LLM behind a Unix-socket HTTP API.
//!
//! Process shape, decided deliberately (see tasks/008): a standalone
//! daemon under systemd, NOT part of the UI process. The agent must
//! outlive the window -- ingest jobs and background work continue if the
//! UI restarts -- and the same daemon ships unchanged to the DGX tier
//! where the UI may differ. The UI is a thin client on the socket.
//!
//! Unix socket, not a TCP port: nothing on a customer device should
//! listen on the network for this. The socket file is chmod 0600, so
//! only the device user talks to it.
//!
//! API:
//!   GET  /status -> routing inputs and decision, inspectable at any time
//!   POST /chat   -> {"messages":[{"role","content"},...]} streamed back
//!                   as ndjson: {"type":"token","content":...} lines,
//!                   then {"type":"done","backend":...,"model":...}
//!
//! The constitution (brain/constitution.md) is the system prompt, loaded
//! at startup. Refusing to start without it is deliberate: a daemon that
//! can't find its behavior spec must not improvise one. Client-supplied
//! "system" messages are rejected for the same reason.

mod llm;

use std::os::unix::fs::PermissionsExt;
use std::sync::Arc;

use axum::body::Body;
use axum::extract::State;
use axum::http::{StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use tokio::net::UnixListener;
use tokio::sync::mpsc;

use llm::{ChatMessage, OllamaClient, OpenRouterClient, StreamEvent};

fn env_or(name: &str, default: &str) -> String {
    std::env::var(name).unwrap_or_else(|_| default.to_string())
}

struct Config {
    socket_path: String,
    constitution_path: String,
    ollama_url: String,
    openrouter_url: String,
    local_model: String,
    cloud_model: String,
}

impl Config {
    fn from_env() -> Self {
        Config {
            socket_path: env_or("AGENTIC_OS_SOCKET", "/run/agentic-os/orchestrator.sock"),
            constitution_path: env_or(
                "AGENTIC_OS_CONSTITUTION",
                "/etc/agentic-os/constitution.md",
            ),
            ollama_url: env_or("AGENTIC_OS_OLLAMA_URL", "http://127.0.0.1:11434"),
            openrouter_url: env_or("AGENTIC_OS_OPENROUTER_URL", "https://openrouter.ai/api/v1"),
            local_model: env_or("AGENTIC_OS_LOCAL_MODEL", "hermes3:3b"),
            // Same model family as the local tier on purpose -- routing
            // flips must not change the assistant's personality. Keep in
            // lockstep with DEFAULT_OPENROUTER_MODEL in
            // agent-core/ingest/extraction.py, which encodes the same
            // product decision for extraction calls.
            cloud_model: env_or("AGENTIC_OS_CLOUD_MODEL", "nousresearch/hermes-4-70b"),
        }
    }
}

struct AppState {
    config: Config,
    constitution: String,
    // Probed once at startup -- hardware doesn't change under a running
    // daemon. Connectivity and credentials are checked per request; they
    // do change.
    profile: hw_probe::HwProfile,
    tier: hw_probe::Tier,
    http: reqwest::Client,
}

#[derive(Serialize)]
struct StatusResponse {
    profile: hw_probe::HwProfile,
    tier: hw_probe::Tier,
    online: bool,
    key_source: Option<&'static str>,
    default_routing: hw_probe::RoutingLean,
    backend_model: String,
}

async fn status(State(state): State<Arc<AppState>>) -> Response {
    let online = hw_probe::is_online();
    let key_source = match cloud_key::key_status() {
        Ok(source) => source,
        Err(e) => {
            log::error!("credential check failed during status: {e}");
            return (StatusCode::INTERNAL_SERVER_ERROR, e).into_response();
        }
    };
    let routing = hw_probe::decide_default_routing(
        &state.tier,
        online,
        key_source.is_some(),
        false, // no vertical config exists yet to force offline
    );
    let backend_model = match routing {
        hw_probe::RoutingLean::Local => state.config.local_model.clone(),
        hw_probe::RoutingLean::Cloud => state.config.cloud_model.clone(),
    };

    Json(StatusResponse {
        profile: state.profile.clone(),
        tier: state.tier.clone(),
        online,
        key_source: key_source.map(|s| match s {
            cloud_key::KeySource::Keyring => "keyring",
            cloud_key::KeySource::Provisioned => "provisioned",
        }),
        default_routing: routing,
        backend_model,
    })
    .into_response()
}

#[derive(Deserialize)]
struct ChatRequest {
    messages: Vec<ChatMessage>,
}

#[derive(Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum ChatStreamLine<'a> {
    Token { content: &'a str },
    Done { backend: &'a str, model: &'a str },
    Error { message: &'a str },
}

fn ndjson_line(line: &ChatStreamLine) -> Vec<u8> {
    let mut bytes = serde_json::to_vec(line).expect("stream line is always serializable");
    bytes.push(b'\n');
    bytes
}

async fn chat(State(state): State<Arc<AppState>>, Json(req): Json<ChatRequest>) -> Response {
    if req.messages.is_empty() {
        return (StatusCode::BAD_REQUEST, "messages must not be empty").into_response();
    }
    // The constitution is the only system prompt this device runs. A
    // client trying to supply its own is either a bug or an attempt to
    // override shipped behavior -- refuse loudly either way.
    if req.messages.iter().any(|m| m.role == "system") {
        return (
            StatusCode::BAD_REQUEST,
            "system messages are set by the device, not the client",
        )
            .into_response();
    }

    // Route this request: cached tier + live connectivity + live key.
    let online = hw_probe::is_online();
    let key = match cloud_key::resolve_openrouter_key() {
        Ok(key) => key,
        Err(e) => {
            log::error!("credential check failed during chat: {e}");
            return (StatusCode::INTERNAL_SERVER_ERROR, e).into_response();
        }
    };
    let routing = hw_probe::decide_default_routing(&state.tier, online, key.is_some(), false);
    let backend = match routing {
        hw_probe::RoutingLean::Local => "local",
        hw_probe::RoutingLean::Cloud => "cloud",
    };
    // Every routing decision is inspectable, with its inputs -- never a
    // silent internal choice.
    log::info!(
        "chat routed {backend}: tier={:?} online={online} has_key={}",
        state.tier,
        key.is_some()
    );

    let mut messages = vec![ChatMessage {
        role: "system".to_string(),
        content: state.constitution.clone(),
    }];
    messages.extend(req.messages);

    let (tx, rx) = mpsc::channel::<Result<StreamEvent, String>>(32);
    let state_for_task = state.clone();
    tokio::spawn(async move {
        let result = match routing {
            hw_probe::RoutingLean::Local => {
                let client = OllamaClient {
                    base_url: state_for_task.config.ollama_url.clone(),
                    model: state_for_task.config.local_model.clone(),
                };
                client.chat(&state_for_task.http, &messages, &tx).await
            }
            hw_probe::RoutingLean::Cloud => {
                let client = OpenRouterClient {
                    base_url: state_for_task.config.openrouter_url.clone(),
                    model: state_for_task.config.cloud_model.clone(),
                };
                // Routing chose cloud, so a key exists today -- but a
                // changed precondition should surface as an error event,
                // not a daemon panic.
                match key {
                    Some(key) => {
                        client
                            .chat(&state_for_task.http, &key, &messages, &tx)
                            .await
                    }
                    None => Err("routing chose cloud but no key resolved".to_string()),
                }
            }
        };
        if let Err(e) = result {
            // No silent fallback to the other backend: the user asked a
            // question and the chosen path failed -- say so. Automatic
            // local<->cloud failover is a product decision to make
            // deliberately, not a default baked in here.
            log::error!("chat via {backend} failed: {e}");
            let _ = tx.send(Err(e)).await;
        }
    });

    let model = match routing {
        hw_probe::RoutingLean::Local => state.config.local_model.clone(),
        hw_probe::RoutingLean::Cloud => state.config.cloud_model.clone(),
    };
    let stream = futures_util::stream::unfold(
        (rx, false, backend, model),
        |(mut rx, finished, backend, model)| async move {
            if finished {
                return None;
            }
            match rx.recv().await {
                Some(Ok(StreamEvent::Token(content))) => Some((
                    Ok::<_, std::convert::Infallible>(ndjson_line(&ChatStreamLine::Token {
                        content: &content,
                    })),
                    (rx, false, backend, model),
                )),
                Some(Ok(StreamEvent::Done)) => Some((
                    Ok(ndjson_line(&ChatStreamLine::Done {
                        backend,
                        model: &model,
                    })),
                    (rx, true, backend, model),
                )),
                Some(Err(message)) => Some((
                    Ok(ndjson_line(&ChatStreamLine::Error { message: &message })),
                    (rx, true, backend, model),
                )),
                None => None,
            }
        },
    );

    (
        [(header::CONTENT_TYPE, "application/x-ndjson")],
        Body::from_stream(stream),
    )
        .into_response()
}

#[tokio::main]
async fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    let config = Config::from_env();

    // No constitution, no daemon. This file defines how the shipped agent
    // behaves; running without it would mean improvising behavior.
    let constitution = match std::fs::read_to_string(&config.constitution_path) {
        Ok(text) if !text.trim().is_empty() => text,
        Ok(_) => {
            eprintln!(
                "constitution at {} is empty -- refusing to start",
                config.constitution_path
            );
            std::process::exit(1);
        }
        Err(e) => {
            eprintln!(
                "cannot read constitution at {}: {e} -- refusing to start (set AGENTIC_OS_CONSTITUTION)",
                config.constitution_path
            );
            std::process::exit(1);
        }
    };

    let profile = hw_probe::probe_hardware();
    let tier = hw_probe::classify_tier(&profile);
    log::info!(
        "probed: cores={} mem={:.1}GiB gpus={:?} npu={} -> tier={:?}",
        profile.logical_cores,
        profile.total_memory_gib,
        profile.gpu_vendors,
        profile.npu_present,
        tier
    );

    // Bind the socket. A stale file from an unclean shutdown would make
    // bind fail; removing it first is safe because only this daemon owns
    // the path.
    let socket_path = config.socket_path.clone();
    if let Some(parent) = std::path::Path::new(&socket_path).parent() {
        std::fs::create_dir_all(parent).expect("socket directory must be creatable");
    }
    let _ = std::fs::remove_file(&socket_path);
    let listener = UnixListener::bind(&socket_path)
        .unwrap_or_else(|e| panic!("cannot bind {socket_path}: {e}"));
    std::fs::set_permissions(&socket_path, std::fs::Permissions::from_mode(0o600))
        .expect("socket permissions must be settable");
    log::info!("listening on {socket_path}");

    let state = Arc::new(AppState {
        config,
        constitution,
        profile,
        tier,
        http: reqwest::Client::new(),
    });

    let app = Router::new()
        .route("/status", get(status))
        .route("/chat", post(chat))
        .with_state(state);

    axum::serve(listener, app)
        .with_graceful_shutdown(async {
            let _ = tokio::signal::ctrl_c().await;
        })
        .await
        .expect("server failed");
}
