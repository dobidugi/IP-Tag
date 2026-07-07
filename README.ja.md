# IP Tag

[한국어](README.md) · [English](README.en.md) · **日本語**

> いまの自分のパブリックIPが**どこ**なのか、メニューバーでひと目で。

macOS のメニューバーに常駐し、現在のパブリックIPと Wi-Fi を定期的にチェックして、
IP に付けた**エイリアス**で「今どこに繋がっているか」をすぐに教えてくれる小さなトレイアプリです。

<p align="center">
  <img src="resources/image.png" alt="IP Tag スクリーンショット" width="320">
</p>

---

## なぜ作ったか

リモートや在宅で複数のネットワークを行き来していると、「今つながっているのは自宅か、会社か、VPN か」
が分からなくなりがちです。`123.45.67.89` のような数字の IP を見ても、ピンときません。

そこで、よく使う IP に **`本社VPN`、`自宅`、`カフェ`** といったエイリアスをあらかじめ紐づけておき、
メニューバーを見るだけで今いる場所がすぐ読み取れるようにしました。

- ファイアウォール/ホワイトリスト制限のあるサービスに接続する前に、今の IP が登録済みか確認
- VPN が実際に接続されたかを目視で確認
- ネットワークが変わると、ラベルも自動で切り替わる

---

## 機能

- **IP → エイリアス紐づけ** — IP ごとに好きな名前を付けて CRUD（追加/編集/削除）
- **自動ポーリング** — 45秒ごとにパブリックIP + Wi-Fi SSID をバックグラウンドで更新
- **賢いラベル** — 表示の優先順位: `エイリアス > Wi-Fi 名 > パブリックIP`
- **メニューバー常駐** — Dock に出ない Accessory アプリ。トレイアイコンをクリックでポップオーバー
- **オフライン耐性** — IP 取得に失敗しても落ちず、状態のみ表示。復旧時は自動反映
- **多言語対応** — 한국어 / English / 日本語。設定から切り替え（トレイメニューまで反映）
- **ログイン時に自動起動** トグル
- **ダークモード** 対応

---

## 仕組み

| 項目 | 方法 |
|------|------|
| パブリックIP | `api.ipify.org` を照会、失敗時は `icanhazip.com` にフォールバック |
| Wi-Fi SSID | `ipconfig getsummary <iface>` — 位置情報権限なしで SSID を取得 |
| 紐づけ保存 | `tauri-plugin-store`（ローカル JSON） |
| 更新通知 | ポーラーが照会のたびに `net-status` イベントをフロントへ emit |

フロント（React）はイベントを購読するだけで、実際の照会・保存・ラベル判定はすべて Rust バックエンドが行います。

---

## 技術スタック

- **[Tauri v2](https://tauri.app/)**（Rust）— トレイ / ポーリング / ネットワーク / 保存
- **React + TypeScript + Vite** — ポップオーバー UI
- プラグイン: `store`、`autostart`、`opener`

---

## 事前準備

- **Node.js**（18+）
- **Xcode Command Line Tools** — `xcode-select --install`
- **Rust ツールチェーン** — 未インストールの場合:

  ```bash
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
  ```

> **zsh 利用者への注意:** rustup は PATH 設定を `~/.profile` に書き込みますが、zsh はこのファイルを読み込みません。
> 新しいターミナルで `cargo --version` が失敗する場合は、次の一行を `~/.zshrc` に追加してください。
>
> ```bash
> echo '. "$HOME/.cargo/env"' >> ~/.zshrc && source ~/.zshrc
> ```
>
> （これを行わないと `npm run tauri dev` 実行時に `cargo metadata ... No such file or directory` エラーになります。）

## 開発

```bash
# 依存関係のインストール
npm install

# 開発モード（ホットリロード）
npm run tauri dev

# リリースビルド（.app / .dmg）
npm run tauri build
```

---

## プロジェクト構成

```
src/                  # React ポップオーバー UI
  App.tsx             # IP カード + 紐づけ CRUD 画面
  api.ts              # invoke / イベントバインディング
  i18n.tsx            # 多言語辞書 + 言語コンテキスト（ko/en/ja）

src-tauri/src/
  poller.rs           # 45秒周期のポーリング + ラベル判定 + イベント emit
  net.rs              # パブリックIP / Wi-Fi SSID 取得
  mappings.rs         # IP-エイリアス紐づけ保存（store）
  tray.rs             # メニューバーのトレイアイコン + ポップオーバー
  state.rs            # 共有状態
  lib.rs              # コマンド登録 / アプリ初期化
```
