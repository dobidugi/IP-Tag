import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

/** 백엔드 NetStatus (serde camelCase) */
export interface NetStatus {
  ip: string | null;
  ssid: string | null;
  alias: string | null;
  label: string;
  ok: boolean;
  updatedAt: number;
}

/** IP ↔ 별칭 매핑 */
export interface Mapping {
  id: string;
  ip: string;
  alias: string;
}

export const getStatus = () => invoke<NetStatus>("get_status");
export const refreshNow = () => invoke<void>("refresh_now");

export const getMappings = () => invoke<Mapping[]>("get_mappings");
export const addMapping = (ip: string, alias: string) =>
  invoke<Mapping[]>("add_mapping", { ip, alias });
export const updateMapping = (id: string, ip: string, alias: string) =>
  invoke<Mapping[]>("update_mapping", { id, ip, alias });
export const deleteMapping = (id: string) =>
  invoke<Mapping[]>("delete_mapping", { id });

/** poller가 매 조회마다 emit 하는 net-status 이벤트 구독 */
export const onNetStatus = (cb: (s: NetStatus) => void): Promise<UnlistenFn> =>
  listen<NetStatus>("net-status", (e) => cb(e.payload));
