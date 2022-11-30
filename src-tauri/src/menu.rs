use tauri::{CustomMenuItem, Menu, MenuItem, Submenu};

use crate::store::Settings;

pub fn create_menu(settings: Settings) -> Menu {
    let mut vsync = CustomMenuItem::new("disable-vsync".to_string(), "V-Sync");
    if settings.vsync {
        vsync = vsync.selected();
    }

    let mut unlimited_fps =
        CustomMenuItem::new("disable-fps-limit".to_string(), "Unlimited Framerate");
    if settings.unlimited_fps {
        unlimited_fps = unlimited_fps.selected();
    }

    Menu::new()
        .add_submenu(Submenu::new(
            "Edit",
            Menu::new()
                .add_native_item(MenuItem::Undo)
                .add_native_item(MenuItem::Redo)
                .add_native_item(MenuItem::Separator)
                .add_native_item(MenuItem::Cut)
                .add_native_item(MenuItem::Copy)
                .add_native_item(MenuItem::Paste)
                .add_native_item(MenuItem::SelectAll),
        ))
        .add_submenu(Submenu::new(
            "View",
            Menu::new().add_item(CustomMenuItem::new("reload".to_string(), "Reload")),
        ))
        .add_submenu(Submenu::new(
            "Game",
            Menu::new()
                .add_item(
                    CustomMenuItem::new(
                        "id".to_string(),
                        concat!("App Version ", env!("CARGO_PKG_VERSION")),
                    )
                    .disabled(),
                )
                .add_native_item(MenuItem::Separator)
                .add_item(vsync)
                .add_item(unlimited_fps)
                .add_native_item(MenuItem::Separator)
                .add_native_item(MenuItem::Quit),
        ))
}
