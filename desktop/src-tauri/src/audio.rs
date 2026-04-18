use tauri::{AppHandle, Runtime};
use tauri_plugin_dialog::DialogExt;

#[tauri::command]
pub async fn pick_audio_file<R: Runtime>(app: AppHandle<R>) -> Option<String> {
    let (tx, rx) = tokio::sync::oneshot::channel();
    app.dialog()
        .file()
        .add_filter("Audio", &["wav", "mp3", "ogg", "flac", "m4a", "aac"])
        .pick_file(move |chosen| {
            let path = chosen.and_then(|p| p.as_path().map(|p| p.to_string_lossy().into_owned()));
            let _ = tx.send(path);
        });
    rx.await.ok().flatten()
}
