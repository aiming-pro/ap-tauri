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

mod commands;
mod constants;
mod menu;
mod protocol;
mod store;
mod webview;

const INIT_SCRIPT: &str = include_str!("js/script.js");

#[derive(Default)]
pub struct AppState {
    ready: bool,
    queued_action: Option<ProtocolType>,
}

fn main() {
    tauri_plugin_deep_link::prepare("pro.aiming");

    // See if args includes custom protocol
    let mut protocol_action = None;
    if let Some(request) = std::env::args().nth(1) {
        if let Ok(protocol) = request.parse::<ProtocolType>() {
            protocol_action = Some(protocol);
        }
    }

    // Activate Discord Rich Presence
    let mut client = DiscordIpcClient::new(constants::DISCORD_CLIENTID).ok();
    client = client.and_then(|mut client| {
        if client.connect().is_err() {
            return None;
        }
        Some(client)
    });

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
                if let Ok(protocol) = request.parse::<ProtocolType>() {
                    let state = handle.state::<sync::Mutex<AppState>>();
                    let mut value = state.lock().unwrap();
                    if value.ready {
                        if let Some(window) = handle.get_window("main") {
                            protocol.activate(&window);
                        }
                    } else {
                        value.queued_action = Some(protocol);
                    }
                }
            })
            .ok();

            // Setup additional browser args
            let mut args =
                String::from("--disable-features=msWebOOUI,msPdfOOUI,msSmartScreenProtection");
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
            .disable_file_drop_handler()
            .additional_browser_args(&args)
            .menu(menu::create_menu(&settings))
            .initialization_script(INIT_SCRIPT)
            .on_navigation(|url| {
                if let Some(host) = url.host_str() {
                    if host.ends_with("aiming.pro") || host.ends_with("localhost") {
                        return true;
                    }
                }

                open::that(url.as_str()).ok();
                false
            })
            .build()?;

            // Set user agent and disable new window creation
            window
                .with_webview(|webview| {
                    // Silently fails but should have no reason to
                    webview::set_user_agent(&webview).ok();
                    webview::disable_new_windows(&webview).ok();
                })
                .expect("Failed to extend webview");

            // Fullscreen shortcut

            let focused = Arc::new(AtomicBool::new(false));

            {
                let focused = focused.clone();
                window.on_window_event(move |event| {
                    if let WindowEvent::Focused(value) = event {
                        focused.store(*value, Ordering::Relaxed);
                    }
                });
            }

            let mut gsm = app.global_shortcut_manager();
            {
                let focused = focused.clone();
                let window = window.clone();
                gsm.register("F11", move || {
                    if focused.load(Ordering::Relaxed) {
                        if let Ok(fullscreened) = window.is_fullscreen() {
                            if window.set_fullscreen(!fullscreened).is_ok() {
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
            }

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
                    if window.set_fullscreen(!fullscreened).is_ok() {
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

    app.run(|app_handle, event| {
        if let tauri::RunEvent::ExitRequested { .. } = event {
            if let Ok(mut client) = app_handle
                .state::<sync::Mutex<Option<DiscordIpcClient>>>()
                .lock()
            {
                if let Some(client) = &mut *client {
                    client.close().ok();
                }
            }
        }
    });
}
