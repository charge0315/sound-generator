pub mod events;

use std::ptr;
use windows::{
    core::{Interface, Result},
    Win32::{
        Foundation::{HANDLE, MAX_PATH},
        Media::Audio::{
            eConsole, eRender, IAudioSessionControl2, IAudioSessionEnumerator,
            IAudioSessionManager2, IMMDevice, IMMDeviceEnumerator, ISimpleAudioVolume,
            MMDeviceEnumerator,
        },
        System::Com::{CoCreateInstance, CoInitializeEx, CLSCTX_ALL, COINIT_MULTITHREADED},
        System::ProcessStatus::K32GetProcessImageFileNameW,
        System::Threading::{OpenProcess, PROCESS_QUERY_INFORMATION, PROCESS_VM_READ},
    },
};

/// アプリケーション起動時に呼ばれるべき、COMの初期化処理
///
/// COMはスレッドごとに初期化が必要。Tauriのイベントループやサウンド制御は
/// 並行して動作する可能性が高いため、マルチスレッドアパートメント(MTA)として初期化する。
pub fn init_com() -> Result<()> {
    unsafe {
        // CoInitializeExは、現在のスレッドのCOMライブラリを初期化する
        // MTAを指定し、スレッドセーフなCOMオブジェクトの呼び出しを許可する
        let _ = CoInitializeEx(Some(ptr::null()), COINIT_MULTITHREADED);
    }
    // CoInitializeExが S_FALSE を返すことがあるが（既に初期化済みなど）、
    // これはエラーではないため、厳密なエラーチェックは省略または許容する
    Ok(())
}

/// フロントエンドに返すための、各オーディオセッションの情報
#[derive(Debug, serde::Serialize)]
pub struct AudioSessionInfo {
    pub process_id: u32,
    pub process_name: String,
    pub volume: f32,
    pub is_muted: bool,
}

pub struct AudioManager {
    device_enumerator: IMMDeviceEnumerator,
}

// IMMDeviceEnumeratorをはじめとするCOMインターフェースは標準ではスレッドセーフ（Send/Sync）とは
// みなされないことが多いですが、MTA（マルチスレッドアパートメント）として初期化していれば、
// 異なるスレッドから呼び出しても安全に（プロキシ経由などで）処理されます。
// TauriのStateとして共有するために、自己責任でSendとSyncを付与します。
unsafe impl Send for AudioManager {}
unsafe impl Sync for AudioManager {}

impl AudioManager {
    pub fn new() -> Result<Self> {
        unsafe {
            let device_enumerator: IMMDeviceEnumerator =
                CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL)?;
            Ok(Self { device_enumerator })
        }
    }

    /// デフォルトの再生デバイス（スピーカー等）を取得する
    fn get_default_render_device(&self) -> Result<IMMDevice> {
        unsafe {
            self.device_enumerator
                .GetDefaultAudioEndpoint(eRender, eConsole)
        }
    }

    /// 現在アクティブな全オーディオセッションの情報を列挙して返す
    pub fn get_sessions(&self) -> Result<Vec<AudioSessionInfo>> {
        let mut sessions_info = Vec::new();

        unsafe {
            let device = self.get_default_render_device()?;
            // IAudioSessionManager2を取得してセッションを列挙する
            let session_manager: IAudioSessionManager2 = device.Activate(CLSCTX_ALL, None)?;
            let enumerator: IAudioSessionEnumerator = session_manager.GetSessionEnumerator()?;

            let count = enumerator.GetCount()?;

            for i in 0..count {
                let session = enumerator.GetSession(i)?;
                let control_query: Result<IAudioSessionControl2> = session.cast();

                if let Ok(control2) = control_query {
                    let pid = control2.GetProcessId()?;
                    // PID 0 はシステム音なのでスキップまたは特別扱い
                    if pid == 0 {
                        continue;
                    }

                    // 音量情報の取得 (ISimpleAudioVolume)
                    let simple_volume: Result<ISimpleAudioVolume> = session.cast();
                    let (mut vol, mut mute) = (0.0, windows::Win32::Foundation::BOOL::default());

                    if let Ok(sv) = &simple_volume {
                        if let Ok(v) = sv.GetMasterVolume() {
                            vol = v;
                        }
                        if let Ok(m) = sv.GetMute() {
                            mute = m;
                        }
                    }

                    use events::SessionEventsListener;
                    // イベントリスナーの登録 (オプション)
                    // TODO: 本格的にイベントをUIへ送る場合は、このリスナーインスタンスを保持し続け、
                    // Unregisterする必要があります。今回は試験的に登録だけおこないます。
                    if let Ok(control) =
                        session.cast::<windows::Win32::Media::Audio::IAudioSessionControl>()
                    {
                        let listener: windows::Win32::Media::Audio::IAudioSessionEvents =
                            SessionEventsListener { process_id: pid }.into();
                        let _ = unsafe { control.RegisterAudioSessionNotification(&listener) };
                    }

                    // プロセス名を取得する
                    let process_name =
                        Self::get_process_name(pid).unwrap_or_else(|| "Unknown".to_string());

                    sessions_info.push(AudioSessionInfo {
                        process_id: pid,
                        process_name,
                        volume: vol,
                        is_muted: mute.as_bool(),
                    });
                }
            }
        }

        Ok(sessions_info)
    }

    /// 指定された PID から実行ファイル名を取得するヘルパー関数
    fn get_process_name(pid: u32) -> Option<String> {
        unsafe {
            // PROCESS_QUERY_INFORMATIONとPROCESS_VM_READ権限でプロセスを開く
            let handle: HANDLE =
                OpenProcess(PROCESS_QUERY_INFORMATION | PROCESS_VM_READ, false, pid).ok()?;

            let mut buffer = [0u16; MAX_PATH as usize];
            let len = K32GetProcessImageFileNameW(handle, &mut buffer);
            let _ = windows::Win32::Foundation::CloseHandle(handle);

            if len > 0 {
                // 返ってくるのはパス全体 (例: \\Device\\HarddiskVolume3\\Windows\\System32\\svchost.exe)
                if let Ok(full_path) = String::from_utf16(&buffer[..len as usize]) {
                    // 最後のバックスラッシュより後を抽出する
                    if let Some(pos) = full_path.rfind('\\') {
                        return Some(full_path[pos + 1..].to_string());
                    }
                    return Some(full_path);
                }
            }
            None
        }
    }

    /// 指定されたプロセスIDのオーディオセッションの音量を設定する（0.0 ~ 1.0）
    pub fn set_session_volume(&self, target_pid: u32, volume: f32) -> Result<()> {
        self.apply_to_session(target_pid, |simple_volume| unsafe {
            simple_volume.SetMasterVolume(volume, ptr::null())
        })
    }

    /// 指定されたプロセスIDのオーディオセッションのミュート状態を設定する
    pub fn set_session_mute(&self, target_pid: u32, mute: bool) -> Result<()> {
        self.apply_to_session(target_pid, |simple_volume| unsafe {
            simple_volume.SetMute(mute, ptr::null())
        })
    }

    /// 特定のPIDを持つセッションの ISimpleAudioVolume に対して処理を行うヘルパーメソッド
    fn apply_to_session<F>(&self, target_pid: u32, action: F) -> Result<()>
    where
        F: Fn(&ISimpleAudioVolume) -> Result<()>,
    {
        unsafe {
            let device = self.get_default_render_device()?;
            let session_manager: IAudioSessionManager2 = device.Activate(CLSCTX_ALL, None)?;
            let enumerator: IAudioSessionEnumerator = session_manager.GetSessionEnumerator()?;

            let count = enumerator.GetCount()?;

            for i in 0..count {
                let session = enumerator.GetSession(i)?;
                let control_query: Result<IAudioSessionControl2> = session.cast();

                if let Ok(control2) = &control_query {
                    let pid = control2.GetProcessId()?;
                    if pid == target_pid {
                        let simple_volume: Result<ISimpleAudioVolume> = session.cast();
                        if let Ok(sv) = &simple_volume {
                            return action(sv);
                        }
                    }
                }
            }
        }

        // PIDが見つからなかった、またはエラーだった場合は現状とりあえずOkを返すか、独自エラーにする
        // ここではAPIとしてエラーにせず、単に何も起きなかったのと同じ扱いにしている
        Ok(())
    }
}
