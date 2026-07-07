use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    App, AppHandle, Manager, PhysicalPosition, Runtime,
};

use crate::state::AppState;

/// 언어 코드별 트레이 메뉴 라벨 (새로고침, 종료).
fn menu_labels(lang: &str) -> (&'static str, &'static str) {
    match lang {
        "en" => ("Refresh", "Quit"),
        "ja" => ("更新", "終了"),
        _ => ("새로고침", "종료"),
    }
}

/// 선택한 언어로 트레이 메뉴를 만든다. (라벨은 언어별로 번역)
fn build_menu<M: Manager<R>, R: Runtime>(app: &M, lang: &str) -> tauri::Result<Menu<R>> {
    let (refresh_label, quit_label) = menu_labels(lang);
    let refresh = MenuItem::with_id(app, "refresh", refresh_label, true, None::<&str>)?;
    let sep = PredefinedMenuItem::separator(app)?;
    let quit = MenuItem::with_id(app, "quit", quit_label, true, None::<&str>)?;
    Menu::with_items(app, &[&refresh, &sep, &quit])
}

/// 프론트에서 선택한 언어를 트레이 메뉴에 반영한다.
/// (메뉴 이벤트 핸들러는 트레이에 붙어 있으므로 메뉴를 교체해도 유지된다.)
pub fn apply_language<R: Runtime>(app: &AppHandle<R>, lang: &str) {
    let Some(tray) = app.tray_by_id("main-tray") else {
        return;
    };
    if let Ok(menu) = build_menu(app, lang) {
        let _ = tray.set_menu(Some(menu));
    }
}

/// 메뉴바(트레이) 아이콘을 등록한다.
/// - 좌클릭: 팝오버 윈도우 토글
/// - 우클릭: 메뉴(새로고침 / 종료)
pub fn setup_tray<R: Runtime>(app: &App<R>) -> tauri::Result<()> {
    // 시작 시엔 기본(한국어) 라벨로 만들고, 프론트가 뜨면 저장된 언어로 교정한다.
    let menu = build_menu(app, "ko")?;

    TrayIconBuilder::with_id("main-tray")
        .icon(app.default_window_icon().unwrap().clone())
        // macOS 메뉴바 색(라이트/다크)에 맞춰 자동 반전되도록 템플릿 이미지로 처리
        .icon_as_template(true)
        .title("…")
        .tooltip("IP Tag")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "quit" => app.exit(0),
            "refresh" => {
                app.state::<AppState>().refresh.notify_one();
            }
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                position,
                ..
            } = event
            {
                toggle_popover(tray.app_handle(), position);
            }
        })
        .build(app)?;

    Ok(())
}

/// 트레이 아이콘 클릭 위치 아래로 팝오버 윈도우를 띄우거나 숨긴다.
fn toggle_popover<R: Runtime>(app: &tauri::AppHandle<R>, cursor: PhysicalPosition<f64>) {
    let Some(win) = app.get_webview_window("main") else {
        return;
    };

    if win.is_visible().unwrap_or(false) {
        let _ = win.hide();
        return;
    }

    // 커서(트레이 아이콘 근처) 중앙 아래에 창을 배치한다.
    if let Ok(size) = win.outer_size() {
        let x = (cursor.x - size.width as f64 / 2.0).max(8.0);
        let y = cursor.y + 8.0;
        let _ = win.set_position(PhysicalPosition::new(x, y));
    }
    let _ = win.show();
    let _ = win.set_focus();
}
