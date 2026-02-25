use std::sync::Mutex;
use tauri::menu::{Menu, MenuItem};
use tauri::tray::{TrayIconBuilder, TrayIconEvent};
use tauri::Manager;

mod audio;
use audio::{init_com, AudioManager, AudioSessionInfo};

// AudioManagerはスレッドセーフではないCOMインターフェースを持つため、
// TauriのStateとして持たせる場合はMutex等で保護する必要がある。
pub struct AudioState(Mutex<Option<AudioManager>>);

impl AudioState {
    fn with_manager<F, R>(&self, f: F) -> Result<R, String>
    where
        F: FnOnce(&AudioManager) -> Result<R, String>,
    {
        // コマンドを実行するスレッド（Tauriのバックグラウンドスレッド）でCOMをMTAとして初期化
        let _ = init_com();
        let mut guard = self.0.lock().map_err(|_| "Deadlock".to_string())?;
        if guard.is_none() {
            *guard = AudioManager::new().ok();
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
    state: tauri::State<'_, AudioState>,
) -> Result<Vec<AudioSessionInfo>, String> {
    state.with_manager(|manager| {
        manager
            .get_sessions()
            .map_err(|e| format!("Failed to get sessions: {}", e))
    })
}

#[tauri::command]
fn set_session_volume(
    process_id: u32,
    volume: f32,
    state: tauri::State<'_, AudioState>,
) -> Result<(), String> {
    state.with_manager(|manager| {
        manager
            .set_session_volume(process_id, volume)
            .map_err(|e| format!("Failed to set volume: {}", e))
    })
}

#[tauri::command]
fn set_session_mute(
    process_id: u32,
    mute: bool,
    state: tauri::State<'_, AudioState>,
) -> Result<(), String> {
    state.with_manager(|manager| {
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
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                })
                .build(app)?;

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
