# sound-generator

Rust & React (Tauri v2) を使用した、次世代のオーディオコントロール・アプリケーション。

## 🚀 プロジェクトの概要
Windowsのオーディオセッションを直感的に制御し、EarTrumpetのような利便性を持ちつつ、独自のカスタマイズと低遅延な操作性を実現するモダンなミキサーアプリです。

## 🛠️ 技術スタック
- **Frontend**: React 18 + Tailwind CSS (Vite)
- **Backend**: Rust 1.93 + Tauri (v2)
- **API**: `windows-rs` (Windows Core Audio APIs & Undocumented Audio Policy APIs)
- **Visuals**: Mica/Acrylic Effects (`window-vibrancy`)
- **Protocol**: Antigravity (Gemini-AI driven engineering)

## ⚙️ 開発環境のセットアップ
Windowsでビルドするために、`Visual Studio Build Tools 18 (2022)` が必要です。

### 開発サーバの起動
ビルド環境変数をロードして起動する必要があります：
```powershell
cmd /c "call ""C:\Program Files (x86)\Microsoft Visual Studio\18\BuildTools\VC\Auxiliary\Build\vcvarsall.bat"" x64 && cd /d %CD% && npm run tauri dev"
```

## ✨ 主な機能
- **アプリケーションごとの音量ミキシング**: 個別のアプリ音量をシームレスに操作。
- **Per-App Audio Routing**: Windowsの非公開APIをハックし、アプリごとに異なる出力デバイスを割り当て可能。
- **タスクトレイ常駐型フライアウトUI**: EarTrumpet風のクリック連動ポップアップ表示。
- **Fluid UI Animation**: 下からスッと浮かび上がるフェードインアニメーション。
- **インテリジェント配置**: タスクバーの位置（上下左右）やマルチモニター環境を自動検知し、常に最適な位置に吸着表示。
- **動的アイコン抽出**: 実行ファイルからアプリアイコンと製品名を抽出し、直感的なUIを提供。
- **Mica/Acrylic グラスモーフィズム**: Windows 11に最適化されたモダンなFluent Design。
- **自動テスト環境**: Rust & React 両面でのユニットテストによる品質担保。

## 🏗️ 開発ロードマップ
1. [x] **Phase 1**: Tauri + React の基本構成とタスクトレイ常駐の実装。（2026-02-24 完了）
2. [x] **Phase 2**: `windows-rs` を用いたオーディオセッションの列挙とボリューム制御ロジックの構築。（2026-02-25 完了）
3. [x] **Phase 3**: UIのブラッシュアップとフロントエンドへのイベント同期。（2026-02-25 完了）
4. [x] **Phase 4**: ネイティブUI体験の強化と非公開APIによるルーティング実装。（2026-04-21 完了）
5. [x] **Phase 5**: 視覚的ブラッシュアップとセキュリティ・メモリ安全性の強化。（2026-04-22 完了）
   - Fluent UI 3 (WinUI 3) 風スライダーとアニメーションの実装
   - `PROPVARIANT` 等のメモリリーク修正とバリデーションの追加
   - 自動テスト（Cargo / Vitest）の導入
6. [ ] **Phase 6**: 高度なオーディオ機能（永続化）とパッケージング


## 📝 開発ログ (Antigravity)
詳細な進捗や技術的な意思決定プロセスは [GEMINI.md](./gemini.md) を参照してください。
🚀🎸エンジニアの熱意とAIの知性が融合した、最先端のオーディオコントロール体験をお届けします。
