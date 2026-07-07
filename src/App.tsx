import { useEffect, useState } from "react";
import { disable, enable, isEnabled } from "@tauri-apps/plugin-autostart";
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
import { LANGUAGES, useI18n } from "./i18n";
import "./App.css";

type LabelKind = "alias" | "wifi" | "ip" | "none";

/** 라벨이 어디서 왔는지(별칭/WiFi/IP) 배지로 보여주기 위한 판별 */
function labelKind(s: NetStatus): { kind: LabelKind; cls: string } {
  if (s.alias) return { kind: "alias", cls: "badge-alias" };
  if (s.ssid && s.label === s.ssid) return { kind: "wifi", cls: "badge-wifi" };
  if (s.ip && s.label === s.ip) return { kind: "ip", cls: "badge-ip" };
  return { kind: "none", cls: "badge-none" };
}

function formatTime(ms: number, locale: string): string {
  if (!ms) return "—";
  const d = new Date(ms);
  return d.toLocaleTimeString(locale, { hour12: false });
}

function App() {
  const { lang, setLang, t } = useI18n();
  const [status, setStatus] = useState<NetStatus | null>(null);
  const [mappings, setMappings] = useState<Mapping[]>([]);
  const [ip, setIp] = useState("");
  const [alias, setAlias] = useState("");
  const [editingId, setEditingId] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [refreshing, setRefreshing] = useState(false);
  const [autostart, setAutostart] = useState(false);

  useEffect(() => {
    getStatus().then(setStatus).catch(() => {});
    getMappings().then(setMappings).catch(() => {});
    isEnabled().then(setAutostart).catch(() => {});
    const unlisten = onNetStatus(setStatus);
    return () => {
      unlisten.then((fn) => fn());
    };
  }, []);

  async function toggleAutostart() {
    try {
      if (autostart) {
        await disable();
        setAutostart(false);
      } else {
        await enable();
        setAutostart(true);
      }
    } catch (err) {
      setError(String(err));
    }
  }

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
  const badgeText =
    kind?.kind === "alias"
      ? t("badgeAlias")
      : kind?.kind === "wifi"
        ? "WiFi"
        : kind?.kind === "ip"
          ? "IP"
          : "—";
  const locale = lang === "ko" ? "ko-KR" : lang === "ja" ? "ja-JP" : "en-US";

  return (
    <div className="popover">
      <header className="status-card">
        <div className="status-main">
          <span className={`badge ${kind?.cls ?? "badge-none"}`}>
            {badgeText}
          </span>
          <span className="status-label" title={status?.label}>
            {status?.label ?? "…"}
          </span>
          <span className={`dot ${status?.ok ? "dot-ok" : "dot-bad"}`} />
        </div>
        <dl className="status-meta">
          <div>
            <dt>{t("metaPublicIp")}</dt>
            <dd>{status?.ip ?? t("lookupFailed")}</dd>
          </div>
          <div>
            <dt>{t("metaWifi")}</dt>
            <dd>{status?.ssid ?? "—"}</dd>
          </div>
          <div>
            <dt>{t("metaUpdated")}</dt>
            <dd>{formatTime(status?.updatedAt ?? 0, locale)}</dd>
          </div>
        </dl>
        <button
          className="refresh-btn"
          onClick={handleRefresh}
          disabled={refreshing}
        >
          {refreshing ? t("refreshing") : t("refresh")}
        </button>
      </header>

      <section className="mappings">
        <h2>{t("mappingsTitle")}</h2>

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
              title={t("useCurrentIpTitle")}
            >
              {t("useCurrentIp")}
            </button>
          </div>
          <input
            placeholder={t("aliasPlaceholder")}
            value={alias}
            onChange={(e) => setAlias(e.target.value)}
          />
          <div className="form-actions">
            <button type="submit" className="primary">
              {editingId ? t("save") : t("add")}
            </button>
            {editingId && (
              <button type="button" onClick={resetForm}>
                {t("cancel")}
              </button>
            )}
          </div>
        </form>

        {error && <p className="error">{error}</p>}

        <ul className="mapping-list">
          {mappings.length === 0 && (
            <li className="empty">{t("noMappings")}</li>
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
                  {t("edit")}
                </button>
                <button
                  className="mini-btn danger"
                  onClick={() => handleDelete(m.id)}
                >
                  {t("delete")}
                </button>
              </div>
            </li>
          ))}
        </ul>
      </section>

      <footer className="footer">
        <label className="autostart">
          <input
            type="checkbox"
            checked={autostart}
            onChange={toggleAutostart}
          />
          {t("autostart")}
        </label>
        <label className="lang-select">
          <span>{t("language")}</span>
          <select
            value={lang}
            onChange={(e) => setLang(e.target.value as (typeof LANGUAGES)[number]["code"])}
          >
            {LANGUAGES.map((l) => (
              <option key={l.code} value={l.code}>
                {l.label}
              </option>
            ))}
          </select>
        </label>
      </footer>
    </div>
  );
}

export default App;
