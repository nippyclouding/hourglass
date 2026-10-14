use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::AppHandle;
use tauri_plugin_autostart::ManagerExt;
use tauri_plugin_store::{Store, StoreExt};

// One year is a generous upper bound that also keeps SystemTime arithmetic safe.
pub const MAX_MINUTES: u32 = 525_600;

pub type SettingsStore = Arc<Store<tauri::Wry>>;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    pub minutes: u32,
    pub launch_at_startup: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            minutes: 50,
            launch_at_startup: false,
        }
    }
}

pub fn validate(minutes: u32) -> Result<(), String> {
    if !(1..=MAX_MINUTES).contains(&minutes) {
        return Err(format!(
            "Enter a whole number from 1 to {MAX_MINUTES} minutes."
        ));
    }
    Ok(())
}

pub fn load(app: &AppHandle) -> Result<(Settings, SettingsStore), Box<dyn std::error::Error>> {
    let store = app
        .store_builder("settings.json")
        .disable_auto_save()
        .build()?;
    let settings = match store.get("settings") {
        Some(value) => {
            let settings: Settings = serde_json::from_value(value)?;
            validate(settings.minutes)?;
            settings
        }
        None => Settings::default(),
    };
    Ok((settings, store))
}

pub fn set_autostart(app: &AppHandle, enabled: bool) -> Result<(), String> {
    let manager = app.autolaunch();
    let current = manager.is_enabled().map_err(|error| error.to_string())?;
    if current == enabled {
        return Ok(());
    }
    if enabled {
        manager.enable()
    } else {
        manager.disable()
    }
    .map_err(|error| error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn duration_validation() {
        assert!(validate(0).is_err());
        assert!(validate(1).is_ok());
        assert!(validate(120).is_ok());
        assert!(validate(MAX_MINUTES).is_ok());
        assert!(validate(MAX_MINUTES + 1).is_err());
    }
}
