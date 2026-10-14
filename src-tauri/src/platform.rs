//! OS-specific presentation lives here; timer behavior is shared.
use tauri::{App, AppHandle, PhysicalPosition, WebviewWindow};

pub fn configure(app: &mut App) {
    #[cfg(target_os = "macos")]
    app.set_activation_policy(tauri::ActivationPolicy::Accessory);
    #[cfg(not(target_os = "macos"))]
    let _ = app;
}

pub fn autostart_plugin() -> tauri::plugin::TauriPlugin<tauri::Wry> {
    let builder = tauri_plugin_autostart::Builder::new().app_name("io.hourglass.timer");
    #[cfg(target_os = "macos")]
    let builder = builder.macos_launcher(tauri_plugin_autostart::MacosLauncher::LaunchAgent);
    builder.build()
}

pub fn set_tray_title(tray: &tauri::tray::TrayIcon, title: &str) -> tauri::Result<()> {
    #[cfg(target_os = "macos")]
    tray.set_title(Some(title))?;
    #[cfg(not(target_os = "macos"))]
    let _ = (tray, title); // Windows uses the tooltip instead.
    Ok(())
}

pub fn tray_image(visible: bool, frame: Option<usize>) -> tauri::image::Image<'static> {
    const SIZE: u32 = 32;
    if !visible {
        // Keep the tray item (and its click target) while displaying transparent pixels.
        return tauri::image::Image::new_owned(vec![0; (SIZE * SIZE * 4) as usize], SIZE, SIZE);
    }
    if let Some(frame) = frame {
        const BYTES: usize = 32 * 32 * 4;
        #[cfg(target_os = "windows")]
        const FRAMES: &[u8; BYTES * crate::timer::ANIMATION_FRAMES] =
            include_bytes!("../icons/tray-animation-windows.rgba");
        #[cfg(not(target_os = "windows"))]
        const FRAMES: &[u8; BYTES * crate::timer::ANIMATION_FRAMES] =
            include_bytes!("../icons/tray-animation.rgba");
        let start = frame.min(crate::timer::ANIMATION_FRAMES - 1) * BYTES;
        return tauri::image::Image::new_owned(FRAMES[start..start + BYTES].to_vec(), SIZE, SIZE);
    }
    #[cfg(target_os = "windows")]
    let pixels = include_bytes!("../icons/tray-windows.rgba");
    #[cfg(not(target_os = "windows"))]
    let pixels = include_bytes!("../icons/tray.rgba");
    tauri::image::Image::new_owned(pixels.to_vec(), SIZE, SIZE)
}

pub fn position_settings(
    window: &WebviewWindow,
    click: PhysicalPosition<f64>,
) -> tauri::Result<()> {
    let monitor = window
        .monitor_from_point(click.x, click.y)?
        .or(window.primary_monitor()?);
    let Some(monitor) = monitor else {
        return Ok(());
    };
    let scale = monitor.scale_factor();
    let size = window.outer_size()?;
    let width = f64::from(size.width);
    let height = f64::from(size.height);
    let area = monitor.work_area();
    let origin = &area.position;
    let bounds = &area.size;
    let gap = 8.0 * scale;
    let min_x = f64::from(origin.x) + gap;
    let min_y = f64::from(origin.y) + gap;
    let max_x = (f64::from(origin.x) + f64::from(bounds.width) - width - gap).max(min_x);
    let max_y = (f64::from(origin.y) + f64::from(bounds.height) - height - gap).max(min_y);
    let x = (click.x - width / 2.0).clamp(min_x, max_x);
    // Below the macOS menu bar. On Windows, also handle taskbars on any edge.
    let y = if click.y > f64::from(origin.y) + f64::from(bounds.height) / 2.0 {
        click.y - height - gap
    } else {
        click.y + gap
    }
    .clamp(min_y, max_y);
    window.set_position(PhysicalPosition::new(x as i32, y as i32))
}

pub fn close_settings(app: &AppHandle) -> tauri::Result<()> {
    use tauri::Manager;
    if let Some(window) = app.get_webview_window("settings") {
        window.close()?;
    }
    Ok(())
}
