#![allow(non_snake_case)]
#![allow(non_camel_case_types)]

use windows::core::{Interface, GUID, HRESULT, HSTRING, IUnknown};
use windows::Win32::System::WinRT::RoGetActivationFactory;

// --- IAudioPolicyConfigFactory インターフェースの定義 ---
// EarTrumpet の C# 実装と 100% ABI 互換を持たせた定義

#[windows_core::interface("ab3d4648-e242-459f-b02f-541c70306324")]
pub unsafe trait IAudioPolicyConfigFactoryWin11 {
    // IInspectable methods
    fn GetIids(&self, count: *mut u32, iids: *mut *mut GUID) -> HRESULT;
    fn GetRuntimeClassName(&self, class_name: *mut *mut std::ffi::c_void) -> HRESULT;
    fn GetTrustLevel(&self, trust_level: *mut i32) -> HRESULT;

    // 19 dummy methods to match VTable offset
    fn d1(&self) -> HRESULT; fn d2(&self) -> HRESULT; fn d3(&self) -> HRESULT; fn d4(&self) -> HRESULT;
    fn d5(&self) -> HRESULT; fn d6(&self) -> HRESULT; fn d7(&self) -> HRESULT; fn d8(&self) -> HRESULT;
    fn d9(&self) -> HRESULT; fn d10(&self) -> HRESULT; fn d11(&self) -> HRESULT; fn d12(&self) -> HRESULT;
    fn d13(&self) -> HRESULT; fn d14(&self) -> HRESULT; fn d15(&self) -> HRESULT; fn d16(&self) -> HRESULT;
    fn d17(&self) -> HRESULT; fn d18(&self) -> HRESULT; fn d19(&self) -> HRESULT;

    // Index 25: SetPersistedDefaultAudioEndpoint
    // ABI 的に最も安全な *const std::ffi::c_void (HSTRINGハンドル) を使用
    pub unsafe fn SetPersistedDefaultAudioEndpoint(
        &self,
        process_id: u32,
        flow: i32,
        role: i32,
        device_id: *const std::ffi::c_void,
    ) -> HRESULT;
}

pub struct AudioPolicyConfigFactory {
    inner: IUnknown,
}

impl AudioPolicyConfigFactory {
    pub fn new() -> windows::core::Result<Self> {
        let class_id = HSTRING::from("Windows.Media.Internal.AudioPolicyConfig");
        let inner: IUnknown = unsafe { RoGetActivationFactory(&class_id)? };
        Ok(Self { inner })
    }

    pub fn set_persisted_default_audio_endpoint(
        &self,
        process_id: u32,
        device_id: &str,
    ) -> windows::core::Result<()> {
        if device_id.is_empty() || process_id == 0 {
            return Ok(());
        }

        let device_id_hstring = HSTRING::from(device_id);
        let flow_render = 0; // eRender
        let role_multimedia = 1; // eMultimedia

        unsafe {
            if let Ok(factory) = self.inner.cast::<IAudioPolicyConfigFactoryWin11>() {
                // HSTRING の生ハンドルを渡す
                let _ = factory.SetPersistedDefaultAudioEndpoint(
                    process_id, 
                    flow_render, 
                    role_multimedia, 
                    device_id_hstring.as_ptr() as _
                );
            }
        }
        Ok(())
    }
}
