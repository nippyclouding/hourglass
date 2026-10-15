#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod completion;
mod installation;
mod platform;
mod settings;
mod timer;
mod tray;

use serde::Serialize;
use std::sync::{Arc, Condvar, Mutex};
use std::time::{Duration, SystemTime};
use tauri::{AppHandle, Manager, RunEvent, State};
use tauri_plugin_autostart::ManagerExt;
use tauri_plugin_store::Store;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SettingsSnapshot {
    #[serde(flatten)]
    settings: settings::Settings,
    can_restart: bool,
}

struct Inner {
    timer: timer::Timer,
    settings: settings::Settings,
    last_view: Option<tray::View>,
    stopping: bool,
}

impl Inner {
    fn restart_timer(
        &mut self,
        minutes: u32,
        now: SystemTime,
        persist: impl FnOnce(&settings::Settings, &settings::Settings) -> Result<(), String>,
    ) -> Result<(), String> {
        settings::validate(minutes)?;
        let mut timer = self.timer;
        if !timer.restart(Duration::from_secs(u64::from(minutes) * 60), now) {
            return Err("There is no active timer to restart.".into());
        }
        let settings = settings::Settings {
            minutes,
            ..self.settings.clone()
        };
        persist(&self.settings, &settings)?;
        self.settings = settings;
        self.timer = timer;
        Ok(())
    }
}

struct AppState {
    inner: Mutex<Inner>,
    wake: Condvar,
    store: Arc<Store<tauri::Wry>>,
}

#[tauri::command]
async fn get_settings(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<SettingsSnapshot, String> {
    let (mut settings, can_restart) = {
        let inner = state.inner.lock().unwrap();
        (
            inner.settings.clone(),
            inner.timer.can_restart(SystemTime::now()),
        )
    };
    // Reflect changes made in the operating system's login-item controls.
    settings.launch_at_startup = app
        .autolaunch()
        .is_enabled()
        .map_err(|error| error.to_string())?;
    Ok(SettingsSnapshot {
        settings,
        can_restart,
    })
}

#[tauri::command]
async fn restart_timer(
    app: AppHandle,
    state: State<'_, AppState>,
    minutes: u32,
) -> Result<(), String> {
    state.inner.lock().unwrap().restart_timer(
        minutes,
        SystemTime::now(),
        |previous, settings| {
            state.store.set(
                "settings",
                serde_json::to_value(settings).map_err(|error| error.to_string())?,
            );
            if let Err(error) = state.store.save() {
                state
                    .store
                    .set("settings", serde_json::to_value(previous).unwrap());
                return Err(format!("Could not save settings: {error}"));
            }
            Ok(())
        },
    )?;
    state.wake.notify_one();
    let handle = app.clone();
    app.run_on_main_thread(move || tray::refresh(&handle))
        .map_err(|error| error.to_string())?;
    Ok(())
}

#[tauri::command]
async fn save_settings(
    app: AppHandle,
    state: State<'_, AppState>,
    minutes: u32,
    launch_at_startup: bool,
) -> Result<(), String> {
    settings::validate(minutes)?;
    let actual_autostart = app
        .autolaunch()
        .is_enabled()
        .map_err(|error| error.to_string())?;
    let mut inner = state.inner.lock().unwrap();
    let previous = inner.settings.clone();
    let settings = settings::Settings {
        minutes,
        launch_at_startup,
    };
    settings::set_autostart(&app, launch_at_startup)?;
    state.store.set(
        "settings",
        serde_json::to_value(&settings).map_err(|error| error.to_string())?,
    );
    if let Err(error) = state.store.save() {
        state
            .store
            .set("settings", serde_json::to_value(&previous).unwrap());
        let rollback = settings::set_autostart(&app, actual_autostart);
        return Err(match rollback {
            Ok(()) => format!("Could not save settings: {error}"),
            Err(rollback) => format!(
                "Could not save settings: {error}. Could not restore login setting: {rollback}"
            ),
        });
    }
    // Keep the active session unchanged until the next start or explicit restart.
    inner.settings = settings;
    drop(inner);
    let handle = app.clone();
    app.run_on_main_thread(move || tray::refresh(&handle))
        .map_err(|error| error.to_string())?;
    Ok(())
}

#[tauri::command]
async fn close_settings(app: AppHandle) -> Result<(), String> {
    platform::close_settings(&app).map_err(|error| error.to_string())
}

#[tauri::command]
fn quit(app: AppHandle) {
    app.exit(0);
}

fn main() {
    let app = tauri::Builder::default()
        // Register first: a second launch must not create another tray icon.
        .plugin(tauri_plugin_single_instance::init(|_, _, _| {}))
        .plugin(platform::autostart_plugin())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .invoke_handler(tauri::generate_handler![
            get_settings,
            save_settings,
            restart_timer,
            completion::dismiss_completion,
            close_settings,
            quit
        ])
        .setup(|app| {
            platform::configure(app);
            let (mut settings, store) = settings::load(app.handle())?;
            if let Err(error) = installation::initialize(app.handle(), &mut settings, &store) {
                eprintln!("Could not initialize launch at login: {error}");
            }
            // After installation, respect changes made in settings or the OS.
            app.manage(AppState {
                inner: Mutex::new(Inner {
                    timer: timer::Timer::Idle,
                    settings,
                    last_view: None,
                    stopping: false,
                }),
                wake: Condvar::new(),
                store,
            });
            tray::create(app.handle())?;
            tray::spawn_worker(app.handle().clone())?;
            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("Could not initialize Hourglass");

    app.run(|handle, event| match event {
        RunEvent::ExitRequested { api, code, .. } if code.is_none() => {
            // Closing the settings window must never quit the tray-only app.
            api.prevent_exit();
        }
        RunEvent::Exit => {
            if let Some(state) = handle.try_state::<AppState>() {
                state.inner.lock().unwrap().stopping = true;
                state.wake.notify_one();
            }
        }
        _ => {}
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    fn active() -> Inner {
        let mut timer = timer::Timer::Idle;
        timer.click(Duration::from_secs(3000), SystemTime::UNIX_EPOCH);
        Inner {
            timer,
            settings: settings::Settings::default(),
            last_view: None,
            stopping: false,
        }
    }

    #[test]
    fn restart_persists_the_new_minutes_without_changing_autostart() {
        let mut inner = active();
        let now = SystemTime::UNIX_EPOCH + Duration::from_secs(600);
        let mut saved = None;
        inner
            .restart_timer(25, now, |previous, settings| {
                assert_eq!(previous.minutes, 50);
                saved = Some(settings.clone());
                Ok(())
            })
            .unwrap();
        assert_eq!(saved.unwrap().minutes, 25);
        assert_eq!(inner.settings.minutes, 25);
        assert!(!inner.settings.launch_at_startup);
        assert_eq!(inner.timer.remaining(now), Duration::from_secs(1500));
    }

    #[test]
    fn failed_restart_persistence_keeps_the_timer_and_settings() {
        let mut inner = active();
        let previous_timer = inner.timer;
        let result =
            inner.restart_timer(
                25,
                SystemTime::UNIX_EPOCH,
                |_, _| Err("Disk is full".into()),
            );
        assert_eq!(result, Err("Disk is full".into()));
        assert_eq!(inner.timer, previous_timer);
        assert_eq!(inner.settings.minutes, 50);
    }

    #[test]
    fn invalid_or_inactive_restart_does_not_persist_or_change_state() {
        for (minutes, now, timer) in [
            (0, SystemTime::UNIX_EPOCH, active().timer),
            (
                settings::MAX_MINUTES + 1,
                SystemTime::UNIX_EPOCH,
                active().timer,
            ),
            (25, SystemTime::UNIX_EPOCH, timer::Timer::Idle),
            (
                25,
                SystemTime::UNIX_EPOCH + Duration::from_secs(3000),
                active().timer,
            ),
            (
                25,
                SystemTime::UNIX_EPOCH,
                timer::Timer::Finished {
                    blink_started: SystemTime::UNIX_EPOCH,
                },
            ),
        ] {
            let mut inner = active();
            inner.timer = timer;
            assert!(inner
                .restart_timer(minutes, now, |_, _| panic!("must not save"))
                .is_err());
            assert_eq!(inner.timer, timer);
            assert_eq!(inner.settings.minutes, 50);
        }
    }
}
