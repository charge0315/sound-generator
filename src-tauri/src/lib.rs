use std::sync::Mutex;
use tauri::{AppHandle, Manager, State};
use tauri::tray::{TrayIconBuilder, TrayIconEvent, MouseButton, MouseButtonState};

mod audio;
mod window;

use audio::{AudioManager, AudioSessionInfo};
use window::WindowManager;

pub struct AudioState(Mutex<Option<AudioManager>>);

impl AudioState {
    fn with_manager<F, R>(&self, app_handle: &AppHandle, f: F) -> Result<R, String>
    where
        F: FnOnce(&mut AudioManager) -> Result<R, String>,
    {
        let mut guard = self.0.lock().map_err(|_| "Lock failed")?;
        if guard.is_none() {
            let _ = audio::com::init_mta();
            let mut manager = AudioManager::new().map_err(|e| e.to_string())?;
            manager.set_app_handle(app_handle.clone());
            *guard = Some(manager);
        }
        f(guard.as_mut().unwrap())
    }
}

#[tauri::command]
fn get_audio_sessions(app: AppHandle, state: State<'_, AudioState>) -> Result<Vec<AudioSessionInfo>, String> {
    state.with_manager(&app, |m| m.get_sessions().map_err(|e| e.to_string()))
}

#[tauri::command]
fn set_session_volume(app: AppHandle, state: State<'_, AudioState>, pid: u32, volume: f32) -> Result<(), String> {
    state.with_manager(&app, |m| m.set_session_volume(pid, volume).map_err(|e| e.to_string()))
}

#[tauri::command]
fn set_session_mute(app: AppHandle, state: State<'_, AudioState>, pid: u32, mute: bool) -> Result<(), String> {
    state.with_manager(&app, |m| m.set_session_mute(pid, mute).map_err(|e| e.to_string()))
}

#[tauri::command]
fn set_audio_routing(app: AppHandle, state: State<'_, AudioState>, pid: u32, device_id: String) -> Result<(), String> {
    state.with_manager(&app, |m| m.set_audio_routing(pid, &device_id).map_err(|e| e.to_string()))
}

#[tauri::command]
fn get_audio_devices(app: AppHandle, state: State<'_, AudioState>) -> Result<Vec<audio::AudioDeviceInfo>, String> {
    state.with_manager(&app, |m| m.get_audio_devices().map_err(|e| e.to_string()))
}

#[tauri::command]
fn is_auto_launch_enabled() -> Result<bool, String> {
    use winreg::enums::*;
    use winreg::RegKey;
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let key = hkcu.open_subkey("Software\\Microsoft\\Windows\\CurrentVersion\\Run").map_err(|e: std::io::Error| e.to_string())?;
    let val: String = key.get_value("AntigravityPulse").unwrap_or_default();
    Ok(!val.is_empty())
}

#[tauri::command]
fn toggle_auto_launch(enable: bool) -> Result<(), String> {
    use winreg::enums::*;
    use winreg::RegKey;
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let (key, _) = hkcu.create_subkey("Software\\Microsoft\\Windows\\CurrentVersion\\Run").map_err(|e: std::io::Error| e.to_string())?;

    if enable {
        let exe_path = std::env::current_exe().map_err(|e: std::io::Error| e.to_string())?;
        let exe_str = exe_path.to_str().ok_or("Invalid EXE path")?;
        key.set_value("AntigravityPulse", &exe_str).map_err(|e: std::io::Error| e.to_string())?;
    } else {
        let _ = key.delete_value("AntigravityPulse");
    }
    Ok(())
}

#[tauri::command]
fn set_tactical_mode(window: tauri::WebviewWindow, enabled: bool) -> Result<(), String> {
    window.set_always_on_top(enabled).map_err(|e| e.to_string())?;
    let _opacity = if enabled { 0.7 } else { 1.0 };
    window.set_shadow(!enabled).map_err(|e| e.to_string())?;
    // Note: window-vibrancy is used for overall effect, but we can nudge opacity
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_global_shortcut::Builder::new()
            .with_handler(move |app, _shortcut, event| {
                use tauri_plugin_global_shortcut::ShortcutState;
                if event.state() == ShortcutState::Pressed {
                    let wm_state = app.state::<Mutex<WindowManager>>();
                    let mut wm = wm_state.lock().unwrap();
                    let mut point = windows::Win32::Foundation::POINT { x: 0, y: 0 };
                    let pos = unsafe {
                        if windows::Win32::UI::WindowsAndMessaging::GetCursorPos(&mut point).is_ok() {
                            (point.x, point.y)
                        } else {
                            (0, 0)
                        }
                    };
                    wm.toggle(app, pos);
                }
            })
            .build()
        )
        .manage(AudioState(Mutex::new(None)))
        .manage(Mutex::new(WindowManager::default()))
        .setup(|app| {
            use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut};
            use std::str::FromStr;
            let _ = app.global_shortcut().register(Shortcut::from_str("Super+Alt+A").unwrap());

            let handle = app.handle().clone();
            
            TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click { position, button, button_state, .. } = event {
                        if button == MouseButton::Left && button_state == MouseButtonState::Up {
                            let app = tray.app_handle();
                            let wm_state = app.state::<Mutex<WindowManager>>();
                            let mut wm = wm_state.lock().unwrap();
                            wm.toggle(app, (position.x as i32, position.y as i32));
                        }
                    }
                })
                .build(app)?;

            if let Some(window) = app.get_webview_window("main") {
                let wm_state = app.state::<Mutex<WindowManager>>();
                let wm = wm_state.lock().unwrap();
                wm.apply_visual_effects(&window);

                // テスト用：環境変数があれば即座に中央に表示
                if std::env::var("PULSE_TEST_MODE").is_ok() {
                    let _ = window.set_position(tauri::PhysicalPosition::new(200, 200));
                    let _ = window.show();
                    let _ = window.set_focus();
                    let _ = window.set_always_on_top(true);
                }
            }

            let handle_task = handle.clone();
            std::thread::spawn(move || {
                let mut session_refresh_counter = 0;
                loop {
                    std::thread::sleep(std::time::Duration::from_millis(16));
                    session_refresh_counter += 1;
                    
                    let state = handle_task.state::<AudioState>();
                    let _ = state.with_manager(&handle_task, |m| {
                        use tauri::Emitter;
                        if let Ok(peaks) = m.get_peak_levels() {
                            let _ = handle_task.emit("audio-pulse", peaks);
                        }
                        
                        if session_refresh_counter >= 120 {
                            session_refresh_counter = 0;
                            if let Ok(sessions) = m.get_sessions() {
                                let _ = handle_task.emit("refresh-sessions", sessions);
                            }
                        }
                        Ok(())
                    });
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_audio_sessions,
            set_session_volume,
            set_session_mute,
            set_audio_routing,
            get_audio_devices,
            is_auto_launch_enabled,
            toggle_auto_launch,
            set_tactical_mode
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
