use std::ffi::c_void;
use windows::core::{IUnknown, HRESULT, HSTRING};
use windows::Win32::System::WinRT::RoGetActivationFactory;

// --- IAudioPolicyConfigFactory インターフェースのハック宣言 ---
// Windows上で「プロセスごとに異なるオーディオ出力先を割り当てる（Audio Routing）」ためのAPI群は、
// EarTrumpet等のアプリが利用しているものの、Windows SDK上で公式にヘッダファイルとして公開されていない「非公開API（Undocumented API）」である。
// そのため `windows-rs`（0.58等）には対応するインターフェース生成マクロが存在しないか不完全である。
// ここでは、COMの内部構造（VTable=関数ポインタの配列）のメモリレイアウトをRustの struct として直書き（ハードコーディング）し、
// ポインタをC言語のように直接操作・キャストすることで、強引に非公開APIを呼び出せるようにしている。

#[repr(C)]
#[allow(non_snake_case)]
pub struct IAudioPolicyConfigFactoryVtable {
    pub QueryInterface: unsafe extern "system" fn(
        this: *mut c_void,
        iid: *const std::ffi::c_void,
        ppv: *mut *mut c_void,
    ) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(this: *mut c_void) -> u32,
    pub Release: unsafe extern "system" fn(this: *mut c_void) -> u32,

    // IInspectable methods
    pub GetIids: unsafe extern "system" fn(
        this: *mut c_void,
        iidCount: *mut u32,
        iids: *mut *mut std::ffi::c_void,
    ) -> HRESULT,
    pub GetRuntimeClassName:
        unsafe extern "system" fn(this: *mut c_void, className: *mut *mut c_void) -> HRESULT,
    pub GetTrustLevel:
        unsafe extern "system" fn(this: *mut c_void, trustLevel: *mut i32) -> HRESULT,

    // IAudioPolicyConfigFactory methods
    // Indices 6 through 24 are padding/incomplete methods. (19 methods)
    pub padding: [unsafe extern "system" fn(); 19],

    // Index 25
    pub SetPersistedDefaultAudioEndpoint: unsafe extern "system" fn(
        this: *mut c_void,
        processId: u32,
        flow: i32,
        role: i32,
        deviceId: *mut c_void, // HSTRING ABI pointer
    ) -> HRESULT,

    pub GetPersistedDefaultAudioEndpoint: unsafe extern "system" fn(
        this: *mut c_void,
        processId: u32,
        flow: i32,
        role: i32,
        deviceId: *mut *mut c_void,
    ) -> HRESULT,

    pub ClearAllPersistedApplicationDefaultEndpoints:
        unsafe extern "system" fn(this: *mut c_void) -> HRESULT,
}

#[repr(C)]
pub struct IAudioPolicyConfigFactoryRaw {
    pub vtable: *const IAudioPolicyConfigFactoryVtable,
}

pub struct AudioPolicyConfigFactory {
    factory_raw: *mut IAudioPolicyConfigFactoryRaw,
    // Keep IUnknown alive so reference count doesn't drop
    _factory_unknown: IUnknown,
}

// GUID: ab3d4648-e242-459f-b02f-541c70306324
const IID_21H2: windows::core::GUID =
    windows::core::GUID::from_u128(0xab3d4648_e242_459f_b02f_541c70306324);
// GUID: 2a59116d-6c4f-45e0-a74f-707e3fef9258
const IID_DOWNLEVEL: windows::core::GUID =
    windows::core::GUID::from_u128(0x2a59116d_6c4f_45e0_a74f_707e3fef9258);

impl AudioPolicyConfigFactory {
    pub fn new() -> windows::core::Result<Self> {
        let class_id = HSTRING::from("Windows.Media.Internal.AudioPolicyConfig");

        let unknown: IUnknown = unsafe { RoGetActivationFactory(&class_id)? };

        // Try casting using raw COM QueryInterface
        let mut factory_ptr: *mut c_void = std::ptr::null_mut();

        unsafe {
            // we can get the raw pointer to IUnknown
            let unknown_raw: *mut c_void = std::mem::transmute_copy(&unknown);

            // Layout of IUnknown vtable is the standard QueryInterface, AddRef, Release
            let unknown_vtable_ptr =
                *(unknown_raw as *const *const IAudioPolicyConfigFactoryVtable);

            // Try 21H2 first
            let hr = ((*unknown_vtable_ptr).QueryInterface)(
                unknown_raw,
                &IID_21H2 as *const _ as *const c_void,
                &mut factory_ptr,
            );

            if hr.is_err() {
                // If it fails, try Downlevel
                factory_ptr = std::ptr::null_mut();
                let hr2 = ((*unknown_vtable_ptr).QueryInterface)(
                    unknown_raw,
                    &IID_DOWNLEVEL as *const _ as *const c_void,
                    &mut factory_ptr,
                );
                if hr2.is_err() {
                    return Err(windows::core::Error::empty());
                }
            }
        }

        if factory_ptr.is_null() {
            return Err(windows::core::Error::empty());
        }

        Ok(Self {
            factory_raw: factory_ptr as *mut IAudioPolicyConfigFactoryRaw,
            _factory_unknown: unknown,
        })
    }

    pub fn set_persisted_default_audio_endpoint(
        &self,
        process_id: u32,
        device_id: &str,
    ) -> windows::core::Result<()> {
        let device_id_hstring = HSTRING::from(device_id);

        // flow = 0 (eRender)
        // role = 1 (eMultimedia) and 0 (eConsole)
        let flow_render = 0;
        let role_multimedia = 1;
        let role_console = 0;

        // Since the interface expects *mut c_void for the HString handle,
        // we can cast the internal ABI pointer of HSTRING using std::mem::transmute_copy.
        // HSTRING wraps an HSTRING handle natively.
        let hstring_handle: *mut c_void = unsafe { std::mem::transmute_copy(&device_id_hstring) };

        unsafe {
            let vtable = (*self.factory_raw).vtable;

            let hr1 = ((*vtable).SetPersistedDefaultAudioEndpoint)(
                self.factory_raw as *mut c_void,
                process_id,
                flow_render,
                role_multimedia,
                hstring_handle,
            );

            let hr2 = ((*vtable).SetPersistedDefaultAudioEndpoint)(
                self.factory_raw as *mut c_void,
                process_id,
                flow_render,
                role_console,
                hstring_handle,
            );

            hr1.ok()?;
            hr2.ok()?;
        }

        Ok(())
    }
}
