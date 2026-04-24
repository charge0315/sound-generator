use tauri::{AppHandle, Emitter};
use windows::Win32::Media::Audio::{IAudioSessionEvents, IAudioSessionEvents_Impl, AudioSessionState};

#[windows_core::implement(IAudioSessionEvents)]
pub struct SessionEventsListener {
    pub app_handle: AppHandle,
    pub process_id: u32,
}

impl IAudioSessionEvents_Impl for SessionEventsListener_Impl {
    fn OnDisplayNameChanged(&self, _newdisplayname: &windows::core::PCWSTR, _eventcontext: *const windows::core::GUID) -> windows::core::Result<()> {
        Ok(())
    }
    fn OnIconPathChanged(&self, _newiconpath: &windows::core::PCWSTR, _eventcontext: *const windows::core::GUID) -> windows::core::Result<()> {
        Ok(())
    }
    fn OnSimpleVolumeChanged(&self, newvolume: f32, newmute: windows::Win32::Foundation::BOOL, _eventcontext: *const windows::core::GUID) -> windows::core::Result<()> {
        let _ = self.app_handle.emit("volume-change", serde_json::json!({
            "pid": self.process_id,
            "volume": newvolume,
            "muted": newmute.as_bool()
        }));
        Ok(())
    }
    fn OnChannelVolumeChanged(&self, _channelcount: u32, _newchannelvolumearray: *const f32, _changedchannel: u32, _eventcontext: *const windows::core::GUID) -> windows::core::Result<()> {
        Ok(())
    }
    fn OnGroupingParamChanged(&self, _newgroupingparam: *const windows::core::GUID, _eventcontext: *const windows::core::GUID) -> windows::core::Result<()> {
        Ok(())
    }
    fn OnStateChanged(&self, newstate: AudioSessionState) -> windows::core::Result<()> {
        let _ = self.app_handle.emit("session-state-change", serde_json::json!({
            "pid": self.process_id,
            "state": format!("{:?}", newstate)
        }));
        Ok(())
    }
    fn OnSessionDisconnected(&self, _disconnectreason: windows::Win32::Media::Audio::AudioSessionDisconnectReason) -> windows::core::Result<()> {
        let _ = self.app_handle.emit("refresh-trigger", ());
        Ok(())
    }
}
