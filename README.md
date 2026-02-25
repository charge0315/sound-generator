# sound-generator

Rust & React (Tauri) を使用した、次世代のオーディオコントロール・アプリケーション。

## 🚀 プロジェクトの概要
Windowsのオーディオセッションを直感的に制御し、EarTrumpetのような利便性を持ちつつ、独自のカスタマイズを可能にするミキサーアプリです。

## 🛠️ 技術スタック
- **Frontend**: React + Tailwind CSS (Vite)
- **Backend**: Rust + Tauri (v2)
- **API**: `windows-rs` (Windows Core Audio APIs)
- **Protocol**: Antigravity (AI-driven development)

## ⚙️ 開発環境のセットアップ
Windowsでビルドするために、`Visual Studio Build Tools 2022` (C++によるデスクトップ開発) が必要です。

### 開発サーバの起動
ビルド環境変数をロードして起動する必要があります：
```powershell
cmd /c "call ""C:\Program Files (x86)\Microsoft Visual Studio\18\BuildTools\VC\Auxiliary\Build\vcvarsall.bat"" x64 && cd /d %CD% && npm run tauri dev"
```

## ✨ 主な機能（予定）
- **アプリケーションごとの音量ミキシング**: 個別のアプリ音量をシームレスに操作。
- **タスクトレイ常駐型モダンUI**: 「表示」「終了」メニュー、左クリックでのウィンドウ表示を実装済み。
- **スマホ音源ミックス機能**: Bluetooth Audio Receiver経由でスマホの音をPCに取り込み、管理。

## 🏗️ 開発ロードマップ
1. [x] **Phase 1**: Tauri + React の基本構成とタスクトレイ常駐の実装。（2026-02-24 完了）
2. [x] **Phase 2**: `windows-rs` を用いたオーディオセッションの列挙とボリューム制御ロジックの構築。（2026-02-25 完了）
3. [x] **Phase 3**: UIのブラッシュアップとフロントエンドへのイベント同期。（2026-02-25 完了）
   - Tailwindと `window-vibrancy` によるMica効果のフル実装
   - `IAudioSessionEvents` から Tauriイベントを用いた超低遅延フロントエンド同期
4. [/] **Phase 4**: ネイティブUI体験の強化とプロセスアイコン抽出

## 📝 次回以降のタスク (Phase 4: ネイティブ化とアイコン抽出)
- [ ] **Windows EXEsからのアプリアイコン自動抽出 (`SHGetFileInfo` / `ExtractIcon`)**
  - セッションのPIDから実行可能ファイルのパスを割り出し、高解像度のアイコンをBase64またはローカル画像URIとして抽出し、フロントエンドへ送る。現在の一時的な文字アイコンを本来のアプリアイコンに置き換える。
- [ ] **アプリごとの音声ルーティング (Per-App Audio Routing)**
  - EarTrumpetのC#実装を解析し、非公開COMインターフェース `IAudioPolicyConfigFactory` のRustバインディングを実装中。現在、`cargo check` のCOM vtable周りのコンパイルエラーを解決中。
- [ ] **タスクトレイ(システムトレイ)からのポップアップ（フライアウト）化**
  - EarTrumpetのように、タスクトレイアイコンをクリックした際にカーソル位置（右下）付近にウィンドウが表示され、フォーカスが外れたら自動で隠れる「フライアウト」動作を実装し、ネイティブなWindowsミキサーの代替品としてのUXを完成させる。
