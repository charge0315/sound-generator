use std::sync::Mutex;
use tauri::menu::{Menu, MenuItem};
use tauri::tray::{TrayIconBuilder, TrayIconEvent};
use tauri::{Manager, PhysicalPosition, WindowEvent};
use window_vibrancy::apply_acrylic;

mod audio;
use audio::{init_com, AudioManager, AudioSessionInfo};

// AudioManagerはスレッドセーフではないCOMインターフェースを持つため、
// AudioManagerは、WindowsのオーディオAPI (WASAPI / EndpointVolume) のCOMオブジェクトの参照を保持する。
// COMの世界ではスレッドの「アパートメント(STA/MTA)」の概念があり、RustのようにSend/Syncを
// 単純にスレッド間で渡すことが許されないことが多いが、MTAとして初期化した上でMutexでラップすることで
// Tauriのコマンド（バックグラウンドスレッドプール）から安全に呼び出せるようにハックしている。
pub struct AudioState(Mutex<Option<AudioManager>>);

impl AudioState {
    fn with_manager<F, R>(&self, app_handle: &tauri::AppHandle, f: F) -> Result<R, String>
    where
        F: FnOnce(&AudioManager) -> Result<R, String>,
    {
        // どのスレッドから呼ばれても安全にCOMを操作できるよう、毎回 MTA (Multi-Threaded Apartment) としてCOMを初期化する。
        // S_FALSE（既に初期化済み）が返ることもあるが、気にしない。
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
fn set_audio_routing(
    app: tauri::AppHandle,
    process_id: u32,
    device_id: String,
    state: tauri::State<'_, AudioState>,
) -> Result<(), String> {
    state.with_manager(&app, |manager| {
        manager
            .set_audio_routing(process_id, &device_id)
            .map_err(|e| format!("Failed to set audio routing: {}", e))
    })
}

#[tauri::command]
fn get_audio_devices(
    app: tauri::AppHandle,
    state: tauri::State<'_, AudioState>,
) -> Result<Vec<audio::AudioDeviceInfo>, String> {
    state.with_manager(&app, |manager| {
        manager
            .get_audio_devices()
            .map_err(|e| format!("Failed to get audio devices: {}", e))
    })
}

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let app = tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .on_window_event(|window, event| {
            match event {
                // Focus out logic temporarily disabled for dragging
                // WindowEvent::Focused(false) => {
                //     let _ = window.hide();
                // }
                WindowEvent::CloseRequested { api, .. } => {
                    // Prevent the window from completely closing (which might cause the app to exit)
                    // Instead, just hide it.
                    let _ = window.hide();
                    api.prevent_close();
                }
                _ => {}
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
                        use tauri::Emitter;
                        let _ = app.emit("tray_menu_show", ());
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: tauri::tray::MouseButton::Left,
                        button_state: tauri::tray::MouseButtonState::Up,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();

                        let mut point = windows::Win32::Foundation::POINT { x: 0, y: 0 };
                        let (icon_x, icon_y) = unsafe {
                            if windows::Win32::UI::WindowsAndMessaging::GetCursorPos(&mut point)
                                .is_ok()
                            {
                                (point.x, point.y)
                            } else {
                                (0, 0)
                            }
                        };

                        #[derive(serde::Serialize, Clone)]
                        struct TrayPos {
                            x: i32,
                            y: i32,
                        }

                        use tauri::Emitter;
                        let _ = app.emit(
                            "tray_click_left",
                            TrayPos {
                                x: icon_x,
                                y: icon_y,
                            },
                        );
                    }
                })
                .build(app)?;

            // フロントエンド側の背景透過と組み合わせて、Windowsネイティブの「すりガラス」効果を適用する。
            if let Some(window) = app.get_webview_window("main") {
                #[cfg(target_os = "windows")]
                {
                    // alwaysOnTop や skipTaskbar なウィンドウの場合、OSの制約により Mica が効かない（フォールバックされる）ケースが多いため、
                    // ここでは強制的に Acrylic 効果 (より深く透過するすりガラス) を適用している。
                    // RGB/Альファ値の調整でダークテーマに馴染ませている。
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
            set_session_mute,
            set_audio_routing,
            get_audio_devices
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application");

    app.run(|_app_handle, event| {
        if let tauri::RunEvent::ExitRequested { api, .. } = event {
            // Keep the app running in the background for the tray icon
            api.prevent_exit();
        }
    });
}
