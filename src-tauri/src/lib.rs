use std::sync::Mutex;
use tauri::menu::{Menu, MenuItem};
use tauri::tray::{TrayIconBuilder, TrayIconEvent};
use tauri::{Manager, PhysicalPosition, WindowEvent};
use window_vibrancy::apply_acrylic;

mod audio;
use audio::{init_com, AudioManager, AudioSessionInfo};

// AudioManagerはスレッドセーフではないCOMインターフェースを持つため、
// TauriのStateとして持たせる場合はMutex等で保護する必要がある。
pub struct AudioState(Mutex<Option<AudioManager>>);

impl AudioState {
    fn with_manager<F, R>(&self, app_handle: &tauri::AppHandle, f: F) -> Result<R, String>
    where
        F: FnOnce(&AudioManager) -> Result<R, String>,
    {
        // コマンドを実行するスレッド（Tauriのバックグラウンドスレッド）でCOMをMTAとして初期化
        let _ = init_com();
        let mut guard = self.0.lock().map_err(|_| "Deadlock".to_string())?;
        if guard.is_none() {
            let mut manager = AudioManager::new().map_err(|e| e.to_string())?;
            manager.set_app_handle(app_handle.clone());
            *guard = Some(manager);
        }
        if let Some(manager) = guard.as_ref() {
            f(manager)
        } else {
            Err("Failed to initialize AudioManager".to_string())
        }
    }
}

#[tauri::command]
fn get_audio_sessions(
    app: tauri::AppHandle,
    state: tauri::State<'_, AudioState>,
) -> Result<Vec<AudioSessionInfo>, String> {
    state.with_manager(&app, |manager| {
        manager
            .get_sessions()
            .map_err(|e| format!("Failed to get sessions: {}", e))
    })
}

#[tauri::command]
fn set_session_volume(
    app: tauri::AppHandle,
    process_id: u32,
    volume: f32,
    state: tauri::State<'_, AudioState>,
) -> Result<(), String> {
    state.with_manager(&app, |manager| {
        manager
            .set_session_volume(process_id, volume)
            .map_err(|e| format!("Failed to set volume: {}", e))
    })
}

#[tauri::command]
fn set_session_mute(
    app: tauri::AppHandle,
    process_id: u32,
    mute: bool,
    state: tauri::State<'_, AudioState>,
) -> Result<(), String> {
    state.with_manager(&app, |manager| {
        manager
            .set_session_mute(process_id, mute)
            .map_err(|e| format!("Failed to set mute: {}", e))
    })
}

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .on_window_event(|window, event| {
            // ウィンドウからフォーカスが外れたとき（他の場所をクリックしたとき）に自動で隠す
            if let WindowEvent::Focused(false) = event {
                let _ = window.hide();
            }
        })
        .setup(|app| {
            let quit_i = MenuItem::with_id(app, "quit", "Exit", true, None::<&str>)?;
            let show_i = MenuItem::with_id(app, "show", "Show", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show_i, &quit_i])?;

            let _tray = TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "quit" => {
                        app.exit(0);
                    }
                    "show" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: tauri::tray::MouseButton::Left,
                        button_state: tauri::tray::MouseButtonState::Up,
                        rect,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("main") {
                            // フライアウト動作のため、クリック位置の近くにウィンドウを移動する
                            let window_size = window.outer_size().unwrap_or_default();

                            // タスクトレイアイコンの中心からウィンドウ幅の半分だけ左にずらし、上にウィンドウ高さ分だけ移動する
                            let (icon_x, icon_y) = match rect.position {
                                tauri::Position::Physical(p) => (p.x as i32, p.y as i32),
                                tauri::Position::Logical(p) => (p.x as i32, p.y as i32),
                            };

                            // 画面のスケールファクタを取得（マルチモニター対応のためウィンドウ等から）
                            let scale_factor = window.scale_factor().unwrap_or(1.0);
                            let width = (window_size.width as f64 / scale_factor) as i32;
                            let height = (window_size.height as f64 / scale_factor) as i32;

                            // x 座標: アイコンの中心から左へずらす。右端切れを防ぐ
                            let mut x = icon_x - (width / 2);
                            // y 座標: アイコンの上（タスクバーの上）に配置する
                            let mut y = icon_y - height - 40; // 余裕を持たせる

                            // 位置の補正 (簡易的)
                            if x < 0 {
                                x = 0;
                            }
                            if y < 0 {
                                y = 0;
                            }

                            let pos = PhysicalPosition::new(
                                (x as f64 * scale_factor) as i32,
                                (y as f64 * scale_factor) as i32,
                            );
                            let _ = window.set_position(tauri::Position::Physical(pos));

                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                })
                .build(app)?;

            // メインウィンドウにエフェクトを適用する
            if let Some(window) = app.get_webview_window("main") {
                #[cfg(target_os = "windows")]
                {
                    // alwaysOnTop や skipTaskbar なウィンドウの場合、Micaは無効化されるケースが多いため、
                    // 強制的に Acrylic 効果 (より深いすりガラス) を適用し、背景透過を確実にする
                    let _ = apply_acrylic(&window, Some((18, 18, 18, 160)));
                }
            }

            Ok(())
        })
        .manage(AudioState(Mutex::new(None)))
        .invoke_handler(tauri::generate_handler![
            greet,
            get_audio_sessions,
            set_session_volume,
            set_session_mute
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
