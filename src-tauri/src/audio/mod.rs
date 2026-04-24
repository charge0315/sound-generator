pub mod com;
pub mod events;
pub mod icon;
pub mod policy_v2;

use std::collections::{HashMap, HashSet};
use std::ptr;
use windows::core::{Interface, Result, HSTRING};
use windows::Win32::Media::Audio::{
    eRender, IMMDeviceEnumerator, MMDeviceEnumerator, DEVICE_STATE_ACTIVE,
    IAudioSessionManager2, IAudioSessionControl2,
    ISimpleAudioVolume, eConsole, eMultimedia, eCommunications
};
use windows::Win32::Media::Audio::Endpoints::IAudioMeterInformation;
use windows::Win32::System::Com::{CoCreateInstance, CLSCTX_ALL, CoTaskMemFree};
use windows::Win32::System::Threading::{OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION};
use windows::Win32::Foundation::{CloseHandle, HANDLE};
use tauri::AppHandle;

#[derive(Debug, serde::Serialize, Clone)]
pub struct AudioSessionInfo {
    pub process_id: u32,
    pub process_name: String,
    pub volume: f32,
    pub is_muted: bool,
    pub peak_level: f32,
    pub icon_base64: Option<String>,
    pub device_id: String,
}

#[derive(Debug, serde::Serialize, Clone)]
pub struct AudioDeviceInfo {
    pub id: String,
    pub name: String,
    pub is_default: bool,
}

pub struct AudioManager {
    device_enumerator: IMMDeviceEnumerator,
    app_handle: Option<AppHandle>,
    process_handles: HashMap<u32, HANDLE>,
    meter_cache: HashMap<String, IAudioMeterInformation>,
}

unsafe impl Send for AudioManager {}
unsafe impl Sync for AudioManager {}

impl Drop for AudioManager {
    fn drop(&mut self) {
        for (_, handle) in self.process_handles.drain() {
            unsafe { let _ = CloseHandle(handle); }
        }
    }
}

impl AudioManager {
    pub fn new() -> Result<Self> {
        let device_enumerator: IMMDeviceEnumerator = unsafe {
            CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL)?
        };
        Ok(Self {
            device_enumerator,
            app_handle: None,
            process_handles: HashMap::new(),
            meter_cache: HashMap::new(),
        })
    }

    pub fn set_app_handle(&mut self, handle: AppHandle) {
        self.app_handle = Some(handle);
    }

    pub fn get_sessions(&mut self) -> Result<Vec<AudioSessionInfo>> {
        let mut sessions = Vec::new();
        let mut active_session_keys = HashSet::new();
        let mut active_pids = HashSet::new();

        unsafe {
            let collection = self.device_enumerator.EnumAudioEndpoints(eRender, DEVICE_STATE_ACTIVE)?;
            let count = collection.GetCount()?;

            for i in 0..count {
                let device = collection.Item(i)?;
                let id_pwstr = device.GetId()?;
                let device_id = id_pwstr.to_string().unwrap_or_default();
                CoTaskMemFree(Some(id_pwstr.as_ptr() as _));

                if let Ok(session_manager) = device.Activate::<IAudioSessionManager2>(CLSCTX_ALL, None) {
                    if let Ok(enumerator) = session_manager.GetSessionEnumerator() {
                        let session_count = enumerator.GetCount()?;
                        for j in 0..session_count {
                            let session = enumerator.GetSession(j)?;
                            if let Ok(control2) = session.cast::<IAudioSessionControl2>() {
                                let pid = control2.GetProcessId().unwrap_or(0);
                                let session_key = format!("{}-{}", pid, device_id);
                                active_session_keys.insert(session_key.clone());

                                if pid != 0 {
                                    if !self.is_process_alive(pid) { continue; }
                                    active_pids.insert(pid);
                                }

                                if let (Ok(vol), Ok(meter)) = (session.cast::<ISimpleAudioVolume>(), session.cast::<IAudioMeterInformation>()) {
                                    let volume = vol.GetMasterVolume().unwrap_or(1.0);
                                    let muted = vol.GetMute().map(|m| m.as_bool()).unwrap_or(false);
                                    let peak = meter.GetPeakValue().unwrap_or(0.0);

                                    self.meter_cache.insert(session_key, meter);

                                    let process_name = if pid == 0 {
                                        "System Sounds".to_string()
                                    } else {
                                        icon::get_process_name(pid).unwrap_or_else(|| format!("PROCESS {}", pid))
                                    };
                                    
                                    let icon_base64 = if pid == 0 { None } else { icon::extract_icon_base64(pid) };

                                    sessions.push(AudioSessionInfo {
                                        process_id: pid,
                                        process_name,
                                        volume,
                                        is_muted: muted,
                                        peak_level: peak,
                                        icon_base64,
                                        device_id: device_id.clone(),
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }

        self.process_handles.retain(|pid, _| active_pids.contains(pid));
        self.meter_cache.retain(|key, _| active_session_keys.contains(key));

        Ok(sessions)
    }

    fn is_process_alive(&mut self, pid: u32) -> bool {
        if let Some(&handle) = self.process_handles.get(&pid) {
            let mut exit_code = 0u32;
            unsafe {
                if windows::Win32::System::Threading::GetExitCodeProcess(handle, &mut exit_code).is_ok() {
                    if exit_code == 259 { return true; }
                }
            }
            unsafe { let _ = CloseHandle(handle); }
            self.process_handles.remove(&pid);
        }
        unsafe {
            match OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid) {
                Ok(handle) => {
                    self.process_handles.insert(pid, handle);
                    true
                }
                Err(_) => false,
            }
        }
    }

    pub fn set_session_volume(&self, pid: u32, volume: f32) -> Result<()> {
        self.apply_to_session(pid, |sv| unsafe { sv.SetMasterVolume(volume, ptr::null()) })
    }

    pub fn set_session_mute(&self, pid: u32, mute: bool) -> Result<()> {
        self.apply_to_session(pid, |sv| unsafe { sv.SetMute(mute, ptr::null()) })
    }

    fn apply_to_session<F>(&self, target_pid: u32, action: F) -> Result<()>
    where
        F: Fn(&ISimpleAudioVolume) -> Result<()>,
    {
        unsafe {
            let collection = self.device_enumerator.EnumAudioEndpoints(eRender, DEVICE_STATE_ACTIVE)?;
            for i in 0..collection.GetCount()? {
                let device = collection.Item(i)?;
                if let Ok(sm) = device.Activate::<IAudioSessionManager2>(CLSCTX_ALL, None) {
                    if let Ok(en) = sm.GetSessionEnumerator() {
                        for j in 0..en.GetCount()? {
                            let session = en.GetSession(j)?;
                            if let Ok(control2) = session.cast::<IAudioSessionControl2>() {
                                if control2.GetProcessId().unwrap_or(0) == target_pid {
                                    if let Ok(sv) = session.cast::<ISimpleAudioVolume>() {
                                        let _ = action(&sv);
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

    pub fn set_audio_routing(&self, pid: u32, device_id: &str) -> Result<()> {
        let config = policy_v2::AudioPolicyConfigFactory::new()?;
        let endpoint_hstring = HSTRING::from(device_id);
        unsafe {
            // 3つの役割すべてに対して設定を行うことで、確実な切り替えを実現
            let _ = config.set_persisted_default_endpoint(pid, eConsole, &endpoint_hstring);
            let _ = config.set_persisted_default_endpoint(pid, eMultimedia, &endpoint_hstring);
            let _ = config.set_persisted_default_endpoint(pid, eCommunications, &endpoint_hstring);
        }
        Ok(())
    }

    pub fn get_audio_devices(&self) -> Result<Vec<AudioDeviceInfo>> {
        let mut devices = Vec::new();
        unsafe {
            use windows::Win32::Devices::Properties::DEVPKEY_Device_FriendlyName;
            use windows::Win32::UI::Shell::PropertiesSystem::PROPERTYKEY;
            use windows::Win32::System::Com::STGM_READ;

            let collection = self.device_enumerator.EnumAudioEndpoints(eRender, DEVICE_STATE_ACTIVE)?;
            let default_device = self.device_enumerator.GetDefaultAudioEndpoint(eRender, eConsole)?;
            let default_id_pwstr = default_device.GetId()?;
            let default_id = default_id_pwstr.to_string().unwrap_or_default();
            CoTaskMemFree(Some(default_id_pwstr.as_ptr() as _));

            for i in 0..collection.GetCount()? {
                let device = collection.Item(i)?;
                let id_pwstr = device.GetId()?;
                let id = id_pwstr.to_string().unwrap_or_default();
                CoTaskMemFree(Some(id_pwstr.as_ptr() as _));
                
                let is_default = id == default_id;

                if let Ok(store) = device.OpenPropertyStore(STGM_READ) {
                    let prop_key = PROPERTYKEY {
                        fmtid: DEVPKEY_Device_FriendlyName.fmtid,
                        pid: DEVPKEY_Device_FriendlyName.pid,
                    };
                    let name = store.GetValue(&prop_key).map(|v| v.to_string()).unwrap_or_else(|_| "Unknown Device".to_string());
                    devices.push(AudioDeviceInfo { id, name, is_default });
                }
            }
        }
        Ok(devices)
    }

    pub fn get_peak_levels(&self) -> Result<Vec<serde_json::Value>> {
        let mut peaks = Vec::new();
        for (key, meter) in &self.meter_cache {
            unsafe {
                if let Ok(peak) = meter.GetPeakValue() {
                    let pid_str = key.split('-').next().unwrap_or("0");
                    let pid = pid_str.parse::<u32>().unwrap_or(0);
                    peaks.push(serde_json::json!({ "pid": pid, "peak": peak }));
                }
            }
        }
        Ok(peaks)
    }
}
