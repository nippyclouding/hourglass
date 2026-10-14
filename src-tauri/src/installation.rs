//! Enable launch at login once, after the release app reaches its install location.
//! Development builds never register their temporary executable paths.
use std::path::Path;
use tauri::{AppHandle, Manager};
use tauri_plugin_autostart::ManagerExt;

use crate::settings::{self, Settings, SettingsStore};

const INITIALIZED: &str = "autostartInitialized";

fn is_mac_install_path(executable: &Path, home: &Path) -> bool {
    let Some(bundle) = executable
        .ancestors()
        .find(|path| path.file_name().is_some_and(|name| name == "Hourglass.app"))
    else {
        return false;
    };
    bundle.parent().is_some_and(|parent| {
        parent == Path::new("/Applications") || parent == home.join("Applications")
    })
}

fn should_initialize(debug: bool, installed: bool, initialized: bool) -> bool {
    !debug && installed && !initialized
}

pub fn initialize(
    app: &AppHandle,
    settings: &mut Settings,
    store: &SettingsStore,
) -> Result<(), String> {
    let executable = std::env::current_exe().map_err(|error| error.to_string())?;
    let home = app.path().home_dir().map_err(|error| error.to_string())?;
    let installed = if cfg!(target_os = "macos") {
        is_mac_install_path(&executable, &home)
    } else {
        cfg!(target_os = "windows")
    };
    let initialized = store
        .get(INITIALIZED)
        .and_then(|value| value.as_bool())
        .unwrap_or(false);
    if !should_initialize(cfg!(debug_assertions), installed, initialized) {
        return Ok(());
    }
    let previous_enabled = app
        .autolaunch()
        .is_enabled()
        .map_err(|error| error.to_string())?;
    let previous_settings = store.get("settings");
    let mut updated = settings.clone();
    updated.launch_at_startup = true;
    let value = serde_json::to_value(&updated).map_err(|error| error.to_string())?;
    settings::set_autostart(app, true)?;
    store.set("settings", value);
    store.set(INITIALIZED, true);
    if let Err(error) = store.save() {
        store.delete(INITIALIZED);
        if let Some(previous) = previous_settings {
            store.set("settings", previous);
        } else {
            store.delete("settings");
        }
        let rollback = settings::set_autostart(app, previous_enabled);
        return Err(match rollback {
            Ok(()) => format!("Could not save launch-at-login settings: {error}"),
            Err(rollback) => format!(
                "Could not save launch-at-login settings: {error}; rollback failed: {rollback}"
            ),
        });
    }
    *settings = updated;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recognizes_system_and_user_applications_but_not_build_or_download_paths() {
        let home = Path::new("/Users/example");
        assert!(is_mac_install_path(
            Path::new("/Applications/Hourglass.app/Contents/MacOS/hourglass"),
            home
        ));
        assert!(is_mac_install_path(
            Path::new("/Users/example/Applications/Hourglass.app/Contents/MacOS/hourglass"),
            home
        ));
        assert!(!is_mac_install_path(
            Path::new("/Users/example/Downloads/Hourglass.app/Contents/MacOS/hourglass"),
            home
        ));
        assert!(!is_mac_install_path(Path::new("/project/src-tauri/target/release/bundle/macos/Hourglass.app/Contents/MacOS/hourglass"), home));
        assert!(!is_mac_install_path(
            Path::new("/Applications/Another.app/Contents/MacOS/hourglass"),
            home
        ));
    }

    #[test]
    fn first_install_enables_login_but_later_launches_respect_user_disable() {
        assert!(should_initialize(false, true, false));
        assert!(!should_initialize(false, true, true));
        assert!(!should_initialize(true, true, false));
        assert!(!should_initialize(false, false, false));
    }
}
