mod agent;
mod approval_mode;
mod cloud_key;
mod dev;
mod hermes_config;
mod http;
mod model;
mod onboarding;
mod sessions;
mod shelf;
mod views;
mod voice;
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
        // Views are served to a sandboxed frame over their own scheme
        // rather than read into the shell's document as text. The page
        // is model-authored, and this keeps it in an opaque origin with
        // no path back to the command bridge. On Linux the webview
        // addresses this as `view://localhost/<path>`.
        .register_uri_scheme_protocol("view", |_ctx, request| views::serve(&request))
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

            // The microphone, which is off twice over in this engine.
            //
            // WebKitGTK ships with media-stream support disabled, and
            // separately denies any permission the embedder does not
            // answer -- so a page calling getUserMedia gets a refusal
            // that no amount of frontend code can undo. Both switches
            // are here, and neither is a prompt: the owner already said
            // yes by holding down a talk button on a device they bought
            // to talk to, and a second dialog asking them to confirm it
            // is the operating system leaking through the appliance.
            //
            // Deliberately narrow. Only a request for a capture device
            // is allowed, and only on the shell's own window; a model-
            // authored page runs in a `view://` frame, is a separate
            // origin, and falls through to WebKitGTK's own denial.
            #[cfg(target_os = "linux")]
            if let Some(window) = tauri::Manager::get_webview_window(app, "main") {
                if let Err(error) = window.with_webview(|webview| {
                    use webkit2gtk::glib::prelude::Cast;
                    use webkit2gtk::{
                        PermissionRequestExt, SettingsExt, UserMediaPermissionRequest,
                        UserMediaPermissionRequestExt, WebViewExt,
                    };

                    let view = webview.inner();
                    if let Some(settings) = WebViewExt::settings(&view) {
                        settings.set_enable_media_stream(true);
                    }
                    view.connect_permission_request(|_, request| {
                        let Some(media) = request.downcast_ref::<UserMediaPermissionRequest>()
                        else {
                            return false;
                        };
                        // The shell records sound and nothing else. A
                        // camera request is not part of this product and
                        // is left to be denied.
                        if !media.is_for_audio_device() || media.is_for_video_device() {
                            return false;
                        }
                        media.allow();
                        true
                    });
                }) {
                    // Not fatal: everything except speaking to the device
                    // still works, and the voice layer says so in words
                    // the owner can act on rather than failing silently.
                    log::warn!("could not enable the microphone: {error}");
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
            cloud_key::voice_key_save,
            cloud_key::voice_key_status,
            cloud_key::voice_key_delete,
            voice::voice_transcribe,
            voice::voice_speak,
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
            sessions::sessions_list,
            sessions::sessions_active,
            sessions::sessions_open,
            sessions::sessions_new,
            sessions::sessions_rename,
            sessions::sessions_keep,
            sessions::sessions_delete,
            dev::dev_reset_setup,
            model::model_current,
            model::model_options,
            model::model_set,
            shelf::shelf_list,
            views::views_list
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
