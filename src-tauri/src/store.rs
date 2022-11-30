use std::{error::Error, path::PathBuf};

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Runtime};
use tokio::{
    fs::{create_dir_all, read, File},
    io::AsyncWriteExt,
};

#[derive(Serialize, Deserialize, Clone)]
pub struct Settings {
    #[serde(rename = "unlimitedfps")]
    pub unlimited_fps: bool,
    #[serde(rename = "fullscreenOnGameStart")]
    pub fullscreen_on_game_start: bool,
    pub vsync: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            unlimited_fps: true,
            fullscreen_on_game_start: true,
            vsync: false,
        }
    }
}

fn get_config_path<R: Runtime>(app: &AppHandle<R>) -> PathBuf {
    let dir = app
        .path_resolver()
        .app_config_dir()
        .expect("Failed to resolve app dir");

    dir.join("config.json")
}

impl Settings {
    pub async fn save<R: Runtime>(&self, app: &AppHandle<R>) -> Result<(), Box<dyn Error>> {
        let store_path = get_config_path(app);

        create_dir_all(store_path.parent().expect("Invalid store path")).await?;

        let mut f = File::create(&store_path).await?;
        f.write_all(&serde_json::to_string(self).unwrap().as_bytes())
            .await?;

        Ok(())
    }

    pub async fn load<R: Runtime>(&mut self, app: &AppHandle<R>) -> Result<(), Box<dyn Error>> {
        let store_path = get_config_path(app);

        let bytes = read(&store_path).await?;

        *self = serde_json::from_slice::<Settings>(&bytes).unwrap();

        Ok(())
    }
}
