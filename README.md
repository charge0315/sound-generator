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
3. [/] **Phase 3**: UIのブラッシュアップと、スマホ入力セッションの特定・統合。

## 📝 次回以降のタスク (Phase 3)
- [ ] **Windows 11 スタイルの UI レインフォース・プロトタイプ作成**
  - TailwindCSS を活用した Mica/Acrylic 効果のシミュレーション、モダンなスライダー実装
- [ ] **リアルタイムイベントのUI反映**
  - アプリ側や Windows ミキサーでの音量・ミュート変更 (`IAudioSessionEvents`) を Tauri イベント経由で React へ即座に反映させる
- [ ] **プロセスのアイコン取得連携**
  - プロセスパスからアイコンを抽出し、UI 上に表示するロジック (`Win32_UI_Shell`) の追加
