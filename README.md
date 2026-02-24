# sound-generator

Rust & React (Tauri) を使用した、次世代のオーディオコントロール・アプリケーション。

## 🚀 プロジェクトの概要
Windowsのオーディオセッションを直感的に制御し、EarTrumpetのような利便性を持ちつつ、独自のカスタマイズを可能にするミキサーアプリです。

## 🛠️ 技術スタック
- **Frontend**: React + Tailwind CSS (Vite)
- **Backend**: Rust + Tauri
- **API**: `windows-rs` (Windows Core Audio APIs)
- **Protocol**: Antigravity (AI-driven development)

## ✨ 主な機能（予定）
- **アプリケーションごとの音量ミキシング**: 個別のアプリ音量をシームレスに操作。
- **タスクトレイ常駐型モダンUI**: Windows 11に馴染むアクリル効果と角丸デザイン。
- **スマホ音源ミックス機能**: Bluetooth Audio Receiver経由でスマホの音をPCに取り込み、軽量に音量管理。

## 🏗️ 開発ロードマップ
1. **Phase 1**: Tauri + React の基本構成とタスクトレイ常駐の実装。
2. **Phase 2**: `windows-rs` を用いたオーディオセッションの列挙とボリューム制御ロジックの構築。
3. **Phase 3**: UIのブラッシュアップと、スマホ入力セッションの特定・統合。
