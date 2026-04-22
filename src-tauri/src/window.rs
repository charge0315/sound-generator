use tauri::{AppHandle, Manager, PhysicalPosition};

#[derive(Debug, Default)]
pub struct WindowManager {
    pub is_visible: bool,
}

impl WindowManager {
    /// ウィンドウの表示・非表示を切り替える
    pub fn toggle(&mut self, app: &AppHandle, tray_pos: (i32, i32)) {
        let window = match app.get_webview_window("main") {
            Some(w) => w,
            None => return,
        };
        
        if self.is_visible {
            let _ = window.hide();
            self.is_visible = false;
        } else {
            let (x, y) = self.calculate_position(&window, tray_pos);
            let _ = window.set_position(PhysicalPosition::new(x, y));
            
            let _ = window.unminimize();
            let _ = window.show();
            let _ = window.set_focus();
            let _ = window.set_always_on_top(true);
            
            self.is_visible = true;

            // 重要: 表示された瞬間にフロントエンドへデータ更新を促す
            use tauri::Emitter;
            let _ = app.emit("window-visible", ());
        }
    }

    pub fn hide(&mut self, app: &AppHandle) {
        if let Some(window) = app.get_webview_window("main") {
            let _ = window.hide();
        }
        self.is_visible = false;
    }

    fn calculate_position(&self, window: &tauri::WebviewWindow, tray_pos: (i32, i32)) -> (i32, i32) {
        let monitor = window.current_monitor().ok().flatten().unwrap_or_else(|| {
            panic!("Fatal: No monitor detected.");
        });

        let scale_factor = monitor.scale_factor();
        let (tray_x, tray_y) = tray_pos;
        
        let width = (360.0 * scale_factor) as i32;
        let height = (500.0 * scale_factor) as i32;
        
        let m_size = monitor.size();
        let m_pos = monitor.position();

        let mut target_x = tray_x - (width / 2);
        let mut target_y = tray_y - height - (8.0 * scale_factor) as i32;

        if target_x < m_pos.x { target_x = m_pos.x + (8.0 * scale_factor) as i32; }
        if target_x + width > m_pos.x + m_size.width as i32 {
            target_x = m_pos.x + m_size.width as i32 - width - (8.0 * scale_factor) as i32;
        }

        if tray_y < m_pos.y + (m_size.height as i32 / 5) {
            target_y = tray_y + (24.0 * scale_factor) as i32;
        }

        (target_x, target_y)
    }
}
