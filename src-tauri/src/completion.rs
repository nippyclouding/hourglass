use tauri::{AppHandle, Manager, State, WebviewUrl, WebviewWindowBuilder};

use crate::{timer::Timer, tray, AppState};

pub const MESSAGE: &str = "All the sand has run through the hourglass.";
const WINDOW_ID: &str = "completion";

/// Called on the UI thread when a timer crosses its deadline.
pub fn show(app: &AppHandle) -> tauri::Result<()> {
    if let Some(window) = app.get_webview_window(WINDOW_ID) {
        window.center()?;
        window.show()?;
        return window.set_focus();
    }
    let window =
        WebviewWindowBuilder::new(app, WINDOW_ID, WebviewUrl::App("completion.html".into()))
            .title("Timer complete")
            .inner_size(380.0, 230.0)
            .resizable(false)
            .maximizable(false)
            .minimizable(false)
            .decorations(false)
            .always_on_top(true)
            .skip_taskbar(true)
            .center()
            .visible(false)
            .focused(false)
            .build()?;
    window.show()?;
    window.set_focus()
}

pub fn close(app: &AppHandle) -> tauri::Result<()> {
    if let Some(window) = app.get_webview_window(WINDOW_ID) {
        window.close()?;
    }
    Ok(())
}

fn acknowledge(timer: &mut Timer) {
    // A delayed dismissal must never reset a timer started from the tray.
    if matches!(timer, Timer::Finished { .. }) {
        *timer = Timer::Idle;
    }
}

#[tauri::command]
pub async fn dismiss_completion(app: AppHandle, state: State<'_, AppState>) -> Result<(), String> {
    acknowledge(&mut state.inner.lock().unwrap().timer);
    state.wake.notify_one();
    let closed = close(&app).map_err(|error| error.to_string());
    let handle = app.clone();
    app.run_on_main_thread(move || tray::refresh(&handle))
        .map_err(|error| error.to_string())?;
    closed
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, SystemTime};

    #[test]
    fn dismissal_acknowledges_a_finished_timer() {
        let mut timer = Timer::Finished {
            blink_started: SystemTime::UNIX_EPOCH,
        };
        acknowledge(&mut timer);
        assert_eq!(timer, Timer::Idle);
    }

    #[test]
    fn stale_dismissal_preserves_a_new_running_or_paused_timer() {
        for original in [
            Timer::Idle,
            Timer::Running {
                end_time: SystemTime::UNIX_EPOCH + Duration::from_secs(300),
                duration: Duration::from_secs(300),
            },
            Timer::Paused {
                remaining: Duration::from_secs(120),
                duration: Duration::from_secs(300),
            },
        ] {
            let mut timer = original;
            acknowledge(&mut timer);
            assert_eq!(timer, original);
        }
    }
}
