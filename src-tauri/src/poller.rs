use std::time::{Duration, SystemTime, UNIX_EPOCH};

use tauri::{Emitter, Manager, Runtime};

use crate::mappings;
use crate::net;
use crate::state::{AppState, NetStatus};

/// 폴링 주기 (초)
const POLL_SECS: u64 = 45;

/// 백그라운드 폴링 태스크를 띄운다.
pub fn spawn<R: Runtime>(app: tauri::AppHandle<R>) {
    let notify = app.state::<AppState>().refresh.clone();

    tauri::async_runtime::spawn(async move {
        let mut ticker = tokio::time::interval(Duration::from_secs(POLL_SECS));
        loop {
            // 첫 tick은 즉시 발생하므로 시작하자마자 1회 조회한다.
            tokio::select! {
                _ = ticker.tick() => {}
                _ = notify.notified() => {}
            }
            poll_once(&app).await;
        }
    });
}

/// 1회 조회: 공인 IP + SSID를 읽고, 매핑으로 별칭을 찾아 라벨을 결정한 뒤
/// 상태 저장 → (변경 시) 트레이 타이틀 갱신 → 프론트로 emit.
pub async fn poll_once<R: Runtime>(app: &tauri::AppHandle<R>) {
    let prev = app.state::<AppState>().status.lock().unwrap().clone();

    let fetched = net::fetch_public_ip().await;
    // SSID 조회는 외부 프로세스를 띄우므로 블로킹 스레드에서 실행한다.
    let ssid = tokio::task::spawn_blocking(net::current_ssid)
        .await
        .ok()
        .flatten();

    // 이번 조회 성공 여부. 실패해도 직전 IP를 유지해 라벨 깜빡임을 막는다.
    let ok = fetched.is_some();
    let ip = fetched.or(prev.ip);

    // 별칭: IP가 매핑 테이블에 정확히 매칭되면 사용 (없으면 None)
    let alias = ip
        .as_deref()
        .and_then(|ip| mappings::alias_for_ip(app, ip));

    let label = resolve_label(&alias, &ssid, &ip);
    // 갱신 시각은 성공했을 때만 갱신한다.
    let updated_at = if ok { now_millis() } else { prev.updated_at };

    let status = NetStatus {
        ip,
        ssid,
        alias,
        label: label.clone(),
        ok,
        updated_at,
    };

    // 상태 저장 + 라벨 변경 여부 판단
    let changed = {
        let state = app.state::<AppState>();
        let mut guard = state.status.lock().unwrap();
        let changed = guard.label != status.label;
        *guard = status.clone();
        changed
    };

    // 트레이 타이틀은 라벨이 바뀐 경우에만 갱신한다.
    if changed {
        if let Some(tray) = app.tray_by_id("main-tray") {
            let _ = tray.set_title(Some(&label));
        }
    }

    // 프론트엔드는 매 조회마다 갱신 (마지막 갱신 시각 표시 등)
    let _ = app.emit("net-status", status);
}

/// 표시 라벨 우선순위: 별칭 > SSID(WiFi 이름) > 공인 IP > "조회 실패"
fn resolve_label(alias: &Option<String>, ssid: &Option<String>, ip: &Option<String>) -> String {
    alias
        .clone()
        .or_else(|| ssid.clone())
        .or_else(|| ip.clone())
        .unwrap_or_else(|| "조회 실패".to_string())
}

fn now_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}
