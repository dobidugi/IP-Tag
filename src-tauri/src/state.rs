use std::sync::{Arc, Mutex};

use serde::Serialize;
use tokio::sync::Notify;

/// 현재 네트워크 상태. 프론트엔드로 그대로 emit 되고, 트레이 타이틀의 소스가 된다.
#[derive(Clone, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct NetStatus {
    /// 공인 IP (조회 실패 시 None)
    pub ip: Option<String>,
    /// 접속 중인 WiFi 이름 (없거나 못 읽으면 None)
    pub ssid: Option<String>,
    /// IP가 매핑 테이블에 등록돼 있으면 그 별칭
    pub alias: Option<String>,
    /// 트레이/UI에 실제로 표시할 최종 라벨 (별칭 > SSID > IP 우선순위)
    pub label: String,
    /// 마지막 조회 성공 여부
    pub ok: bool,
    /// 마지막 갱신 시각 (unix epoch millis)
    pub updated_at: u64,
}

/// 앱 전역 상태.
pub struct AppState {
    pub status: Mutex<NetStatus>,
    /// 즉시 재조회를 깨우는 신호 (메뉴 "새로고침" / refresh_now 커맨드)
    pub refresh: Arc<Notify>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            status: Mutex::new(NetStatus::default()),
            refresh: Arc::new(Notify::new()),
        }
    }
}
