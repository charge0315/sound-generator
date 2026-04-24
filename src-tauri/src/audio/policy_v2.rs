use windows::core::{IUnknown, IUnknown_Vtbl, Interface, GUID, HSTRING, HRESULT};
use windows::Win32::Media::Audio::ERole;

// 非公開インターフェース IAudioPolicyConfig の定義
// VTable Index 25 は EarTrumpet 等で使用されている 
// SetPersistedDefaultAudioEndpoint (Windows 10 Build 1709+) に相当します。

#[repr(C)]
#[allow(non_snake_case)]
pub struct IAudioPolicyConfig_Vtbl {
    pub base: IUnknown_Vtbl,
    pub AddPersistedDefaultAudioEndpoint: unsafe extern "system" fn(this: *mut core::ffi::c_void, process_id: u32, role: ERole, endpoint_id: *const core::ffi::c_void) -> HRESULT,
    pub RemovePersistedDefaultAudioEndpoint: unsafe extern "system" fn(this: *mut core::ffi::c_void, process_id: u32, role: ERole) -> HRESULT,
    pub GetPersistedDefaultAudioEndpoint: unsafe extern "system" fn(this: *mut core::ffi::c_void, process_id: u32, role: ERole, endpoint_id: *mut *mut core::ffi::c_void) -> HRESULT,
    // ... 中間のメソッド ...
    pub reserved: [usize; 21], // インデックス 25 までのパディング
    pub SetPersistedDefaultAudioEndpoint: unsafe extern "system" fn(this: *mut core::ffi::c_void, process_id: u32, role: ERole, endpoint_id: HSTRING) -> HRESULT,
}

#[repr(transparent)]
#[derive(Clone, PartialEq, Eq)]
pub struct IAudioPolicyConfig(IUnknown);

unsafe impl Interface for IAudioPolicyConfig {
    type Vtable = IAudioPolicyConfig_Vtbl;
    const IID: GUID = GUID::from_u128(0xf3419f00_83cd_4f30_b556_9074092d057a); // IID_IAudioPolicyConfig
}

impl IAudioPolicyConfig {
    /// 特定のプロセスのデフォルトオーディオエンドポイントを永続的に設定します。
    pub unsafe fn set_persisted_default_endpoint(&self, process_id: u32, role: ERole, endpoint_id: &HSTRING) -> windows::core::Result<()> {
        let vtbl = self.vtable();
        (vtbl.SetPersistedDefaultAudioEndpoint)(core::mem::transmute_copy(self), process_id, role, endpoint_id.clone()).ok()
    }
}

pub struct AudioPolicyConfigFactory;

impl AudioPolicyConfigFactory {
    pub fn new() -> windows::core::Result<IAudioPolicyConfig> {
        unsafe {
            windows::Win32::System::Com::CoCreateInstance(
                &windows::core::GUID::from_u128(0x870af99c_171d_4f9e_af0d_e63df40c2bc9), // CLSID_AudioPolicyConfig
                None,
                windows::Win32::System::Com::CLSCTX_ALL,
            )
        }
    }
}

unsafe impl Send for IAudioPolicyConfig {}
unsafe impl Sync for IAudioPolicyConfig {}
