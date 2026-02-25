use windows::{
    core::{implement, Result},
    Win32::Media::Audio::{AudioSessionState, IAudioSessionEvents, IAudioSessionEvents_Impl},
};

use tauri::{AppHandle, Emitter};

// フロントエンドへ送信するイベントのペイロード
#[derive(Clone, serde::Serialize)]
pub struct AudioEventPayload {
    pub process_id: u32,
    pub volume: Option<f32>,
    pub mute: Option<bool>,
    pub state: Option<String>,
}

/// 音量やミュート状態の変化を監視するイベントリスナー
///
/// COMのコールバックを受け取り、Rust側のチャネルやTauriのアプリアンドルを通じて
/// フロントエンドへ変更を通知する役割を担う。
/// windows-rsの `implement` マクロを使うことで、IUnknownのボイラープレート（AddRef/Release等）を
/// マクロがよしなに（安全に）自動生成してくれる。
#[implement(IAudioSessionEvents)]
pub struct SessionEventsListener {
    // Tauriのアプリアンドルを持たせて、イベントをUI側に投げる
    pub app_handle: AppHandle,
    pub process_id: u32,
}

impl IAudioSessionEvents_Impl for SessionEventsListener_Impl {
    fn OnDisplayNameChanged(
        &self,
        _newdisplayname: &windows::core::PCWSTR,
        _eventcontext: *const windows::core::GUID,
    ) -> Result<()> {
        Ok(())
    }

    fn OnIconPathChanged(
        &self,
        _newiconpath: &windows::core::PCWSTR,
        _eventcontext: *const windows::core::GUID,
    ) -> Result<()> {
        Ok(())
    }

    fn OnSimpleVolumeChanged(
        &self,
        newvolume: f32,
        newmute: windows::Win32::Foundation::BOOL,
        _eventcontext: *const windows::core::GUID,
    ) -> Result<()> {
        // 音量・ミュートが変わった通知をフロントエンドへ飛ばす
        let payload = AudioEventPayload {
            process_id: self.process_id,
            volume: Some(newvolume),
            mute: Some(newmute.as_bool()),
            state: None, // Volume change doesn't explicitly change Active/Inactive state here
        };
        // エラーを無視（UIが閉じた後なども呼ばれる可能性があるため）
        let _ = self.app_handle.emit("audio-session-event", payload);

        println!(
            "VOL CHANGED: PID {}, Vol: {}, Mute: {:?}",
            self.process_id, newvolume, newmute
        );
        Ok(())
    }

    fn OnChannelVolumeChanged(
        &self,
        _channelcount: u32,
        _newchannelvolumearray: *const f32,
        _changedchannel: u32,
        _eventcontext: *const windows::core::GUID,
    ) -> Result<()> {
        Ok(())
    }

    fn OnGroupingParamChanged(
        &self,
        _newgroupingparam: *const windows::core::GUID,
        _eventcontext: *const windows::core::GUID,
    ) -> Result<()> {
        Ok(())
    }

    fn OnStateChanged(&self, newstate: AudioSessionState) -> Result<()> {
        let state_str = match newstate {
            windows::Win32::Media::Audio::AudioSessionStateInactive => "Inactive",
            windows::Win32::Media::Audio::AudioSessionStateActive => "Active",
            windows::Win32::Media::Audio::AudioSessionStateExpired => "Expired",
            _ => "Unknown",
        };

        let payload = AudioEventPayload {
            process_id: self.process_id,
            volume: None,
            mute: None,
            state: Some(state_str.to_string()),
        };
        let _ = self.app_handle.emit("audio-session-event", payload);

        // セッションがアクティブになった、期限切れになった、等の状態変更を通知
        println!(
            "STATE CHANGED: PID {}, State: {:?}",
            self.process_id, newstate
        );
        Ok(())
    }

    fn OnSessionDisconnected(
        &self,
        disconnectreason: windows::Win32::Media::Audio::AudioSessionDisconnectReason,
    ) -> Result<()> {
        println!(
            "DISCONNECTED: PID {}, Reason: {:?}",
            self.process_id, disconnectreason
        );
        Ok(())
    }
}
