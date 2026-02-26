# Sound Generator Architecture

このドキュメントは、アプリケーション全体のファイル構造と各モジュールの存在意義・設計方針をまとめています。

## 📁 ディレクトリ構成

### フロントエンド (React + Tailwind CSS)
主にUIレンダリングとユーザーの操作を受け付け、バックエンド(Rust)と通信します。

- **`src/`**
  - **`App.tsx`**
    - アプリケーションのUIのコアです。
    - **意図**: Tauriの `invoke` を使ってRustから音量セッション情報やデバイス情報を取得し、表示します。状態更新において「もたつき」を排除するため、通信を待たずにUIの見た目を先行して更新する（**楽観的UI更新 / Optimistic Update**）を実装しています。
  - **`main.tsx`**, **`index.css`**, **`styles.css`**
    - ReactマウントやTailwindのユーティリティクラスロード用。UIフレームワークの基盤。
  - **`vite.config.ts`**
    - Tauriと連携するためのVite開発サーバーやビルドの最適化設定。

### バックエンド (Rust + Tauri + windows-rs)
OSネイティブレベルの強力なAPI呼び出し（COMコンポーネント操作、ウィンドウ操作）を担当します。

- **`src-tauri/`**
  - **`tauri.conf.json`**
    - アプリケーション名、権限、ビルド設定、ウィンドウの初期透明度などを定義。
  - **`src/`**
    - **`main.rs`**
      - Rust・Tauriアプリの純粋なエントリポイント。
    - **`lib.rs`**
      - アプリケーションのセットアップとライフサイクル管理。
      - **意図**: 
        - カスタムのTauri状態（`AudioState`）を保持し、Mut(スレッドセーフ)で各コマンドへ展開しています。
        - **Tray Flyout**: 通知領域アイコン（タスクトレイ）の左クリックを検知し、EarTrumpetのような「ネイティブなポップアップ感」を出すために座標計算と `on_focus_changed` を自前で管理しています。
        - **Mica/Acrylic Effect**: Fluent Designを実現するため、`window-vibrancy`クレートを利用して背面に強力なすりガラス効果（アクリル効果）を適用しています。
    - **`audio/`** (Windows Audio API コアロジック)
      - **`mod.rs`**
        - WASAPI (Windows Audio Session API) を叩き、現在開いているアプリの一覧や音量を列挙・操作します。
        - **意図**: COMはスレッド固有のコンテキスト（STA/MTA）を要求しますが、Tauriのコマンドは並行に走る可能性があるため、安全策としてバックグラウンドスレッド内で MTA（マルチスレッドアパートメント）としてCOMを都度初期化・保護しています。
      - **`events.rs`**
        - 音量やミュートが外部（Windows標準のミキサー等）から変更された際、そのイベントをPush型でリアルタイムにReactへ送信(`emit`)します。
      - **`icon.rs`**
        - 各セッションのPIDから実行ファイル（.exe）のフルパスを引き当て、Win32 APIの `SHGetFileInfoW` を用いてアプリアイコンを直接メモリDC上に描画・抽出します。
        - **意図**: ディスク保存によるI/Oボトルネックを防ぐため、アイコンのビットマップをIn-MemoryでPNG圧縮・Base64化し、そのままReact側の `<img src="data:image/png;base64,...">` に流し込んでいます。
      - **`policy.rs`**
        - 特定のアプリの音を別のデバイス（スピーカー、ヘッドホン）にルーティングする機能を担います。
        - **意図**: Windows SDKの「非公開API（Undocumented API）」である `IAudioPolicyConfigFactory` に強引にアクセスするため、COMのVTable（関数ポインタ配列）のメモリレイアウトを手動で定義し直すというアグレッシブなハックを行っています。

## 💡 基本設計思想とUX (User Experience)
1. **Fluid UI (ヌルサク感)**: バックエンドの遅延をUIに出さない（React側の即時state更新など）。
2. **Native Feel (ネイティブと遜色ない挙動)**: FlyoutウィンドウやMica/Acrylic透過エフェクト、本物のアプリアイコン抽出により、単なるElectron系のWebアプリではなく、「Windows 11のOSの一部」のような手触りを目指しています。
3. **Push-based Sync (低遅延の同期)**: ポーリング（定期的な状態確認）ではなく、COMコールバックとTauri Eventを組み合わせることで、CPU負荷を最小限にしつつ他アプリでの音量変更をニアリアルタイムで表示します。
