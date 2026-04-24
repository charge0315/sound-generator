use tauri::{AppHandle, Manager, PhysicalPosition, WebviewWindow};
use window_vibrancy::{apply_acrylic, apply_mica};

#[derive(Debug, Default)]
pub struct WindowManager {}

impl WindowManager {
    pub fn apply_visual_effects(&self, window: &WebviewWindow) {
        if let Err(_) = apply_mica(window, None) {
            let _ = apply_acrylic(window, Some((20, 20, 20, 10)));
        }
    }

    pub fn toggle(&mut self, app: &AppHandle, tray_pos: (i32, i32)) {
        let window = match app.get_webview_window("main") {
            Some(w) => w,
            None => return,
        };

        if window.is_visible().unwrap_or(false) {
            let _ = window.hide();
        } else {
            let (x, y) = self.calculate_position(&window, tray_pos);
            let _ = window.set_position(PhysicalPosition::new(x, y));
            let _ = window.show();
            let _ = window.unminimize();
            let _ = window.set_focus();
            let _ = window.set_always_on_top(true);
            
            use tauri::Emitter;
            let _ = app.emit("window-visible", ());
        }
    }

    fn calculate_position(&self, window: &WebviewWindow, (tx, ty): (i32, i32)) -> (i32, i32) {
        let size = window.outer_size().unwrap_or_default();
        let w = size.width as i32;
        let h = size.height as i32;

        // モニター情報を取得して境界チェック
        let monitor = window.current_monitor().ok().flatten().unwrap_or_else(|| {
            window.primary_monitor().ok().flatten().unwrap()
        });
        let m_size = monitor.size();
        let m_pos = monitor.position();

        let mut target_x = tx - (w / 2);
        let mut target_y = ty - h - 10;

        // 画面端の補正
        if target_x < m_pos.x { target_x = m_pos.x + 10; }
        if target_x + w > m_pos.x + m_size.width as i32 {
            target_x = m_pos.x + m_size.width as i32 - w - 10;
        }

        // 上部召喚の場合（タスクバーが上の場合など）
        if target_y < m_pos.y {
            target_y = ty + 10;
        }

        (target_x, target_y)
    }
}
