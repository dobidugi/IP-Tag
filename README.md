# IP Tag

> 지금 내 공인 IP가 **어디**인지 메뉴바에서 한눈에.

macOS 메뉴바에 상주하면서 현재 공인 IP와 WiFi를 주기적으로 확인하고,
IP에 직접 붙여둔 **별칭**으로 "여기가 어디인지"를 바로 알려주는 작은 트레이 앱입니다.

---

## 왜 만들었나

원격/재택으로 여러 네트워크를 오가다 보면 "지금 접속된 곳이 집인지, 회사인지, VPN인지"가
헷갈립니다. IP는 `123.45.67.89` 같은 숫자라 봐도 감이 안 오고요.

그래서 자주 쓰는 IP에 **`본사 VPN`, `집`, `카페`** 같은 별칭을 미리 매핑해두고,
메뉴바만 보면 지금 위치가 어디인지 바로 읽히도록 만들었습니다.

- 방화벽/화이트리스트 걸린 서비스에 접속 전, 지금 IP가 등록된 곳인지 확인
- VPN이 실제로 붙었는지 눈으로 확인
- 네트워크가 바뀌면 자동으로 라벨도 바뀜

---

## 기능

- **IP → 별칭 매핑** — IP마다 원하는 이름을 붙이고 CRUD (추가/수정/삭제)
- **자동 폴링** — 45초마다 공인 IP + WiFi SSID를 백그라운드에서 갱신
- **똑똑한 라벨** — 표시 우선순위: `별칭 > WiFi 이름 > 공인 IP`
- **메뉴바 상주** — Dock에 안 뜨는 Accessory 앱, 트레이 아이콘 클릭으로 팝오버
- **오프라인 내성** — IP 조회 실패해도 죽지 않고 상태만 표시, 복구 시 자동 반영
- **다국어 지원** — 한국어 / English / 日本語, 설정에서 전환 (트레이 메뉴까지 반영)
- **로그인 시 자동 실행** 토글
- **다크모드** 대응

---

## 동작 방식

| 항목 | 방식 |
|------|------|
| 공인 IP | `api.ipify.org` 조회, 실패 시 `icanhazip.com`으로 폴백 |
| WiFi SSID | `ipconfig getsummary <iface>` — 위치 권한 없이 SSID 노출 |
| 매핑 저장 | `tauri-plugin-store` (로컬 JSON) |
| 갱신 알림 | 폴러가 매 조회마다 `net-status` 이벤트를 프론트로 emit |

프론트(React)는 이벤트를 구독만 하고, 실제 조회·저장·라벨 결정은 전부 Rust 백엔드에서 합니다.

---

## 기술 스택

- **[Tauri v2](https://tauri.app/)** (Rust) — 트레이 / 폴링 / 네트워크 / 저장
- **React + TypeScript + Vite** — 팝오버 UI
- 플러그인: `store`, `autostart`, `opener`

---

## 개발

```bash
# 의존성 설치
npm install

# 개발 모드 (핫리로드)
npm run tauri dev

# 릴리즈 빌드 (.app / .dmg)
npm run tauri build
```

> Rust 툴체인과 Xcode Command Line Tools가 필요합니다.

---

## 프로젝트 구조

```
src/                  # React 팝오버 UI
  App.tsx             # IP 카드 + 매핑 CRUD 화면
  api.ts              # invoke / 이벤트 바인딩
  i18n.tsx            # 다국어 사전 + 언어 컨텍스트 (ko/en/ja)

src-tauri/src/
  poller.rs           # 45초 주기 폴링 + 라벨 결정 + 이벤트 emit
  net.rs              # 공인 IP / WiFi SSID 조회
  mappings.rs         # IP-별칭 매핑 저장/조회 (store)
  tray.rs             # 메뉴바 트레이 아이콘 + 팝오버
  state.rs            # 공유 상태
  lib.rs              # 커맨드 등록 / 앱 초기화
```
