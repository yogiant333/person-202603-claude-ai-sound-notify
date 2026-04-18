use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tauri::{AppHandle, Runtime};
use tauri_plugin_autostart::ManagerExt;
use tauri_plugin_store::StoreExt;

const STORE_FILE: &str = "config.json";
const STORE_KEY: &str = "app_config";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct AppConfig {
    pub server_url: String,
    pub sound_preferences: HashMap<String, String>,
    pub custom_audio_paths: HashMap<String, String>,
    pub volume: f32,
    pub global_muted: bool,
    pub source_enabled: HashMap<String, bool>,
    pub autostart: bool,
    pub offline_alarm_enabled: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        let mut source_enabled = HashMap::new();
        source_enabled.insert("claude-code".into(), true);
        source_enabled.insert("gemini".into(), true);
        source_enabled.insert("codex".into(), true);
        Self {
            server_url: "https://ainotify.keymantek.com:777".into(),
            sound_preferences: HashMap::new(),
            custom_audio_paths: HashMap::new(),
            volume: 0.6,
            global_muted: false,
            source_enabled,
            autostart: true,
            offline_alarm_enabled: true,
        }
    }
}

fn load<R: Runtime>(app: &AppHandle<R>) -> AppConfig {
    let Ok(store) = app.store(STORE_FILE) else {
        return AppConfig::default();
    };
    match store.get(STORE_KEY) {
        Some(value) => serde_json::from_value(value).unwrap_or_default(),
        None => AppConfig::default(),
    }
}

fn save<R: Runtime>(app: &AppHandle<R>, cfg: &AppConfig) -> Result<(), String> {
    let store = app.store(STORE_FILE).map_err(|e| e.to_string())?;
    let value = serde_json::to_value(cfg).map_err(|e| e.to_string())?;
    store.set(STORE_KEY, value);
    store.save().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_config<R: Runtime>(app: AppHandle<R>) -> AppConfig {
    load(&app)
}

#[tauri::command]
pub fn set_config<R: Runtime>(app: AppHandle<R>, config: AppConfig) -> Result<(), String> {
    save(&app, &config)
}

#[tauri::command]
pub fn set_autostart<R: Runtime>(app: AppHandle<R>, enabled: bool) -> Result<(), String> {
    let autostart = app.autolaunch();
    if enabled {
        autostart.enable().map_err(|e| e.to_string())
    } else {
        autostart.disable().map_err(|e| e.to_string())
    }
}

#[tauri::command]
pub fn get_autostart<R: Runtime>(app: AppHandle<R>) -> Result<bool, String> {
    app.autolaunch().is_enabled().map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_round_trips_through_json() {
        let cfg = AppConfig::default();
        let json = serde_json::to_string(&cfg).unwrap();
        let parsed: AppConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(cfg, parsed);
    }

    #[test]
    fn partial_json_fills_missing_fields_from_default() {
        let json = r#"{"server_url":"https://example.com"}"#;
        let parsed: AppConfig = serde_json::from_str(json).unwrap();
        assert_eq!(parsed.server_url, "https://example.com");
        assert_eq!(parsed.volume, 0.6);
        assert_eq!(parsed.source_enabled.get("claude-code"), Some(&true));
    }
}
