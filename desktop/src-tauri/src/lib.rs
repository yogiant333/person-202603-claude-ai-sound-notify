mod audio;
mod config;
mod monitor;

use tauri_plugin_store::StoreExt;

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            config::get_config,
            config::set_config,
            audio::pick_audio_file,
        ])
        .setup(|app| {
            let handle = app.handle().clone();
            let cfg = handle
                .store("config.json")
                .ok()
                .and_then(|store| store.get("app_config"))
                .and_then(|v| serde_json::from_value::<config::AppConfig>(v).ok())
                .unwrap_or_default();
            monitor::spawn(handle, cfg.server_url);
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
