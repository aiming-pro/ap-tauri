use std::{iter::once, os::windows::prelude::OsStrExt};
use tauri::window::PlatformWebview;

pub fn set_user_agent(webview: PlatformWebview) {
    #[cfg(windows)]
    use webview2_com::take_pwstr;
    use webview2_com::Microsoft::Web::WebView2::Win32::ICoreWebView2Settings2;
    use windows::core::Interface;
    use windows::core::PCWSTR;
    use windows::core::PWSTR;

    unsafe {
        let mut pwstr = PWSTR::null();
        let settings: ICoreWebView2Settings2 = webview
            .controller()
            .CoreWebView2()
            .unwrap()
            .Settings()
            .unwrap()
            .cast()
            .unwrap();

        settings.UserAgent(&mut pwstr).unwrap();
        let mut user_agent = take_pwstr(pwstr);
        user_agent.push_str(" aiming-pro-client");

        settings
            .SetUserAgent(PCWSTR::from_raw(encode_wide(user_agent).as_ptr()))
            .unwrap();
    }
}

fn encode_wide(string: impl AsRef<std::ffi::OsStr>) -> Vec<u16> {
    string.as_ref().encode_wide().chain(once(0)).collect()
}
