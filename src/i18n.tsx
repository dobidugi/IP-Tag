import {
  createContext,
  useContext,
  useEffect,
  useState,
  type ReactNode,
} from "react";
import { invoke } from "@tauri-apps/api/core";
import { load, type Store } from "@tauri-apps/plugin-store";

/** 지원 언어 코드 */
export type Lang = "ko" | "en" | "ja";

/** 설정 UI의 언어 선택 목록 (각 언어는 자기 언어로 표기) */
export const LANGUAGES: { code: Lang; label: string }[] = [
  { code: "ko", label: "한국어" },
  { code: "en", label: "English" },
  { code: "ja", label: "日本語" },
];

/** UI에서 쓰는 모든 문자열 키 */
interface Dict {
  badgeAlias: string;
  metaPublicIp: string;
  metaWifi: string;
  metaUpdated: string;
  lookupFailed: string;
  refresh: string;
  refreshing: string;
  mappingsTitle: string;
  useCurrentIp: string;
  useCurrentIpTitle: string;
  aliasPlaceholder: string;
  add: string;
  save: string;
  cancel: string;
  noMappings: string;
  edit: string;
  delete: string;
  autostart: string;
  language: string;
}

const translations: Record<Lang, Dict> = {
  ko: {
    badgeAlias: "별칭",
    metaPublicIp: "공인 IP",
    metaWifi: "WiFi",
    metaUpdated: "갱신",
    lookupFailed: "조회 실패",
    refresh: "새로고침",
    refreshing: "조회 중…",
    mappingsTitle: "별칭 매핑",
    useCurrentIp: "현재 IP",
    useCurrentIpTitle: "현재 공인 IP 채우기",
    aliasPlaceholder: "별칭 (예: 본사 VPN)",
    add: "추가",
    save: "수정",
    cancel: "취소",
    noMappings: "등록된 매핑이 없습니다.",
    edit: "수정",
    delete: "삭제",
    autostart: "로그인 시 자동 실행",
    language: "언어",
  },
  en: {
    badgeAlias: "Alias",
    metaPublicIp: "Public IP",
    metaWifi: "Wi-Fi",
    metaUpdated: "Updated",
    lookupFailed: "Lookup failed",
    refresh: "Refresh",
    refreshing: "Checking…",
    mappingsTitle: "Alias mappings",
    useCurrentIp: "Current IP",
    useCurrentIpTitle: "Fill in current public IP",
    aliasPlaceholder: "Alias (e.g. Office VPN)",
    add: "Add",
    save: "Save",
    cancel: "Cancel",
    noMappings: "No mappings yet.",
    edit: "Edit",
    delete: "Delete",
    autostart: "Launch at login",
    language: "Language",
  },
  ja: {
    badgeAlias: "エイリアス",
    metaPublicIp: "パブリックIP",
    metaWifi: "Wi-Fi",
    metaUpdated: "更新",
    lookupFailed: "取得失敗",
    refresh: "更新",
    refreshing: "取得中…",
    mappingsTitle: "エイリアス設定",
    useCurrentIp: "現在のIP",
    useCurrentIpTitle: "現在のパブリックIPを入力",
    aliasPlaceholder: "エイリアス (例: 本社VPN)",
    add: "追加",
    save: "保存",
    cancel: "キャンセル",
    noMappings: "登録された設定がありません。",
    edit: "編集",
    delete: "削除",
    autostart: "ログイン時に自動起動",
    language: "言語",
  },
};

const STORE_FILE = "settings.json";
const LANG_KEY = "language";

/** 저장된 값이 없을 때 시스템 언어로 초기 언어를 추정한다. */
function detectLang(): Lang {
  const n = navigator.language.toLowerCase();
  if (n.startsWith("ko")) return "ko";
  if (n.startsWith("ja")) return "ja";
  return "en";
}

interface I18nValue {
  lang: Lang;
  setLang: (next: Lang) => void;
  t: (key: keyof Dict) => string;
}

const I18nContext = createContext<I18nValue>({
  lang: "ko",
  setLang: () => {},
  t: (key) => translations.ko[key],
});

export function I18nProvider({ children }: { children: ReactNode }) {
  const [lang, setLangState] = useState<Lang>("ko");
  const [store, setStore] = useState<Store | null>(null);

  // 저장된 언어를 읽어 초기화하고, 트레이 메뉴 언어도 맞춘다.
  useEffect(() => {
    (async () => {
      const s = await load(STORE_FILE);
      setStore(s);
      const saved = (await s.get<Lang>(LANG_KEY)) ?? null;
      const initial = saved && translations[saved] ? saved : detectLang();
      setLangState(initial);
      invoke("set_language", { lang: initial }).catch(() => {});
    })().catch(() => {});
  }, []);

  function setLang(next: Lang) {
    setLangState(next);
    invoke("set_language", { lang: next }).catch(() => {});
    if (store) {
      store.set(LANG_KEY, next);
      store.save().catch(() => {});
    }
  }

  const t = (key: keyof Dict) => translations[lang][key];

  return (
    <I18nContext.Provider value={{ lang, setLang, t }}>
      {children}
    </I18nContext.Provider>
  );
}

export function useI18n() {
  return useContext(I18nContext);
}
