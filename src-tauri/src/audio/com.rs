use windows::Win32::System::Com::{CoInitializeEx, COINIT_MULTITHREADED};
use windows::core::Result;

/// オーディオ操作に必要な Multi-Threaded Apartment (MTA) を初期化します。
pub fn init_mta() -> Result<()> {
    unsafe {
        CoInitializeEx(None, COINIT_MULTITHREADED).ok()
    }
}
