use tauri::{async_runtime::Mutex, State};

use crate::store::Settings;

#[tauri::command]
pub fn fullscreen(window: tauri::Window) {
    if let Ok(fullscreened) = window.is_fullscreen() {
        window
            .set_fullscreen(!fullscreened)
            .expect("Failed to fullscreen");

        if fullscreened {
            window.menu_handle().show().ok();
        } else {
            window.menu_handle().hide().ok();
        }
    }
}

#[tauri::command]
pub async fn gamewindow(
    window: tauri::Window,
    open: bool,
    state: State<'_, Mutex<Settings>>,
) -> Result<(), ()> {
    if state.lock().await.fullscreen_on_game_start {
        if let Ok(is_fullscreen) = window.is_fullscreen() {
            if (!is_fullscreen && open) || (is_fullscreen && !open) {
                fullscreen(window);
            }
        }
    }
    Ok(())
}
