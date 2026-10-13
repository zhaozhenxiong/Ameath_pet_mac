use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    LogicalSize, Manager, PhysicalPosition, WebviewWindow,
};

const ONE_X_WIDTH: f64 = 128.0;
const ONE_X_HEIGHT: f64 = 170.0;
const DESIGN_SCALE: f64 = 1.5;

pub fn setup(app: &tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let show_item = MenuItem::with_id(app, "show", "显示/隐藏", true, None::<&str>)?;
    let scale_1x = MenuItem::with_id(app, "scale_1x", "缩放 1x", true, None::<&str>)?;
    let scale_1_5x = MenuItem::with_id(app, "scale_1_5x", "缩放 1.5x", true, None::<&str>)?;
    let scale_2x = MenuItem::with_id(app, "scale_2x", "缩放 2x", true, None::<&str>)?;
    let about_item = MenuItem::with_id(app, "about", "关于", true, None::<&str>)?;
    let quit_item = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;

    let menu = Menu::with_items(
        app,
        &[
            &show_item,
            &scale_1x,
            &scale_1_5x,
            &scale_2x,
            &about_item,
            &quit_item,
        ],
    )?;

    let _tray = TrayIconBuilder::with_id("aemeath-tray")
        .icon(app.default_window_icon().unwrap().clone())
        .menu(&menu)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "show" => {
                if let Some(window) = app.get_webview_window("aemeath") {
                    if window.is_visible().unwrap_or(false) {
                        let _ = window.hide();
                    } else {
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                }
            }
            "scale_1x" => set_scale(app.get_webview_window("aemeath"), 1.0),
            "scale_1_5x" => set_scale(app.get_webview_window("aemeath"), 1.5),
            "scale_2x" => set_scale(app.get_webview_window("aemeath"), 2.0),
            "quit" => {
                app.exit(0);
            }
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                let app = tray.app_handle();
                if let Some(window) = app.get_webview_window("aemeath") {
                    if window.is_visible().unwrap_or(false) {
                        let _ = window.hide();
                    } else {
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                }
            }
        })
        .build(app)?;

    Ok(())
}

fn set_scale(window: Option<WebviewWindow>, factor: f64) {
    let Some(window) = window else {
        return;
    };

    if let Err(error) = apply_scale(&window, factor) {
        eprintln!("Failed to set pet scale to {factor}: {error}");
    }
}

pub(crate) fn apply_scale(window: &WebviewWindow, factor: f64) -> tauri::Result<()> {
    let old_position = window.outer_position().ok();
    let old_size = window.outer_size().ok();
    let display_scale = window.scale_factor().unwrap_or(1.0);
    let (width, height) = scaled_dimensions(factor);
    let zoom = factor / DESIGN_SCALE;

    // Resize the native window and zoom the WebView together. Resizing alone
    // only adds transparent space because the sprite has a fixed CSS size.
    window.set_size(LogicalSize::new(width, height))?;
    window.set_zoom(zoom)?;

    // Keep the pet's feet anchored while it grows so scaling near a screen
    // edge does not push it off-screen.
    if let (Some(position), Some(size)) = (old_position, old_size) {
        let new_width = (width * display_scale).round() as i32;
        let new_height = (height * display_scale).round() as i32;
        let foot_x = position.x + size.width as i32 / 2;
        let foot_y = position.y + size.height as i32;
        window.set_position(PhysicalPosition::new(
            foot_x - new_width / 2,
            foot_y - new_height,
        ))?;
    }

    let _ = window.show();
    Ok(())
}

fn scaled_dimensions(factor: f64) -> (f64, f64) {
    (ONE_X_WIDTH * factor, ONE_X_HEIGHT * factor)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn computes_supported_scale_dimensions() {
        assert_eq!(scaled_dimensions(1.0), (128.0, 170.0));
        assert_eq!(scaled_dimensions(1.5), (192.0, 255.0));
        assert_eq!(scaled_dimensions(2.0), (256.0, 340.0));
    }
}
