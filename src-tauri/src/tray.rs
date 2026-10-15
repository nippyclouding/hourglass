use std::{thread, time::SystemTime};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Manager, PhysicalPosition, WebviewUrl, WebviewWindowBuilder, WindowEvent};
use tauri_plugin_notification::NotificationExt;

use crate::timer::{format_remaining, Timer};
use crate::{completion, platform, AppState};

pub const TRAY_ID: &str = "hourglass";

#[derive(Clone, PartialEq, Eq)]
pub struct View {
    title: String,
    tooltip: String,
    icon_visible: bool,
    icon_frame: Option<usize>,
}

impl View {
    fn new(timer: Timer, minutes: u32, now: SystemTime) -> Self {
        let time = format_remaining(timer.remaining(now));
        let (title, tooltip) = match timer {
            Timer::Idle => (
                String::new(),
                format!("Hourglass · {minutes} min · Click to start"),
            ),
            Timer::Running { .. } => (time.clone(), format!("Hourglass · {time} · Click to pause")),
            Timer::Paused { .. } => (
                format!("Ⅱ {time}"),
                format!("Hourglass · {time} · Paused · Click to resume"),
            ),
            Timer::Finished { .. } => (
                String::from("00:00"),
                String::from("Hourglass · Finished · Click to dismiss"),
            ),
        };
        Self {
            title,
            tooltip,
            icon_visible: timer.icon_visible(now),
            icon_frame: timer.icon_frame(now),
        }
    }

    fn apply(&self, tray: &tauri::tray::TrayIcon, previous: Option<&Self>) -> tauri::Result<()> {
        if previous.map(|old| old.title.as_str()) != Some(self.title.as_str()) {
            platform::set_tray_title(tray, &self.title)?;
        }
        if previous.map(|old| old.tooltip.as_str()) != Some(self.tooltip.as_str()) {
            tray.set_tooltip(Some(&self.tooltip))?;
        }
        if previous.map(|old| (old.icon_visible, old.icon_frame))
            != Some((self.icon_visible, self.icon_frame))
        {
            // Set image and template flag together to avoid a theme-change flash
            // between animation frames on macOS. Windows uses a regular image.
            tray.set_icon_with_as_template(
                Some(platform::tray_image(self.icon_visible, self.icon_frame)),
                cfg!(target_os = "macos"),
            )?;
        }
        Ok(())
    }
}

pub fn create(app: &AppHandle) -> tauri::Result<()> {
    TrayIconBuilder::with_id(TRAY_ID)
        .icon(platform::tray_image(true, None))
        .icon_as_template(cfg!(target_os = "macos"))
        .tooltip("Hourglass")
        .show_menu_on_left_click(false)
        // No native context menu: right click opens our small settings window.
        .on_tray_icon_event(|tray, event| {
            let app = tray.app_handle();
            if let TrayIconEvent::Click {
                button,
                button_state: MouseButtonState::Up,
                position,
                ..
            } = event
            {
                match button {
                    MouseButton::Left => {
                        let state = app.state::<AppState>();
                        let (finished, acknowledged) = {
                            let mut inner = state.inner.lock().unwrap();
                            let duration = std::time::Duration::from_secs(
                                u64::from(inner.settings.minutes) * 60,
                            );
                            let finished = inner.timer.click(duration, SystemTime::now());
                            (finished, !matches!(inner.timer, Timer::Finished { .. }))
                        };
                        state.wake.notify_one();
                        if finished {
                            notify_finished(app);
                        }
                        // A click exactly at expiry also acknowledges completion.
                        if acknowledged {
                            if let Err(error) = completion::close(app) {
                                eprintln!("Could not close completion alert: {error}");
                            }
                        }
                        refresh(app);
                    }
                    MouseButton::Right => {
                        if let Err(error) = show_settings(app, position) {
                            eprintln!("Could not open settings: {error}");
                        }
                    }
                    _ => {}
                }
            }
        })
        .build(app)?;
    refresh(app);
    Ok(())
}

fn show_settings(app: &AppHandle, position: PhysicalPosition<f64>) -> tauri::Result<()> {
    if let Some(window) = app.get_webview_window("settings") {
        platform::position_settings(&window, position)?;
        window.show()?;
        return window.set_focus();
    }
    let window = WebviewWindowBuilder::new(app, "settings", WebviewUrl::App("index.html".into()))
        .title("Hourglass")
        .inner_size(260.0, 256.0)
        .resizable(false)
        .maximizable(false)
        .minimizable(false)
        .decorations(false)
        .always_on_top(true)
        .skip_taskbar(true)
        .visible(false)
        .focused(false)
        .build()?;
    platform::position_settings(&window, position)?;
    window.show()?;
    window.set_focus()?;
    let handle = app.clone();
    window.on_window_event(move |event| {
        if matches!(event, WindowEvent::Focused(false)) {
            if let Err(error) = platform::close_settings(&handle) {
                eprintln!("Could not close settings: {error}");
            }
        }
    });
    Ok(())
}

/// All calls run on the UI thread. Recompute here rather than queueing stale snapshots.
pub fn refresh(app: &AppHandle) {
    let state = app.state::<AppState>();
    let now = SystemTime::now();
    let (view, previous, finished) = {
        let mut inner = state.inner.lock().unwrap();
        let finished = inner.timer.advance(now);
        (
            View::new(inner.timer, inner.settings.minutes, now),
            inner.last_view.clone(),
            finished,
        )
    };
    if finished {
        notify_finished(app);
    }
    if previous.as_ref() == Some(&view) {
        return;
    }
    let Some(tray) = app.tray_by_id(TRAY_ID) else {
        return;
    };
    match view.apply(&tray, previous.as_ref()) {
        Ok(()) => state.inner.lock().unwrap().last_view = Some(view),
        Err(error) => eprintln!("Could not update tray: {error}"),
    }
}

fn notify_finished(app: &AppHandle) {
    if let Err(error) = completion::show(app) {
        eprintln!("Could not show completion alert: {error}");
    }
    if let Err(error) = app
        .notification()
        .builder()
        .title("Hourglass")
        .body(completion::MESSAGE)
        .show()
    {
        eprintln!("Could not show completion notification: {error}");
    }
}

pub fn spawn_worker(app: AppHandle) -> std::io::Result<()> {
    thread::Builder::new()
        .name("hourglass-timer".into())
        .spawn(move || {
            loop {
                let state = app.state::<AppState>();
                let inner = state.inner.lock().unwrap();
                if inner.stopping {
                    break;
                }
                let inner = match inner.timer.next_wake(SystemTime::now()) {
                    Some(delay) => state.wake.wait_timeout(inner, delay).unwrap().0,
                    None => state.wake.wait(inner).unwrap(),
                };
                if inner.stopping {
                    break;
                }
                drop(inner);
                // Wait for this update to complete, so a delayed UI thread cannot
                // accumulate callbacks or repeatedly notify for the same deadline.
                let (sent, received) = std::sync::mpsc::sync_channel(0);
                let handle = app.clone();
                if app
                    .run_on_main_thread(move || {
                        refresh(&handle);
                        let _ = sent.send(());
                    })
                    .is_err()
                {
                    break;
                }
                if received.recv().is_err() {
                    break;
                }
            }
        })?;
    Ok(())
}
