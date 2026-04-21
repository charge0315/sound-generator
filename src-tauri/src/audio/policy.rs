use windows::core::{Interface, GUID, HRESULT, HSTRING, IUnknown};
use windows::Win32::System::WinRT::RoGetActivationFactory;

// --- IAudioPolicyConfigFactory インターフェースの定義 ---
// Windows の非公開 API IAudioPolicyConfigFactory を定義します。

#[windows_core::interface("ab3d4648-e242-459f-b02f-541c70306324")]
pub unsafe trait IAudioPolicyConfigFactoryVariantFor21H2 {
    // IInspectable methods (Index 3, 4, 5)
    // 注意: IUnknown の 3 メソッドは interface マクロにより自動的に先頭に配置されます。
    fn GetIids(&self, count: *mut u32, iids: *mut *mut GUID) -> HRESULT;
    fn GetRuntimeClassName(&self, class_name: *mut *mut std::ffi::c_void) -> HRESULT;
    fn GetTrustLevel(&self, trust_level: *mut i32) -> HRESULT;

    // Custom methods (Index 6 to 24: 19 incomplete methods)
    fn __incomplete__add_CtxVolumeChange(&self) -> HRESULT;
    fn __incomplete__remove_CtxVolumeChanged(&self) -> HRESULT;
    fn __incomplete__add_RingerVibrateStateChanged(&self) -> HRESULT;
    fn __incomplete__remove_RingerVibrateStateChange(&self) -> HRESULT;
    fn __incomplete__SetVolumeGroupGainForId(&self) -> HRESULT;
    fn __incomplete__GetVolumeGroupGainForId(&self) -> HRESULT;
    fn __incomplete__GetActiveVolumeGroupForEndpointId(&self) -> HRESULT;
    fn __incomplete__GetVolumeGroupsForEndpoint(&self) -> HRESULT;
    fn __incomplete__GetCurrentVolumeContext(&self) -> HRESULT;
    fn __incomplete__SetVolumeGroupMuteForId(&self) -> HRESULT;
    fn __incomplete__GetVolumeGroupMuteForId(&self) -> HRESULT;
    fn __incomplete__SetRingerVibrateState(&self) -> HRESULT;
    fn __incomplete__GetRingerVibrateState(&self) -> HRESULT;
    fn __incomplete__SetPreferredChatApplication(&self) -> HRESULT;
    fn __incomplete__ResetPreferredChatApplication(&self) -> HRESULT;
    fn __incomplete__GetPreferredChatApplication(&self) -> HRESULT;
    fn __incomplete__GetCurrentChatApplications(&self) -> HRESULT;
    fn __incomplete__add_ChatContextChanged(&self) -> HRESULT;
    fn __incomplete__remove_ChatContextChanged(&self) -> HRESULT;

    // Actual target method (Index 25)
    pub unsafe fn SetPersistedDefaultAudioEndpoint(
        &self,
        process_id: u32,
        flow: i32,
        role: i32,
        device_id: &HSTRING,
    ) -> HRESULT;

    pub unsafe fn GetPersistedDefaultAudioEndpoint(
        &self,
        process_id: u32,
        flow: i32,
        role: i32,
        device_id: *mut HSTRING,
    ) -> HRESULT;

    pub unsafe fn ClearAllPersistedApplicationDefaultEndpoints(&self) -> HRESULT;
}

#[windows_core::interface("2a59116d-6c4f-45e0-a74f-707e3fef9258")]
pub unsafe trait IAudioPolicyConfigFactoryVariantForDownlevel {
    // IInspectable methods (Index 3, 4, 5)
    fn GetIids(&self, count: *mut u32, iids: *mut *mut GUID) -> HRESULT;
    fn GetRuntimeClassName(&self, class_name: *mut *mut std::ffi::c_void) -> HRESULT;
    fn GetTrustLevel(&self, trust_level: *mut i32) -> HRESULT;

    // Custom methods (Index 6 to 24: 19 incomplete methods)
    fn __incomplete__add_CtxVolumeChange(&self) -> HRESULT;
    fn __incomplete__remove_CtxVolumeChanged(&self) -> HRESULT;
    fn __incomplete__add_RingerVibrateStateChanged(&self) -> HRESULT;
    fn __incomplete__remove_RingerVibrateStateChange(&self) -> HRESULT;
    fn __incomplete__SetVolumeGroupGainForId(&self) -> HRESULT;
    fn __incomplete__GetVolumeGroupGainForId(&self) -> HRESULT;
    fn __incomplete__GetActiveVolumeGroupForEndpointId(&self) -> HRESULT;
    fn __incomplete__GetVolumeGroupsForEndpoint(&self) -> HRESULT;
    fn __incomplete__GetCurrentVolumeContext(&self) -> HRESULT;
    fn __incomplete__SetVolumeGroupMuteForId(&self) -> HRESULT;
    fn __incomplete__GetVolumeGroupMuteForId(&self) -> HRESULT;
    fn __incomplete__SetRingerVibrateState(&self) -> HRESULT;
    fn __incomplete__GetRingerVibrateState(&self) -> HRESULT;
    fn __incomplete__SetPreferredChatApplication(&self) -> HRESULT;
    fn __incomplete__ResetPreferredChatApplication(&self) -> HRESULT;
    fn __incomplete__GetPreferredChatApplication(&self) -> HRESULT;
    fn __incomplete__GetCurrentChatApplications(&self) -> HRESULT;
    fn __incomplete__add_ChatContextChanged(&self) -> HRESULT;
    fn __incomplete__remove_ChatContextChanged(&self) -> HRESULT;

    // Actual target method (Index 25)
    pub unsafe fn SetPersistedDefaultAudioEndpoint(
        &self,
        process_id: u32,
        flow: i32,
        role: i32,
        device_id: &HSTRING,
    ) -> HRESULT;

    pub unsafe fn GetPersistedDefaultAudioEndpoint(
        &self,
        process_id: u32,
        flow: i32,
        role: i32,
        device_id: *mut HSTRING,
    ) -> HRESULT;

    pub unsafe fn ClearAllPersistedApplicationDefaultEndpoints(&self) -> HRESULT;
}

pub enum AudioPolicyConfigFactoryVariant {
    Variant21H2(IAudioPolicyConfigFactoryVariantFor21H2),
    VariantDownlevel(IAudioPolicyConfigFactoryVariantForDownlevel),
}

pub struct AudioPolicyConfigFactory {
    variant: AudioPolicyConfigFactoryVariant,
}

impl AudioPolicyConfigFactory {
    pub fn new() -> windows::core::Result<Self> {
        let class_id = HSTRING::from("Windows.Media.Internal.AudioPolicyConfig");
        let unknown: IUnknown = unsafe { RoGetActivationFactory(&class_id)? };

        // 21H2 バリアントから試行します
        if let Ok(v21h2) = unknown.cast::<IAudioPolicyConfigFactoryVariantFor21H2>() {
            return Ok(Self {
                variant: AudioPolicyConfigFactoryVariant::Variant21H2(v21h2),
            });
        }

        // 失敗した場合は Downlevel バリアントを試行します
        if let Ok(vdown) = unknown.cast::<IAudioPolicyConfigFactoryVariantForDownlevel>() {
            return Ok(Self {
                variant: AudioPolicyConfigFactoryVariant::VariantDownlevel(vdown),
            });
        }

        Err(windows::core::Error::from_win32())
    }

    pub fn set_persisted_default_audio_endpoint(
        &self,
        process_id: u32,
        device_id: &str,
    ) -> windows::core::Result<()> {
        let device_id_hstring = HSTRING::from(device_id);
        let flow_render = 0; // eRender
        let role_console = 0; // eConsole
        let role_multimedia = 1; // eMultimedia
        let role_communications = 2; // eCommunications

        unsafe {
            match &self.variant {
                AudioPolicyConfigFactoryVariant::Variant21H2(v) => {
                    v.SetPersistedDefaultAudioEndpoint(process_id, flow_render, role_console, &device_id_hstring).ok()?;
                    v.SetPersistedDefaultAudioEndpoint(process_id, flow_render, role_multimedia, &device_id_hstring).ok()?;
                    v.SetPersistedDefaultAudioEndpoint(process_id, flow_render, role_communications, &device_id_hstring).ok()?;
                }
                AudioPolicyConfigFactoryVariant::VariantDownlevel(v) => {
                    v.SetPersistedDefaultAudioEndpoint(process_id, flow_render, role_console, &device_id_hstring).ok()?;
                    v.SetPersistedDefaultAudioEndpoint(process_id, flow_render, role_multimedia, &device_id_hstring).ok()?;
                    v.SetPersistedDefaultAudioEndpoint(process_id, flow_render, role_communications, &device_id_hstring).ok()?;
                }
            }
        }
        Ok(())
    }
}
