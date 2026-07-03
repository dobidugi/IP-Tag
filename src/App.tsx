import { useEffect, useState } from "react";
import {
  type Mapping,
  type NetStatus,
  addMapping,
  deleteMapping,
  getMappings,
  getStatus,
  onNetStatus,
  refreshNow,
  updateMapping,
} from "./api";
import "./App.css";

/** 라벨이 어디서 왔는지(별칭/WiFi/IP) 배지로 보여주기 위한 판별 */
function labelKind(s: NetStatus): { text: string; cls: string } {
  if (s.alias) return { text: "별칭", cls: "badge-alias" };
  if (s.ssid && s.label === s.ssid) return { text: "WiFi", cls: "badge-wifi" };
  if (s.ip && s.label === s.ip) return { text: "IP", cls: "badge-ip" };
  return { text: "—", cls: "badge-none" };
}

function formatTime(ms: number): string {
  if (!ms) return "—";
  const d = new Date(ms);
  return d.toLocaleTimeString("ko-KR", { hour12: false });
}

function App() {
  const [status, setStatus] = useState<NetStatus | null>(null);
  const [mappings, setMappings] = useState<Mapping[]>([]);
  const [ip, setIp] = useState("");
  const [alias, setAlias] = useState("");
  const [editingId, setEditingId] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [refreshing, setRefreshing] = useState(false);

  useEffect(() => {
    getStatus().then(setStatus).catch(() => {});
    getMappings().then(setMappings).catch(() => {});
    const unlisten = onNetStatus(setStatus);
    return () => {
      unlisten.then((fn) => fn());
    };
  }, []);

  async function handleRefresh() {
    setRefreshing(true);
    try {
      await refreshNow();
    } finally {
      // 백엔드 조회가 끝나면 net-status 이벤트로 갱신되므로 짧게만 표시
      setTimeout(() => setRefreshing(false), 800);
    }
  }

  function resetForm() {
    setIp("");
    setAlias("");
    setEditingId(null);
    setError(null);
  }

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault();
    setError(null);
    try {
      const next = editingId
        ? await updateMapping(editingId, ip, alias)
        : await addMapping(ip, alias);
      setMappings(next);
      resetForm();
    } catch (err) {
      setError(String(err));
    }
  }

  async function handleDelete(id: string) {
    try {
      setMappings(await deleteMapping(id));
      if (editingId === id) resetForm();
    } catch (err) {
      setError(String(err));
    }
  }

  function startEdit(m: Mapping) {
    setEditingId(m.id);
    setIp(m.ip);
    setAlias(m.alias);
    setError(null);
  }

  function useCurrentIp() {
    if (status?.ip) setIp(status.ip);
  }

  const kind = status ? labelKind(status) : null;

  return (
    <div className="popover" data-tauri-drag-region>
      <header className="status-card">
        <div className="status-main">
          <span className={`badge ${kind?.cls ?? "badge-none"}`}>
            {kind?.text ?? "—"}
          </span>
          <span className="status-label" title={status?.label}>
            {status?.label ?? "…"}
          </span>
          <span className={`dot ${status?.ok ? "dot-ok" : "dot-bad"}`} />
        </div>
        <dl className="status-meta">
          <div>
            <dt>공인 IP</dt>
            <dd>{status?.ip ?? "조회 실패"}</dd>
          </div>
          <div>
            <dt>WiFi</dt>
            <dd>{status?.ssid ?? "—"}</dd>
          </div>
          <div>
            <dt>갱신</dt>
            <dd>{formatTime(status?.updatedAt ?? 0)}</dd>
          </div>
        </dl>
        <button
          className="refresh-btn"
          onClick={handleRefresh}
          disabled={refreshing}
        >
          {refreshing ? "조회 중…" : "새로고침"}
        </button>
      </header>

      <section className="mappings">
        <h2>별칭 매핑</h2>

        <form className="mapping-form" onSubmit={handleSubmit}>
          <div className="ip-row">
            <input
              className="input-ip"
              placeholder="123.45.67.89"
              value={ip}
              onChange={(e) => setIp(e.target.value)}
            />
            <button
              type="button"
              className="mini-btn"
              onClick={useCurrentIp}
              title="현재 공인 IP 채우기"
            >
              현재 IP
            </button>
          </div>
          <input
            placeholder="별칭 (예: 본사 VPN)"
            value={alias}
            onChange={(e) => setAlias(e.target.value)}
          />
          <div className="form-actions">
            <button type="submit" className="primary">
              {editingId ? "수정" : "추가"}
            </button>
            {editingId && (
              <button type="button" onClick={resetForm}>
                취소
              </button>
            )}
          </div>
        </form>

        {error && <p className="error">{error}</p>}

        <ul className="mapping-list">
          {mappings.length === 0 && (
            <li className="empty">등록된 매핑이 없습니다.</li>
          )}
          {mappings.map((m) => (
            <li
              key={m.id}
              className={
                status?.ip === m.ip ? "mapping-item active" : "mapping-item"
              }
            >
              <div className="mapping-text">
                <span className="mapping-alias">{m.alias}</span>
                <span className="mapping-ip">{m.ip}</span>
              </div>
              <div className="mapping-actions">
                <button className="mini-btn" onClick={() => startEdit(m)}>
                  수정
                </button>
                <button
                  className="mini-btn danger"
                  onClick={() => handleDelete(m.id)}
                >
                  삭제
                </button>
              </div>
            </li>
          ))}
        </ul>
      </section>
    </div>
  );
}

export default App;
