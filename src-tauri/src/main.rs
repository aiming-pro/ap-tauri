#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use std::sync::{
    self,
    atomic::{AtomicBool, Ordering},
    Arc,
};

use discord_rich_presence::{DiscordIpc, DiscordIpcClient};
use store::Settings;

use tauri::GlobalShortcutManager;
use tauri::{async_runtime::Mutex, Manager, WindowEvent};

use crate::protocol::ProtocolType;
// use tauri_runtime_wry::wry::application::{event_loop::EventLoop, window::Fullscreen};

mod commands;
mod constants;
mod menu;
mod protocol;
mod store;
mod webview;

const INIT_SCRIPT: &str = include_str!("js/script.js");

pub struct AppState {
    ready: bool,
    queued_action: Option<ProtocolType>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            ready: false,
            queued_action: None,
        }
    }
}

fn main() {
    tauri_plugin_deep_link::prepare("pro.aiming");

    // See if args includes custom protocol
    let mut protocol_action = None;
    if let Some(request) = std::env::args().nth(1) {
        match request.parse::<ProtocolType>() {
            Ok(protocol) => {
                protocol_action = Some(protocol);
            }
            Err(_) => {}
        }
    }

    // Activate Discord Rich Presence
    let mut client = DiscordIpcClient::new(constants::DISCORD_CLIENTID).unwrap();
    // Silently fail any Discord IPC errors
    client.connect().ok();

    let app = tauri::Builder::default()
        .manage(Mutex::new(Settings::default()))
        .manage(sync::Mutex::new(client))
        .manage(sync::Mutex::new(AppState {
            queued_action: protocol_action,
            ..Default::default()
        }))
        .invoke_handler(tauri::generate_handler![
            commands::discordactivity,
            commands::gamewindow,
            commands::fullscreen,
            commands::ready,
        ])
        .setup(|app| {
            // Read settings
            let handle = app.handle();
            let task = tauri::async_runtime::spawn(async move {
                let state = handle.state::<Mutex<Settings>>();
                let mut value = state.lock().await;
                // It's ok if it fails to load, just remain with default values
                value.load(&handle).await.ok();
                value.clone()
            });
            let settings = tauri::async_runtime::block_on(task).unwrap();

            // Set custom protocol
            let handle = app.handle();
            tauri_plugin_deep_link::register(constants::PROTOCOL_PREFIX, move |request| {
                match request.parse::<ProtocolType>() {
                    Ok(protocol) => {
                        let state = handle.state::<sync::Mutex<AppState>>();
                        let mut value = state.lock().unwrap();
                        if value.ready {
                            handle.get_window("main").map(|window| {
                                protocol.activate(&window);
                            });
                        } else {
                            value.queued_action = Some(protocol);
                        }
                    }
                    Err(_) => {}
                }
            })
            .unwrap();

            // Setup additional browser args
            let mut args = String::new();
            if !settings.vsync {
                args.push_str(" --disable-gpu-vsync");
            }
            if settings.unlimited_fps {
                args.push_str(" --disable-frame-rate-limit");
            }

            // Create window & webview
            let window = tauri::WindowBuilder::new(
                app,
                "main",
                tauri::WindowUrl::External("https://aiming.pro".parse().unwrap()),
            )
            .title("Aiming.Pro")
            .maximized(true)
            // Modifications to tauri-runtime-wry make this an additional_browser_args function instead
            .user_agent(&args)
            .menu(menu::create_menu(&settings))
            .initialization_script(INIT_SCRIPT)
            .build()?;

            // Set user agent
            window
                .with_webview(|webview| {
                    webview::set_user_agent(&webview);
                    webview::disable_new_windows(&webview);
                })
                .expect("Failed to set user agent");

            // Fullscreen shortcut

            let focused = Arc::new(AtomicBool::new(false));
            let focused_clone = focused.clone();

            window.on_window_event(move |event| match event {
                WindowEvent::Focused(value) => focused_clone.store(*value, Ordering::Relaxed),
                _ => {}
            });

            app.global_shortcut_manager()
                .register("F11", move || {
                    if focused.load(Ordering::Relaxed) {
                        if let Ok(fullscreened) = window.is_fullscreen() {
                            if let Ok(_) = window.set_fullscreen(!fullscreened) {
                                if fullscreened {
                                    window.menu_handle().show().ok();
                                } else {
                                    window.menu_handle().hide().ok();
                                }
                            }
                        }
                    }
                })
                .expect("Failed to register shortcut");

            Ok(())
        })
        .on_menu_event(|event| match event.menu_item_id() {
            "disable-vsync" => {
                tauri::async_runtime::spawn(async move {
                    let window = event.window();
                    let handle = window.app_handle();
                    let state = window.state::<Mutex<Settings>>();
                    let item = window.menu_handle().get_item(event.menu_item_id());

                    let mut value = state.lock().await;
                    value.vsync = !value.vsync;
                    item.set_selected(value.vsync).unwrap();
                    value.save(&handle).await.expect("Failed to save config");

                    // handle.restart();
                });
            }
            "disable-fps-limit" => {
                tauri::async_runtime::spawn(async move {
                    let window = event.window();
                    let handle = window.app_handle();
                    let state = window.state::<Mutex<Settings>>();
                    let item = window.menu_handle().get_item(event.menu_item_id());

                    let mut value = state.lock().await;
                    value.unlimited_fps = !value.unlimited_fps;
                    item.set_selected(value.unlimited_fps).unwrap();
                    value.save(&handle).await.expect("Failed to save config");

                    // handle.restart();
                });
            }
            "fullscreen-on-game-start" => {
                tauri::async_runtime::spawn(async move {
                    let window = event.window();
                    let handle = window.app_handle();
                    let state = window.state::<Mutex<Settings>>();
                    let item = window.menu_handle().get_item(event.menu_item_id());

                    let mut value = state.lock().await;
                    value.fullscreen_on_game_start = !value.fullscreen_on_game_start;
                    item.set_selected(value.fullscreen_on_game_start).unwrap();
                    value.save(&handle).await.expect("Failed to save config");
                });
            }
            "fullscreen" => {
                let window = event.window();
                if let Ok(fullscreened) = window.is_fullscreen() {
                    if let Ok(_) = window.set_fullscreen(!fullscreened) {
                        if fullscreened {
                            window.menu_handle().show().ok();
                        } else {
                            window.menu_handle().hide().ok();
                        }
                    }
                }
            }
            "reload" => {
                event.window().eval("window.location.reload()").ok();
            }
            "devtools" => {
                let window = event.window();
                if window.is_devtools_open() {
                    event.window().close_devtools();
                } else {
                    event.window().open_devtools();
                }
            }
            _ => {}
        })
        .build(tauri::generate_context!())
        .expect("Error while running application");

    app.run(|app_handle, event| match event {
        tauri::RunEvent::ExitRequested { .. } => {
            if let Ok(mut client) = app_handle.state::<sync::Mutex<DiscordIpcClient>>().lock() {
                client.close().ok();
            }
        }
        _ => {}
    });
}
