use windows::{
    core::{implement, Result},
    Win32::Media::Audio::{AudioSessionState, IAudioSessionEvents, IAudioSessionEvents_Impl},
};

/// 音量やミュート状態の変化を監視するイベントリスナー
///
/// COMのコールバックを受け取り、Rust側のチャネルやTauriのアプリアンドルを通じて
/// フロントエンドへ変更を通知する役割を担う。
/// windows-rsの `implement` マクロを使うことで、IUnknownのボイラープレート（AddRef/Release等）を
/// マクロがよしなに（安全に）自動生成してくれる。
#[implement(IAudioSessionEvents)]
pub struct SessionEventsListener {
    // 将来的にはここにTauriのAppHandleやmpsc::Senderを持たせて、
    // 状態が変わったよーというイベントをUI側に投げる
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
        // 音量・ミュートが変わった！という通知をここでフックする
        // ※ ここはCOMのバックグラウンドスレッドで呼ばれるため、ブロックする処理は厳禁
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
