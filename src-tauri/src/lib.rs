mod mappings;
mod net;
mod poller;
mod state;
mod tray;

use state::{AppState, NetStatus};
use tauri::{Manager, WindowEvent};

/// 현재 네트워크 상태를 반환한다 (프론트 초기 로드용).
#[tauri::command]
fn get_status(state: tauri::State<AppState>) -> NetStatus {
    state.status.lock().unwrap().clone()
}

/// 즉시 재조회를 트리거한다.
#[tauri::command]
fn refresh_now(state: tauri::State<AppState>) {
    state.refresh.notify_one();
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        .manage(AppState::new())
        .invoke_handler(tauri::generate_handler![
            get_status,
            refresh_now,
            mappings::get_mappings,
            mappings::add_mapping,
            mappings::update_mapping,
            mappings::delete_mapping
        ])
        .setup(|app| {
            // macOS: dock 아이콘을 숨기고 메뉴바 상주 앱으로 동작시킨다.
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);

            // 로그인 시 자동 실행 (LaunchAgent). 실제 on/off는 프론트에서 토글.
            #[cfg(desktop)]
            app.handle().plugin(tauri_plugin_autostart::init(
                tauri_plugin_autostart::MacosLauncher::LaunchAgent,
                None,
            ))?;

            tray::setup_tray(app)?;

            // 팝오버는 포커스를 잃으면 자동으로 닫는다.
            if let Some(win) = app.get_webview_window("main") {
                let win_clone = win.clone();
                win.on_window_event(move |event| {
                    if let WindowEvent::Focused(false) = event {
                        let _ = win_clone.hide();
                    }
                });
            }

            // 백그라운드 IP/SSID 폴링 시작
            poller::spawn(app.handle().clone());

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
