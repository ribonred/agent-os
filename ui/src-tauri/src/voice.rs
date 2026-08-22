//! Ears and mouth: speech-to-text and text-to-speech, over the speech
//! provider's HTTP API.
//!
//! The webview records the audio and plays the answer -- it is the only
//! half of the shell with a microphone and a speaker -- but it never
//! learns the API key and never opens a connection off the device.
//! Audio comes in here as raw IPC bytes and goes back out the same way,
//! exactly as the gateway commands proxy the agent.
//!
//! What this deliberately does NOT use is the provider's own
//! conversational agent product. The device already has a brain, with
//! its own constitution, memory and tools; handing the conversation to
//! a second one would mean two assistants disagreeing about who the
//! owner is. This is transcription and speech, nothing more.

use http_body_util::{BodyExt, Full};
use hyper::body::Bytes;

use cloud_key::Provider;

const API_ROOT: &str = "https://api.elevenlabs.io/v1";

/// The device's voice. One voice, chosen once, the same on every unit --
/// like the persona baseline in the constitution, this is a decision the
/// product makes rather than a question the owner is asked on their
/// first day.
const VOICE_ID: &str = "BZgkqPqms7Kj9ulSkVzn";

/// The expressive multilingual model. The device is a presence in a
/// room, not a notification read aloud, and this is the one choice that
/// decides which of those it sounds like.
///
/// It is not the fastest option, which is why a reply is spoken sentence
/// by sentence as it is written rather than in one piece at the end:
/// each call stays short, and the owner hears the first sentence while
/// the rest is still being thought. If the wait before the first sound
/// ever becomes the complaint, a lower-latency model is the dial to
/// turn, and it is this line.
const SPEECH_MODEL: &str = "eleven_v3";

/// Transcription model. Accuracy over speed: the owner is heard once,
/// and a word got wrong becomes a question the device answers
/// confidently and wrongly.
const LISTEN_MODEL: &str = "scribe_v2";

/// mp3 rather than raw PCM: it is available on every account tier, where
/// the PCM output formats are not, and a device that goes silent because
/// of a billing plan is a bad failure to debug from a shop counter.
const SPEECH_FORMAT: &str = "mp3_44100_128";

/// Long enough for a sentence of any language, short enough that a
/// runaway reply cannot bill the owner for a chapter.
const MAX_SPEAK_CHARS: usize = 800;

fn key() -> Result<String, String> {
    cloud_key::resolve_key(Provider::ElevenLabs)?
        .ok_or_else(|| "no speech key configured".to_string())
}

/// The file extension the provider should see for a recording. It reads
/// the container from the name as well as the bytes, and the webview
/// records in whatever its engine actually supports rather than in a
/// format we get to choose.
fn extension_for(mime: &str) -> &'static str {
    let mime = mime.split(';').next().unwrap_or("").trim();
    match mime {
        "audio/webm" | "video/webm" => "webm",
        "audio/ogg" | "application/ogg" => "ogg",
        "audio/mp4" | "video/mp4" | "audio/x-m4a" => "mp4",
        "audio/mpeg" | "audio/mp3" => "mp3",
        "audio/flac" | "audio/x-flac" => "flac",
        _ => "wav",
    }
}

/// A multipart body with one text field and one file.
///
/// Hand-rolled rather than pulling in a second HTTP stack for a form
/// encoder: this is the only multipart request the shell makes, and the
/// hard part of multipart -- picking a delimiter that cannot occur in
/// the payload -- is three lines.
fn multipart(fields: &[(&str, &str)], file: (&str, &str, &[u8])) -> (String, Vec<u8>) {
    let mut boundary = String::from("----agenticos-boundary-0e8f2a17");
    // A boundary that appears inside the audio would split the file in
    // half and the provider would read the tail as a new part. Vanishing
    // odds, but the failure would be a corrupted upload that looks like
    // a bad recording, so it is worth the scan.
    while file.2.windows(boundary.len()).any(|w| w == boundary.as_bytes()) {
        boundary.push('x');
    }

    let mut body: Vec<u8> = Vec::with_capacity(file.2.len() + 512);
    for (name, value) in fields {
        body.extend_from_slice(
            format!(
                "--{boundary}\r\nContent-Disposition: form-data; name=\"{name}\"\r\n\r\n{value}\r\n"
            )
            .as_bytes(),
        );
    }
    let (field, filename, bytes) = file;
    body.extend_from_slice(
        format!(
            "--{boundary}\r\nContent-Disposition: form-data; name=\"{field}\"; \
             filename=\"{filename}\"\r\nContent-Type: application/octet-stream\r\n\r\n"
        )
        .as_bytes(),
    );
    body.extend_from_slice(bytes);
    body.extend_from_slice(format!("\r\n--{boundary}--\r\n").as_bytes());

    (boundary, body)
}

/// Turns a provider failure into something the error table upstairs can
/// recognise. The raw body goes to the log and never to a screen.
fn refused(what: &str, status: hyper::StatusCode, body: &[u8]) -> String {
    let detail = String::from_utf8_lossy(body);
    log::error!("speech provider refused {what} ({status}): {detail}");
    format!("speech provider refused {what} ({status})")
}

/// What the owner just said, as text.
///
/// Takes the recording as a raw IPC body rather than a JSON field: a
/// few seconds of audio encoded as a number array is several times the
/// size and has to be rebuilt byte by byte on this side.
#[tauri::command]
pub async fn voice_transcribe(
    request: tauri::ipc::Request<'_>,
    app: tauri::AppHandle,
) -> Result<String, String> {
    let tauri::ipc::InvokeBody::Raw(audio) = request.body() else {
        return Err("expected the recording as raw bytes".to_string());
    };
    if audio.is_empty() {
        return Err("the recording was empty".to_string());
    }

    let mime = request
        .headers()
        .get("x-audio-mime")
        .and_then(|value| value.to_str().ok())
        .unwrap_or("audio/webm");

    let mut fields: Vec<(&str, &str)> = vec![("model_id", LISTEN_MODEL)];
    // The owner's own language, so a Bahasa question is not transcribed
    // as approximate English. Left to the provider to detect only when
    // setup never established one.
    let language = crate::onboarding::language_from_store(&app);
    if let Some(code) = language.as_deref() {
        fields.push(("language_code", code));
    }

    let filename = format!("speech.{}", extension_for(mime));
    let (boundary, body) = multipart(&fields, ("file", &filename, audio));

    let http_request = hyper::Request::post(format!("{API_ROOT}/speech-to-text"))
        .header("xi-api-key", key()?)
        .header("accept", "application/json")
        .header(
            "content-type",
            format!("multipart/form-data; boundary={boundary}"),
        )
        .body(Full::new(Bytes::from(body)))
        .map_err(|e| format!("could not build the transcription request: {e}"))?;

    let response = crate::http::https_client()
        .request(http_request)
        .await
        .map_err(|e| format!("speech provider unreachable: {e}"))?;
    let status = response.status();
    let body = response
        .into_body()
        .collect()
        .await
        .map_err(|e| format!("transcription response failed: {e}"))?
        .to_bytes();

    if !status.is_success() {
        return Err(refused("the recording", status, &body));
    }

    let parsed: serde_json::Value = serde_json::from_slice(&body)
        .map_err(|e| format!("speech provider sent invalid JSON: {e}"))?;
    Ok(parsed["text"].as_str().unwrap_or_default().trim().to_string())
}

/// One sentence, spoken. Returns the audio for the webview to play.
///
/// One call per sentence rather than one per reply, and non-streaming
/// within a call: at a sentence's length the streaming endpoint saves
/// nothing worth the plumbing, and a finished clip is something the
/// player can queue and cancel cleanly.
#[tauri::command]
pub async fn voice_speak(text: String) -> Result<tauri::ipc::Response, String> {
    let text = text.trim();
    if text.is_empty() {
        return Err("nothing to say".to_string());
    }
    let text: String = text.chars().take(MAX_SPEAK_CHARS).collect();

    let payload = serde_json::json!({
        "text": text,
        "model_id": SPEECH_MODEL,
    });

    let http_request = hyper::Request::post(format!(
        "{API_ROOT}/text-to-speech/{VOICE_ID}?output_format={SPEECH_FORMAT}"
    ))
    .header("xi-api-key", key()?)
    .header("content-type", "application/json")
    .header("accept", "audio/mpeg")
    .body(Full::new(Bytes::from(
        serde_json::to_vec(&payload).map_err(|e| format!("could not encode the request: {e}"))?,
    )))
    .map_err(|e| format!("could not build the speech request: {e}"))?;

    let response = crate::http::https_client()
        .request(http_request)
        .await
        .map_err(|e| format!("speech provider unreachable: {e}"))?;
    let status = response.status();
    let body = response
        .into_body()
        .collect()
        .await
        .map_err(|e| format!("speech response failed: {e}"))?
        .to_bytes();

    if !status.is_success() {
        return Err(refused("the sentence", status, &body));
    }
    if body.is_empty() {
        return Err("speech provider returned no audio".to_string());
    }

    Ok(tauri::ipc::Response::new(body.to_vec()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extension_follows_the_container_the_engine_chose() {
        assert_eq!(extension_for("audio/webm;codecs=opus"), "webm");
        assert_eq!(extension_for("audio/ogg; codecs=opus"), "ogg");
        assert_eq!(extension_for("audio/mp4"), "mp4");
        // Anything unrecognised is treated as wav, which is what the
        // fallback capture path produces.
        assert_eq!(extension_for(""), "wav");
        assert_eq!(extension_for("audio/wav"), "wav");
    }

    #[test]
    fn multipart_carries_the_fields_and_the_file() {
        let (boundary, body) = multipart(
            &[("model_id", "scribe_v1"), ("language_code", "id")],
            ("file", "speech.webm", b"AUDIOBYTES"),
        );
        let text = String::from_utf8_lossy(&body);
        assert!(text.contains("name=\"model_id\"\r\n\r\nscribe_v1"));
        assert!(text.contains("name=\"language_code\"\r\n\r\nid"));
        assert!(text.contains("filename=\"speech.webm\""));
        assert!(text.contains("AUDIOBYTES"));
        assert!(text.ends_with(&format!("--{boundary}--\r\n")));
    }

    /// A delimiter that occurs inside the audio would end the file part
    /// early, and the upload would look like a bad recording rather than
    /// a bad encoding.
    #[test]
    fn a_boundary_occurring_in_the_audio_is_moved_out_of_the_way() {
        let collision = b"----agenticos-boundary-0e8f2a17";
        let (boundary, body) = multipart(&[], ("file", "speech.wav", collision));
        assert_ne!(boundary, "----agenticos-boundary-0e8f2a17");
        assert_eq!(
            body.windows(boundary.len())
                .filter(|w| *w == boundary.as_bytes())
                .count(),
            2,
            "the delimiter must appear only as the two real delimiters"
        );
    }
}
