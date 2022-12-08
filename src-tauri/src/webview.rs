use std::{iter::once, os::windows::prelude::OsStrExt};
use tauri::window::PlatformWebview;

use crate::constants;

pub fn set_user_agent(webview: &PlatformWebview) {
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
        user_agent.push_str(constants::USER_AGENT_SUFFIX);

        settings
            .SetUserAgent(PCWSTR::from_raw(encode_wide(user_agent).as_ptr()))
            .unwrap();
    }
}

fn encode_wide(string: impl AsRef<std::ffi::OsStr>) -> Vec<u16> {
    string.as_ref().encode_wide().chain(once(0)).collect()
}

pub fn disable_new_windows(webview: &PlatformWebview) {
    use webview2_com::take_pwstr;
    use webview2_com::Microsoft::Web::WebView2::Win32::ICoreWebView2NewWindowRequestedEventHandler;
    use webview2_com::NewWindowRequestedEventHandler;
    use windows::core::PWSTR;
    use windows::Win32::System::WinRT::EventRegistrationToken;

    let mut token = EventRegistrationToken::default();

    let callback: ICoreWebView2NewWindowRequestedEventHandler =
        NewWindowRequestedEventHandler::create(Box::new(move |_, args| {
            if let Some(args) = args {
                let mut pwstr = PWSTR::null();

                unsafe {
                    // Cancel open window event
                    args.SetHandled(true)?;
                    args.Uri(&mut pwstr)?;
                }

                let uri = take_pwstr(pwstr);

                println!("{uri}");
                open::that(uri).ok();
            }
            Ok(())
        }));

    unsafe {
        let webview2 = webview.controller().CoreWebView2().unwrap();

        webview2
            .add_NewWindowRequested(&callback, &mut token)
            .unwrap();
    }
}
