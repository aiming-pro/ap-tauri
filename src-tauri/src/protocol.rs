use std::str::FromStr;

use crate::constants;
use tauri::Window;

pub enum ProtocolAction {
    Game,
    Page,
    Playlist,
}

impl FromStr for ProtocolAction {
    type Err = ();

    fn from_str(input: &str) -> Result<ProtocolAction, Self::Err> {
        match input {
            "game" => Ok(ProtocolAction::Game),
            "page" => Ok(ProtocolAction::Page),
            "playlist" => Ok(ProtocolAction::Playlist),
            _ => Err(()),
        }
    }
}

pub struct ProtocolType {
    pub action: ProtocolAction,
    pub parameter: i64,
}

impl FromStr for ProtocolType {
    type Err = ();

    fn from_str(input: &str) -> Result<ProtocolType, Self::Err> {
        let prefix = constants::PROTOCOL_PREFIX;
        let path = input.strip_prefix(&format!("{prefix}://")).ok_or(())?;

        let (action_str, parameter_str) = path.split_once('/').ok_or(())?;

        let action: ProtocolAction = action_str.parse().map_or(Err(()), Ok)?;
        let parameter: i64 = parameter_str.parse().map_or(Err(()), Ok)?;

        Ok(ProtocolType { action, parameter })
    }
}

fn generate_redirect_script(base_url: &str, parameter: &str) -> String {
    format!("const loggedIn = document.cookie.split('; ').some(c => c.startsWith('user_basic')); window.location.replace(loggedIn ? '{}{}' : '{}')", base_url, parameter, constants::LOGIN_URL)
}

impl ProtocolType {
    pub fn activate(&self, window: &Window) {
        window.set_focus().ok();
        match self.action {
            ProtocolAction::Game => {
                window
                    .eval(&generate_redirect_script(
                        constants::GAME_BASE_URL,
                        &self.parameter.to_string(),
                    ))
                    .ok();
            }
            ProtocolAction::Playlist => {
                window
                    .eval(&generate_redirect_script(
                        constants::PLAYLIST_BASE_URL,
                        &self.parameter.to_string(),
                    ))
                    .ok();
            }
            ProtocolAction::Page => {}
        }
    }
}
