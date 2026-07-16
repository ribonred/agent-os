mod agent;
mod cloud_key;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        // Wired in from the start, not added after something breaks --
        // LogDir gives a persistent file to actually read instead of
        // guessing when something goes wrong; Stdout/Webview mirror the
        // same logs to the terminal and devtools console during dev.
        .plugin(
            tauri_plugin_log::Builder::new()
                .targets([
                    tauri_plugin_log::Target::new(tauri_plugin_log::TargetKind::Stdout),
                    tauri_plugin_log::Target::new(tauri_plugin_log::TargetKind::LogDir {
                        file_name: None,
                    }),
                    tauri_plugin_log::Target::new(tauri_plugin_log::TargetKind::Webview),
                ])
                .build(),
        )
        // Non-secret preferences only (language, persona) -- per
        // design/DESIGN.md, the OpenRouter API key needs OS-keyring-backed
        // storage instead, not this plain-file store.
        .plugin(tauri_plugin_store::Builder::default().build())
        .invoke_handler(tauri::generate_handler![
            greet,
            cloud_key::cloud_key_save,
            cloud_key::cloud_key_status,
            cloud_key::cloud_key_delete,
            agent::agent_status,
            agent::agent_chat
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
