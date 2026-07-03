use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    App, Manager, PhysicalPosition, Runtime,
};

/// 메뉴바(트레이) 아이콘을 등록한다.
/// - 좌클릭: 팝오버 윈도우 토글
/// - 우클릭: 메뉴(새로고침 / 종료)
pub fn setup_tray<R: Runtime>(app: &App<R>) -> tauri::Result<()> {
    let refresh = MenuItem::with_id(app, "refresh", "새로고침", true, None::<&str>)?;
    let sep = PredefinedMenuItem::separator(app)?;
    let quit = MenuItem::with_id(app, "quit", "종료", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&refresh, &sep, &quit])?;

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
                // 3단계에서 즉시 폴링 트리거를 연결한다.
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
