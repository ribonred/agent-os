mod agent;
mod cloud_key;
mod shelf;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // On the device's bare kiosk compositor there is no settings daemon,
    // so GTK's screen resolution stays at its -1 "unknown" sentinel and
    // WebKitGTK turns it into an automatic page scale of -1/96 -- a
    // negative near-zero devicePixelRatio that destroys the entire
    // layout. Diagnosed live on a VM install by reading the broken
    // devicePixelRatio off the device. The fix is three-part and all
    // parts are required (verified by removing them one at a time):
    // pin the resolution before tauri builds any webview, nudge
    // gtk-xft-dpi afterwards so WebKit's settings proxy re-reads it,
    // and the frontend pins webview zoom to 1.0 on mount (app.vue).
    #[cfg(target_os = "linux")]
    if gtk::init().is_ok() {
        if let Some(display) = gtk::gdk::Display::default() {
            display.default_screen().set_resolution(96.0);
        }
    }

    tauri::Builder::default()
        .setup(|_app| {
            #[cfg(target_os = "linux")]
            {
                use gtk::glib::object::ObjectExt;
                if let Some(display) = gtk::gdk::Display::default() {
                    display.default_screen().set_resolution(96.0);
                }
                // Nudge to a different value first so the change
                // notification actually fires even if the property
                // already held the target value.
                if let Some(settings) = gtk::Settings::default() {
                    settings.set_property("gtk-xft-dpi", 96 * 1024 + 1);
                    settings.set_property("gtk-xft-dpi", 96 * 1024);
                }
            }
            Ok(())
        })
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
        .manage(agent::AgentSession::default())
        .invoke_handler(tauri::generate_handler![
            greet,
            cloud_key::cloud_key_save,
            cloud_key::cloud_key_status,
            cloud_key::cloud_key_delete,
            agent::agent_status,
            agent::agent_chat,
            agent::agent_onboarding_chat,
            shelf::shelf_list
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
