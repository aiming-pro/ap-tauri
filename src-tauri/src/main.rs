#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use std::sync;

use discord_rich_presence::{DiscordIpc, DiscordIpcClient};
use store::Settings;

use tauri::{async_runtime::Mutex, Manager};
// use tauri_runtime_wry::wry::application::{event_loop::EventLoop, window::Fullscreen};

mod commands;
mod constants;
mod menu;
mod store;
mod webview;

const INIT_SCRIPT: &str = r#"
window.addEventListener('keydown', e => {
    if(e.code === 'F11') {
        window.__TAURI__?.invoke('fullscreen');
    }
});

const parsePageFromTitle = (title) =>
    title.split('|')[0] !== "Aiming.Pro "
        ? title.split('|')[0]
        : 'General';

const gameActivity = (status) => {
    const activity = {
        title: status.gameName,
        description: `Current HS: ${status.highScore.toString()}`
    };

    // Send the activity-update
    window.__TAURI__?.invoke("discordactivity", { activity });
};

const browseActivity = () => {
    // Default activity if window is closed
    const activity = {
        title: "Browsing",
        description: parsePageFromTitle(document.title)
    };
    // Send the activity-update
    window.__TAURI__?.invoke("discordactivity", { activity });
};


window.addEventListener(
    "DOMContentLoaded",
    () => {

        // IF GAME PAGE
        if (typeof (window).gameVue === "object") {
            window.__TAURI__?.invoke("gamewindow", { open: true });
        } else {
            // let the controller know and update activity
            browseActivity();
            window.__TAURI__?.invoke("gamewindow", { open: false });
        }

        /* Wait for Game Events to send to the RPC */
        window.addEventListener(
            "game-status-update",
            (e) => {
                // Prepare the discord template
                gameActivity(e.detail);
            }
        );

        window.addEventListener("game-modal-closed", () => {
            browseActivity();
            window.__TAURI__?.invoke("gamewindow", { open: false });
        });

        // If a game has started
        window.addEventListener("project-started", () => {
            // We don't want to use the regular injection if it's a modal
            if (typeof (window).gameVue !== "object"){
                // Let the controller now that a game has been opened
                window.__TAURI__?.invoke("gamewindow", { open: true });
            }
        });
    },
    false
);

"#;

fn main() {
    let mut client = DiscordIpcClient::new(constants::DISCORD_CLIENTID).unwrap();
    // Silently fail any Discord IPC errors
    client.connect().ok();

    let app = tauri::Builder::default()
        .manage(Mutex::new(Settings::default()))
        .manage(sync::Mutex::new(client))
        .invoke_handler(tauri::generate_handler![
            commands::discordactivity,
            commands::gamewindow,
            commands::fullscreen
        ])
        .setup(|app| {
            let handle = app.handle();
            let task = tauri::async_runtime::spawn(async move {
                let state = handle.state::<Mutex<Settings>>();
                let mut value = state.lock().await;
                // It's ok if it fails to load, just remain with default values
                value.load(&handle).await.ok();
                value.clone()
            });
            let settings = tauri::async_runtime::block_on(task).unwrap();

            let mut args = String::new();
            if !settings.vsync {
                args.push_str(" --disable-gpu-vsync");
            }
            if settings.unlimited_fps {
                args.push_str(" --disable-frame-rate-limit");
            }

            let window = tauri::WindowBuilder::new(
                app,
                "main",
                tauri::WindowUrl::External("https://aiming.pro".parse().unwrap()),
            )
            .title("Aiming.Pro (Test App)")
            .maximized(true)
            // Modifications to tauri-runtime-wry make this an additional_browser_args function instead
            .user_agent(&args)
            .menu(menu::create_menu(settings))
            .initialization_script(INIT_SCRIPT)
            .build()?;

            window
                .with_webview(|webview| {
                    webview::set_user_agent(webview);
                })
                .expect("Failed");

            // let focused = Arc::new(AtomicBool::new(false));
            // let focused_clone = focused.clone();

            // window.on_window_event(move |event| match event {
            //     WindowEvent::Focused(value) => focused_clone.store(*value, Ordering::Relaxed),
            //     _ => {}
            // });

            // app.global_shortcut_manager()
            //     .register("F11", move || {
            //         if focused.load(Ordering::Relaxed) {
            //             if let Ok(fullscreened) = window.is_fullscreen() {
            //                 window
            //                     .set_fullscreen(!fullscreened)
            //                     .expect("Failed to fullscreen");

            //                 if fullscreened {
            //                     window.menu_handle().show().ok();
            //                 } else {
            //                     window.menu_handle().hide().ok();
            //                 }
            //             }
            //         }
            //     })
            //     .expect("Could not setup shortcut");

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
            _ => {}
        })
        .build(tauri::generate_context!())
        .expect("error while running tauri application");

    app.run(|app_handle, event| match event {
        tauri::RunEvent::ExitRequested { .. } => {
            if let Ok(mut client) = app_handle.state::<sync::Mutex<DiscordIpcClient>>().lock() {
                client.close().ok();
            }
        }
        _ => {}
    });
}
