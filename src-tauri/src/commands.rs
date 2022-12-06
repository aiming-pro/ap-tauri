use std::sync;

use discord_rich_presence::{
    activity::{self, Assets},
    DiscordIpc, DiscordIpcClient,
};
use serde::Deserialize;
use tauri::{async_runtime::Mutex, State};

use crate::{constants, store::Settings, AppState};

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
pub fn ready(window: tauri::Window, state: State<'_, sync::Mutex<AppState>>) {
    println!("Ready was printed");
    let mut value = state.lock().unwrap();
    if let Some(protocol) = &value.queued_action {
        protocol.activate(&window);
    }
    value.ready = true;
    value.queued_action = None;
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

#[derive(Default, Deserialize)]
pub struct DiscordActivity {
    pub title: String,
    pub description: String,
}

#[tauri::command]
pub fn discordactivity(activity: DiscordActivity, state: State<'_, sync::Mutex<DiscordIpcClient>>) {
    if let Ok(mut client) = state.lock() {
        client
            .set_activity(
                activity::Activity::new()
                    .details(&activity.title)
                    .state(&activity.description)
                    .assets(
                        Assets::new()
                            .large_text("Aiming.Pro")
                            .large_image(constants::DISCORD_BIGPICID)
                            .small_text("Aiming.Pro"),
                    ),
            )
            .ok();
    }
}
