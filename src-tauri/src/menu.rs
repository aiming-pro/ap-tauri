use tauri::{CustomMenuItem, Menu, MenuItem, Submenu};

use crate::store::Settings;

pub fn create_menu(settings: &Settings) -> Menu {
    let mut auto_fullscreen = CustomMenuItem::new(
        "fullscreen-on-game-start".to_string(),
        "Auto Fullscreen In-Game",
    );
    if settings.fullscreen_on_game_start {
        auto_fullscreen = auto_fullscreen.selected();
    }

    let mut vsync = CustomMenuItem::new("disable-vsync".to_string(), "V-Sync (Requires restart)");
    if settings.vsync {
        vsync = vsync.selected();
    }

    let mut unlimited_fps = CustomMenuItem::new(
        "disable-fps-limit".to_string(),
        "Unlimited Framerate (Requires restart)",
    );
    if settings.unlimited_fps {
        unlimited_fps = unlimited_fps.selected();
    }

    let mut exclusive_fullscreen =
        CustomMenuItem::new("exclusive-fullscreen".to_string(), "Exclusive Fullscreen");
    if settings.exclusive_fullscreen {
        exclusive_fullscreen = exclusive_fullscreen.selected();
    }

    Menu::new()
        .add_submenu(Submenu::new(
            "App",
            Menu::new()
                .add_item(
                    CustomMenuItem::new(
                        "id".to_string(),
                        concat!("App Version ", env!("CARGO_PKG_VERSION")),
                    )
                    .disabled(),
                )
                .add_native_item(MenuItem::Quit),
        ))
        .add_submenu(Submenu::new(
            "Edit",
            Menu::new()
                // .add_native_item(MenuItem::Undo)
                // .add_native_item(MenuItem::Redo)
                // .add_native_item(MenuItem::Separator)
                .add_native_item(MenuItem::Cut)
                .add_native_item(MenuItem::Copy)
                .add_native_item(MenuItem::Paste)
                .add_native_item(MenuItem::SelectAll),
        ))
        .add_submenu(Submenu::new(
            "View",
            Menu::new()
                .add_item(CustomMenuItem::new("reload".to_string(), "Reload").accelerator("Ctrl+R"))
                .add_item(
                    CustomMenuItem::new("fullscreen".to_string(), "Fullscreen").accelerator("F11"),
                )
                .add_native_item(MenuItem::Separator)
                .add_item(auto_fullscreen), // .add_item(CustomMenuItem::new(
                                            //     "devtools".to_string(),
                                            //     "Toggle dev-tools",
                                            // )),
        ))
        .add_submenu(Submenu::new(
            "Performance",
            Menu::new()
                .add_item(exclusive_fullscreen)
                .add_item(vsync)
                .add_item(unlimited_fps),
        ))
}
