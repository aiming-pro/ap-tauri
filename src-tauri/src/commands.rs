use std::sync;

use discord_rich_presence::{
    activity::{self, Assets},
    DiscordIpc, DiscordIpcClient,
};
use serde::Deserialize;
use tauri::{async_runtime::Mutex, State};

use crate::{constants, store::Settings, AppState};

pub fn fullscreen(window: &tauri::Window, value: bool, _exclusive: bool) {
    // if value && exclusive {
    //     window
    //         .set_resizable(value)
    //         .expect("Failed to exclusive fullscreen");
    // } else {
    window.set_fullscreen(value).expect("Failed to fullscreen");
    // }

    if value {
        window.menu_handle().hide().ok();
    } else {
        window.menu_handle().show().ok();
    }
}

#[tauri::command]
pub fn ready(window: tauri::Window, state: State<'_, sync::Mutex<AppState>>) {
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
    let settings = state.lock().await;
    if settings.fullscreen_on_game_start {
        if let Ok(is_fullscreen) = window.is_fullscreen() {
            if (!is_fullscreen && open) || (is_fullscreen && !open) {
                fullscreen(&window, !is_fullscreen, settings.exclusive_fullscreen);
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
pub fn discordactivity(
    activity: DiscordActivity,
    state: State<'_, sync::Mutex<Option<DiscordIpcClient>>>,
) {
    if let Ok(mut client) = state.lock() {
        if let Some(client) = &mut *client {
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
}
