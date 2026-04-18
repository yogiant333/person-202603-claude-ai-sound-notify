mod config;

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::default().build())
        .invoke_handler(tauri::generate_handler![
            config::get_config,
            config::set_config,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
