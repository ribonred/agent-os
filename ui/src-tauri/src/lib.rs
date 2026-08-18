mod agent;
mod approval_mode;
mod cloud_key;
mod dev;
mod shelf;
mod window_mode;

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
        .setup(|app| {
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

            // The device opens in whatever shape it was last left in.
            // Applied here rather than from the frontend so the window is
            // the right size and place before anything is painted --
            // starting full and shrinking to a pill a moment later is a
            // visible flinch on every launch.
            let mode = window_mode::current(app.handle());
            if let Err(error) = window_mode::apply(app.handle(), mode, mode) {
                // Not fatal: a window that ignored one geometry call is
                // still a usable window, and the owner can switch modes.
                log::warn!("could not restore the window mode: {error}");
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
            cloud_key::cloud_key_save,
            cloud_key::cloud_key_status,
            cloud_key::cloud_key_delete,
            agent::agent_status,
            agent::agent_chat,
            agent::agent_onboarding_chat,
            agent::agent_approve,
            agent::agent_stop,
            approval_mode::approval_mode_get,
            approval_mode::approval_mode_set,
            window_mode::window_mode_get,
            window_mode::window_mode_set,
            window_mode::window_pill_expand,
            window_mode::window_drag,
            window_mode::window_toggle_maximize,
            window_mode::window_resize_drag,
            dev::dev_reset_setup,
            shelf::shelf_list
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
