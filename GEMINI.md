# GEMINI.md

## 🤖 AIアシスタントへの指示書 (Antigravity Protocol)

このプロジェクトでは、Gemini / Antigravityをフル活用して開発を進めます。アシスタントは以下のコンテキストを常に考慮してください。

### 👤 開発者コンテキスト
- **名前**: みつひでさん
- **職業**: プログラマ（React/Node.jsに精通）
- **所有PC**: AtomMan G7 Pt (Ryzen 9 7945HX / RX 7600M XT)
- **開発スタイル**: 前向きで正直な対話を好む。技術的な課題に対しては、ユーモアを交えつつも核心を突いた解決策を求めている。

### 🎯 アシスタントへの依頼事項
1. **日本語でのコミュニケーション**:
   対話は必ず日本語で行ってください。コード内のコメントも日本語で記述し、単なる「処理の内容」ではなく「処理の意図・意味」を説明するように心がけてください。
2. **Win32/COMのボイラープレート生成**:
   `windows-rs` を使用したオーディオ操作は、ポインタ管理やCOMの安全な解放が難解です。Rustの `Drop` トレイトなどを活用した、安全でクリーンなラップコードを生成してください。
3. **パフォーマンスと低遅延**:
   オーディオミキサーという性質上、UIと実音量の同期の低遅延化が重要です。効率的なイベントループや通知の仕組みを提案してください。
4. **UI/UXのアドバイス**:
   Tailwind CSSを用いた、Windows 11らしい最新のデザイントレンド（Mica/Acrylic効果など）の実装方法を提示してください。

### ⚠️ ユーモア・ポリシー
エラーが続いたり、コンパイルが通らずにみつひでさんが疲れているような時は、エンジニアにしか分からない気の利いたジョークで励ましてください。🚀🎸

---

## 📈 プロジェクト進捗ログ (Antigravity)

### 🗓️ 2026-02-24: プロジェクト初期セットアップ完了
- **Tauri v2 + React + Tailwind CSS** の基盤を構築。
- **Git** リモート設定完了 (GitHub: `charge0315/sound-generator`)。
- **タスクトレイ** の基本機能を Rust で実装（Show/Exit、左クリック連動）。
- **環境構築**: Rust 1.93.1 と VS Build Tools 18 をセットアップ。
- **ビルドパス**: `C:\Program Files (x86)\Microsoft Visual Studio\18\BuildTools` を使用中。ビルド時は `vcvarsall.bat` のロードが必須。
### 🗓️ 2026-02-25: Phase 2 Kickoff & Audio Control Implementation (Completed)
- **モデル同期**: Gemini 3.1 Pro への移行を検討中（Antigravity Protocol のコアエンジン更新）。
- **Phase 2 完了**: `windows-rs` を用い、MTAスレッド上でのCOM初期化と `IAudioSessionManager2` / `ISimpleAudioVolume` によるリアルタイム音量制御機能のTauriコマンド統合を完了。
- **課題クリア**: winit の STA メインスレッドとの競合を非同期スレッドへ分離することで解決。
### 🗓️ 2026-02-25: Phase 3 Completed (UI & Real-time Sync)
- **Rust to React Sync**: `IAudioSessionEvents` のCOMコールバックを Tauri の `AppHandle` に連携し、イベントリスナーによる超低遅延なUI音量同期を実現。
- **Fluent UI**: `window-vibrancy` による Mica/Acrylic 効果と、Tailwind CSSでのダークテーマ・グラスモーフィズムUIを実装。ReactのStateを最適化し、ポーリングレスに。
### 🗓️ 2026-02-25: Phase 4 Kickoff (App Icons & Tray Flyout)
- **次フェーズ要件**: アプリアイコンの動的抽出と、タスクトレイククリック時のフライアウトウィンドウ（EarTrumpet風の動作）の構築に着手。
- **Per-App Audio Routing**: EarTrumpetのソースコードを解析し、Windowsの非公開API `IAudioPolicyConfigFactory` のRust (`windows-rs`) への移植を開始。現在はCOMのvtable直接バインディングにおけるコンパイルエラーを修正中。明日は引き続きこのインターフェースの実装とコマンド統合を行う予定。