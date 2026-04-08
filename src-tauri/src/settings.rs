use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowSettings {
    pub width: u32,
    pub height: u32,
    pub x: i32,
    pub y: i32,
}

impl Default for WindowSettings {
    fn default() -> Self {
        Self {
            width: 1000,
            height: 720,
            x: -1,
            y: -1,
        }
    }
}

fn get_settings_path(app: &AppHandle) -> Option<PathBuf> {
    let app_dir = app.path().app_data_dir().ok()?;
    fs::create_dir_all(&app_dir).ok()?;
    Some(app_dir.join("settings.json"))
}

pub fn load_settings(app: &AppHandle) -> WindowSettings {
    let path = match get_settings_path(app) {
        Some(p) => p,
        None => return WindowSettings::default(),
    };

    match fs::read_to_string(&path) {
        Ok(content) => {
            serde_json::from_str(&content).unwrap_or_else(|_| WindowSettings::default())
        }
        Err(_) => WindowSettings::default(),
    }
}

pub fn save_settings(app: &AppHandle, settings: &WindowSettings) {
    let path = match get_settings_path(app) {
        Some(p) => p,
        None => return,
    };

    if let Ok(json) = serde_json::to_string_pretty(settings) {
        let _ = fs::write(&path, json);
    }
}