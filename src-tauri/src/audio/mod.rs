pub mod events;
pub mod icon;
pub mod policy;

use std::ptr;
use windows::{
    core::{Interface, Result},
    Win32::{
        Foundation::MAX_PATH,
        Media::Audio::{
            eRender, IAudioSessionControl2,
            IAudioSessionManager2, IMMDeviceEnumerator, ISimpleAudioVolume,
            MMDeviceEnumerator,
            Endpoints::IAudioMeterInformation,
            IMMNotificationClient, IMMNotificationClient_Impl, DEVICE_STATE,
            EDataFlow, ERole, DEVICE_STATEMASK_ALL,
        },
        System::Com::{CoCreateInstance, CLSCTX_ALL, CoTaskMemFree},
        System::WinRT::{RoInitialize, RO_INIT_MULTITHREADED},
    },
};

#[windows_core::implement(IMMNotificationClient)]
struct DeviceNotificationClient {
    app_handle: tauri::AppHandle,
}

impl IMMNotificationClient_Impl for DeviceNotificationClient_Impl {
    fn OnDeviceStateChanged(&self, _pwstrdeviceid: &windows::core::PCWSTR, _dwnewstate: DEVICE_STATE) -> windows::core::Result<()> {
        use tauri::Emitter;
        let _ = self.app_handle.emit("refresh-trigger", ());
        Ok(())
    }
    fn OnDeviceAdded(&self, _pwstrdeviceid: &windows::core::PCWSTR) -> windows::core::Result<()> {
        use tauri::Emitter;
        let _ = self.app_handle.emit("refresh-trigger", ());
        Ok(())
    }
    fn OnDeviceRemoved(&self, _pwstrdeviceid: &windows::core::PCWSTR) -> windows::core::Result<()> {
        use tauri::Emitter;
        let _ = self.app_handle.emit("refresh-trigger", ());
        Ok(())
    }
    fn OnDefaultDeviceChanged(&self, _flow: EDataFlow, _role: ERole, _pwstrdefaultdeviceid: &windows::core::PCWSTR) -> windows::core::Result<()> {
        use tauri::Emitter;
        let _ = self.app_handle.emit("refresh-trigger", ());
        Ok(())
    }
    fn OnPropertyValueChanged(&self, _pwstrdeviceid: &windows::core::PCWSTR, _key: &windows::Win32::UI::Shell::PropertiesSystem::PROPERTYKEY) -> windows::core::Result<()> {
        Ok(())
    }
}

pub fn init_com() -> Result<()> {
    unsafe {
        let _ = RoInitialize(RO_INIT_MULTITHREADED);
    }
    Ok(())
}

#[derive(Debug, serde::Serialize)]
pub struct AudioSessionInfo {
    pub process_id: u32,
    pub process_name: String,
    pub volume: f32,
    pub is_muted: bool,
    pub peak_level: f32,
    pub icon_base64: Option<String>,
    pub device_id: String,
}

#[derive(Debug, serde::Serialize)]
pub struct AudioDeviceInfo {
    pub id: String,
    pub name: String,
}

pub struct AudioManager {
    device_enumerator: IMMDeviceEnumerator,
    app_handle: Option<tauri::AppHandle>,
    registered_pids: std::collections::HashSet<u32>,
    notification_client: Option<IMMNotificationClient>,
}

unsafe impl Send for AudioManager {}
unsafe impl Sync for AudioManager {}

impl AudioManager {
    pub fn new() -> Result<Self> {
        unsafe {
            let device_enumerator: IMMDeviceEnumerator =
                CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL)?;
            Ok(Self {
                device_enumerator,
                app_handle: None,
                registered_pids: std::collections::HashSet::new(),
                notification_client: None,
            })
        }
    }

    pub fn set_app_handle(&mut self, handle: tauri::AppHandle) {
        self.app_handle = Some(handle.clone());
        unsafe {
            let client: IMMNotificationClient = DeviceNotificationClient { app_handle: handle }.into();
            if self.device_enumerator.RegisterEndpointNotificationCallback(&client).is_ok() {
                self.notification_client = Some(client);
            }
        }
    }

    fn get_all_render_devices(
        &self,
    ) -> Result<windows::Win32::Media::Audio::IMMDeviceCollection> {
        unsafe {
            self.device_enumerator
                .EnumAudioEndpoints(eRender, DEVICE_STATE(DEVICE_STATEMASK_ALL))
        }
    }

    pub fn get_sessions(&mut self) -> Result<Vec<AudioSessionInfo>> {
        let mut sessions_info = Vec::new();
        let mut seen_sessions = std::collections::HashSet::new();

        unsafe {
            let collection = match self.get_all_render_devices() {
                Ok(c) => c,
                Err(_) => return Ok(Vec::new()),
            };
            let device_count = collection.GetCount().unwrap_or(0);

            for d in 0..device_count {
                let device = if let Ok(it) = collection.Item(d) { it } else { continue; };
                let id_pwstr = if let Ok(id) = device.GetId() { id } else { continue; };
                let device_id = id_pwstr.to_string().unwrap_or_default();
                CoTaskMemFree(Some(id_pwstr.as_ptr() as _));

                let session_manager: IAudioSessionManager2 = if let Ok(sm) = device.Activate(CLSCTX_ALL, None) { sm } else { continue; };
                let enumerator = if let Ok(en) = session_manager.GetSessionEnumerator() { en } else { continue; };
                let count = enumerator.GetCount().unwrap_or(0);

                for i in 0..count {
                    let session = if let Ok(s) = enumerator.GetSession(i) { s } else { continue; };
                    let control2: IAudioSessionControl2 = if let Ok(c) = session.cast() { c } else { continue; };
                    
                    let pid = control2.GetProcessId().unwrap_or(0);
                    let state = control2.GetState().unwrap_or(windows::Win32::Media::Audio::AudioSessionStateInactive);
                    
                    if state == windows::Win32::Media::Audio::AudioSessionStateExpired { continue; }

                    // 重複排除のためのキー (PID とセッション識別子)
                    let session_id_pwstr = if let Ok(id) = control2.GetSessionIdentifier() { id } else { continue; };
                    let session_id_str = session_id_pwstr.to_string().unwrap_or_default();
                    let session_key = format!("{}-{}", pid, session_id_str);
                    if seen_sessions.contains(&session_key) {
                        CoTaskMemFree(Some(session_id_pwstr.as_ptr() as _));
                        continue;
                    }
                    seen_sessions.insert(session_key);

                    // 音量とピークの取得
                    let simple_volume: Result<ISimpleAudioVolume> = session.cast();
                    let (mut vol, mut mute) = (1.0, windows::Win32::Foundation::BOOL::default());
                    if let Ok(sv) = &simple_volume {
                        vol = sv.GetMasterVolume().unwrap_or(1.0);
                        mute = sv.GetMute().unwrap_or_default();
                    }
                    let peak_level = session.cast::<IAudioMeterInformation>().and_then(|m| m.GetPeakValue()).unwrap_or(0.0);

                    // --- 強力な Chromium 検出ロジック ---
                    let mut name_found = None;
                    let mut icon_found = None;

                    // 1. セッション識別子から直接特定 (最優先)
                    let id_lower = session_id_str.to_lowercase();
                    if id_lower.contains("chrome.exe") { name_found = Some("Google Chrome".to_string()); }
                    else if id_lower.contains("brave.exe") { name_found = Some("Brave Browser".to_string()); }
                    else if id_lower.contains("msedge.exe") { name_found = Some("Microsoft Edge".to_string()); }

                    // 2. プロセス名とアイコンの取得
                    if pid != 0 {
                        if let Some(full_path) = Self::get_process_full_path(pid) {
                            if name_found.is_none() {
                                name_found = Some(icon::extract_product_name(&full_path).unwrap_or_else(|| {
                                    full_path.rsplit('\\').next().unwrap_or(&full_path).to_string()
                                }));
                            }
                            icon_found = icon::extract_icon_base64(&full_path);
                        }
                    }

                    let final_name = if pid == 0 {
                        "System Sounds".to_string()
                    } else {
                        name_found.unwrap_or_else(|| {
                            let disp = control2.GetDisplayName().map(|n| n.to_string().unwrap_or_default()).unwrap_or_default();
                            if !disp.is_empty() && !disp.starts_with('@') { disp }
                            else { format!("Application (PID: {})", pid) }
                        })
                    };

                    sessions_info.push(AudioSessionInfo {
                        process_id: pid,
                        process_name: final_name,
                        volume: vol,
                        is_muted: mute.as_bool(),
                        peak_level,
                        icon_base64: icon_found,
                        device_id: device_id.clone(),
                    });

                    CoTaskMemFree(Some(session_id_pwstr.as_ptr() as _));
                }
            }
        }
        Ok(sessions_info)
    }

    fn get_process_full_path(pid: u32) -> Option<String> {
        unsafe {
            use windows::Win32::System::Threading::{OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_WIN32, PROCESS_QUERY_LIMITED_INFORMATION};
            use windows::Win32::Foundation::HANDLE;
            let handle: HANDLE = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid).ok()?;
            let mut buffer = [0u16; MAX_PATH as usize * 2];
            let mut len = (MAX_PATH * 2) as u32;
            let res = QueryFullProcessImageNameW(handle, PROCESS_NAME_WIN32, windows::core::PWSTR(buffer.as_mut_ptr()), &mut len);
            let _ = windows::Win32::Foundation::CloseHandle(handle);
            if res.is_ok() {
                if let Ok(full_path) = String::from_utf16(&buffer[..len as usize]) {
                    return Some(full_path);
                }
            }
            None
        }
    }

    pub fn set_session_volume(&self, target_pid: u32, volume: f32) -> Result<()> {
        self.apply_to_session(target_pid, |simple_volume| unsafe {
            simple_volume.SetMasterVolume(volume, ptr::null())
        })
    }

    pub fn set_session_mute(&self, target_pid: u32, mute: bool) -> Result<()> {
        self.apply_to_session(target_pid, |simple_volume| unsafe {
            simple_volume.SetMute(mute, ptr::null())
        })
    }

    fn apply_to_session<F>(&self, target_pid: u32, action: F) -> Result<()>
    where
        F: Fn(&ISimpleAudioVolume) -> Result<()>,
    {
        unsafe {
            let collection = self.device_enumerator.EnumAudioEndpoints(eRender, DEVICE_STATE(DEVICE_STATEMASK_ALL))?;
            let device_count = collection.GetCount()?;
            for d in 0..device_count {
                let device = collection.Item(d)?;
                if let Ok(sm) = device.Activate::<IAudioSessionManager2>(CLSCTX_ALL, None) {
                    if let Ok(en) = sm.GetSessionEnumerator() {
                        let count = en.GetCount()?;
                        for i in 0..count {
                            let session = en.GetSession(i)?;
                            if let Ok(control2) = session.cast::<IAudioSessionControl2>() {
                                if control2.GetProcessId().unwrap_or(u32::MAX) == target_pid {
                                    if let Ok(sv) = session.cast::<ISimpleAudioVolume>() {
                                        return action(&sv);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }

    pub fn set_audio_routing(&self, target_pid: u32, device_id: &str) -> Result<()> {
        if target_pid == 0 { return Ok(()); }
        let factory = policy::AudioPolicyConfigFactory::new()?;
        factory.set_persisted_default_audio_endpoint(target_pid, device_id)?;
        Ok(())
    }

    pub fn get_peak_levels(&self) -> Result<Vec<(u32, f32)>> {
        let mut peaks = Vec::new();
        unsafe {
            let collection = self.device_enumerator.EnumAudioEndpoints(eRender, DEVICE_STATE(DEVICE_STATEMASK_ALL))?;
            let device_count = collection.GetCount()?;
            for d in 0..device_count {
                let device = collection.Item(d)?;
                if let Ok(sm) = device.Activate::<IAudioSessionManager2>(CLSCTX_ALL, None) {
                    if let Ok(en) = sm.GetSessionEnumerator() {
                        let count = en.GetCount()?;
                        for i in 0..count {
                            let session = en.GetSession(i)?;
                            if let Ok(ctrl) = session.cast::<IAudioSessionControl2>() {
                                let pid = ctrl.GetProcessId().unwrap_or(u32::MAX);
                                if let Ok(meter) = session.cast::<IAudioMeterInformation>() {
                                    peaks.push((pid, meter.GetPeakValue().unwrap_or(0.0)));
                                }
                            }
                        }
                    }
                }
            }
        }
        Ok(peaks)
    }

    pub fn get_audio_devices(&self) -> Result<Vec<AudioDeviceInfo>> {
        let mut devices = Vec::new();
        unsafe {
            use windows::Win32::Devices::Properties::DEVPKEY_Device_FriendlyName;
            use windows::Win32::System::Com::StructuredStorage::PropVariantClear;
            use windows::Win32::System::Com::STGM_READ;
            use windows::Win32::UI::Shell::PropertiesSystem::IPropertyStore;
            use windows::Win32::Media::Audio::DEVICE_STATE_ACTIVE;
            let collection = self.device_enumerator.EnumAudioEndpoints(eRender, DEVICE_STATE_ACTIVE)?;
            let count = collection.GetCount()?;
            for i in 0..count {
                let device = collection.Item(i)?;
                let id_pwstr = device.GetId()?;
                let id_string = id_pwstr.to_string().unwrap_or_default();
                CoTaskMemFree(Some(id_pwstr.as_ptr() as _));
                let store: IPropertyStore = device.OpenPropertyStore(STGM_READ)?;
                let mut prop_variant = store.GetValue(&DEVPKEY_Device_FriendlyName as *const _ as *const _)?;
                let name = {
                    let raw = prop_variant.as_raw();
                    if raw.Anonymous.Anonymous.vt == 31 {
                        let ptr = raw.Anonymous.Anonymous.Anonymous.pwszVal;
                        if ptr.is_null() { "Unknown Device".to_string() }
                        else {
                            let mut len = 0;
                            while *ptr.add(len) != 0 { len += 1; }
                            String::from_utf16_lossy(std::slice::from_raw_parts(ptr, len))
                        }
                    } else { "Unknown Device".to_string() }
                };
                let _ = PropVariantClear(&mut prop_variant);
                devices.push(AudioDeviceInfo { id: id_string, name });
            }
        }
        Ok(devices)
    }
}
