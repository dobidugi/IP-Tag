use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager, Runtime};
use tauri_plugin_store::StoreExt;

use crate::state::AppState;

const STORE_FILE: &str = "mappings.json";
const KEY: &str = "mappings";

/// 공인 IP ↔ 별칭 매핑 한 건.
#[derive(Clone, Serialize, Deserialize)]
pub struct Mapping {
    pub id: String,
    pub ip: String,
    pub alias: String,
}

/// 저장소에서 매핑 목록을 읽는다. 없거나 파싱 실패 시 빈 목록.
fn load<R: Runtime>(app: &AppHandle<R>) -> Vec<Mapping> {
    let Ok(store) = app.store(STORE_FILE) else {
        return Vec::new();
    };
    store
        .get(KEY)
        .and_then(|v| serde_json::from_value::<Vec<Mapping>>(v).ok())
        .unwrap_or_default()
}

/// 매핑 목록을 저장소에 기록하고 디스크에 저장한다.
fn persist<R: Runtime>(app: &AppHandle<R>, list: &[Mapping]) -> Result<(), String> {
    let store = app.store(STORE_FILE).map_err(|e| e.to_string())?;
    let value = serde_json::to_value(list).map_err(|e| e.to_string())?;
    store.set(KEY, value);
    store.save().map_err(|e| e.to_string())?;
    Ok(())
}

/// IP에 정확히 매칭되는 별칭을 찾는다. (CIDR 대역 매칭은 추후 확장 여지)
pub fn alias_for_ip<R: Runtime>(app: &AppHandle<R>, ip: &str) -> Option<String> {
    load(app).into_iter().find(|m| m.ip == ip).map(|m| m.alias)
}

/// 매핑이 바뀌면 라벨이 즉시 갱신되도록 재조회를 깨운다.
fn trigger_refresh<R: Runtime>(app: &AppHandle<R>) {
    app.state::<AppState>().refresh.notify_one();
}

/// 충돌 가능성이 낮은 간단한 id (nanos 기반).
fn new_id() -> String {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    format!("{nanos:x}")
}

#[tauri::command]
pub fn get_mappings<R: Runtime>(app: AppHandle<R>) -> Vec<Mapping> {
    load(&app)
}

#[tauri::command]
pub fn add_mapping<R: Runtime>(
    app: AppHandle<R>,
    ip: String,
    alias: String,
) -> Result<Vec<Mapping>, String> {
    let ip = ip.trim().to_string();
    let alias = alias.trim().to_string();
    if ip.is_empty() || alias.is_empty() {
        return Err("IP와 별칭을 모두 입력하세요.".into());
    }
    let mut list = load(&app);
    list.push(Mapping {
        id: new_id(),
        ip,
        alias,
    });
    persist(&app, &list)?;
    trigger_refresh(&app);
    Ok(list)
}

#[tauri::command]
pub fn update_mapping<R: Runtime>(
    app: AppHandle<R>,
    id: String,
    ip: String,
    alias: String,
) -> Result<Vec<Mapping>, String> {
    let ip = ip.trim().to_string();
    let alias = alias.trim().to_string();
    if ip.is_empty() || alias.is_empty() {
        return Err("IP와 별칭을 모두 입력하세요.".into());
    }
    let mut list = load(&app);
    match list.iter_mut().find(|m| m.id == id) {
        Some(m) => {
            m.ip = ip;
            m.alias = alias;
        }
        None => return Err("해당 매핑을 찾을 수 없습니다.".into()),
    }
    persist(&app, &list)?;
    trigger_refresh(&app);
    Ok(list)
}

#[tauri::command]
pub fn delete_mapping<R: Runtime>(app: AppHandle<R>, id: String) -> Result<Vec<Mapping>, String> {
    let mut list = load(&app);
    list.retain(|m| m.id != id);
    persist(&app, &list)?;
    trigger_refresh(&app);
    Ok(list)
}
