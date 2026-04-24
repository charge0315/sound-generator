use std::path::Path;
use windows::Win32::Foundation::{MAX_PATH, HANDLE};
use windows::Win32::System::Threading::{OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_WIN32, PROCESS_QUERY_LIMITED_INFORMATION};
use windows::Win32::UI::Shell::{SHGetFileInfoW, SHGFI_ICON, SHGFI_LARGEICON, SHFILEINFOW};
use windows::Win32::UI::WindowsAndMessaging::{DestroyIcon, HICON, GetIconInfo, ICONINFO};
use windows::Win32::Graphics::Gdi::{
    GetDC, ReleaseDC, CreateCompatibleDC, SelectObject, DeleteDC, 
    DeleteObject, GetDIBits, BITMAPINFO, BITMAPINFOHEADER, DIB_RGB_COLORS, RGBQUAD
};
use base64::{engine::general_purpose, Engine as _};
use image::{RgbaImage, ImageFormat};
use std::io::Cursor;

pub fn get_process_name(pid: u32) -> Option<String> {
    unsafe {
        let handle: HANDLE = match OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid) {
            Ok(h) => h,
            Err(_) => return None,
        };
        let mut buffer = [0u16; MAX_PATH as usize * 2];
        let mut len = (MAX_PATH * 2) as u32;
        let res = QueryFullProcessImageNameW(handle, PROCESS_NAME_WIN32, windows::core::PWSTR(buffer.as_mut_ptr()), &mut len);
        let _ = windows::Win32::Foundation::CloseHandle(handle);
        if res.is_ok() {
            if let Ok(full_path) = String::from_utf16(&buffer[..len as usize]) {
                return Path::new(&full_path)
                    .file_name()
                    .and_then(|n| n.to_str())
                    .map(|s| s.to_string().to_uppercase());
            }
        }
        None
    }
}

pub fn extract_icon_base64(pid: u32) -> Option<String> {
    unsafe {
        let handle: HANDLE = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid).ok()?;
        let mut buffer = [0u16; MAX_PATH as usize * 2];
        let mut len = (MAX_PATH * 2) as u32;
        let res = QueryFullProcessImageNameW(handle, PROCESS_NAME_WIN32, windows::core::PWSTR(buffer.as_mut_ptr()), &mut len);
        let _ = windows::Win32::Foundation::CloseHandle(handle);
        
        if res.is_err() { return None; }
        let path_wstr: Vec<u16> = buffer[..len as usize].iter().cloned().chain(std::iter::once(0)).collect();

        let mut shfi: SHFILEINFOW = std::mem::zeroed();
        let res = SHGetFileInfoW(
            windows::core::PCWSTR(path_wstr.as_ptr()),
            windows::Win32::Storage::FileSystem::FILE_FLAGS_AND_ATTRIBUTES(0),
            Some(&mut shfi),
            std::mem::size_of::<SHFILEINFOW>() as u32,
            SHGFI_ICON | SHGFI_LARGEICON
        );

        if res == 0 || shfi.hIcon.is_invalid() { return None; }

        let base64 = hicon_to_base64(shfi.hIcon);
        let _ = DestroyIcon(shfi.hIcon);
        base64
    }
}

#[allow(non_snake_case)]
unsafe fn hicon_to_base64(hicon: HICON) -> Option<String> {
    let mut icon_info = ICONINFO::default();
    if GetIconInfo(hicon, &mut icon_info).is_err() { return None; }

    let hdc_screen = GetDC(None);
    let hdc_mem = CreateCompatibleDC(hdc_screen);
    let h_old_obj = SelectObject(hdc_mem, icon_info.hbmColor);

    let mut bmi = BITMAPINFO {
        bmiHeader: BITMAPINFOHEADER {
            biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
            biWidth: 32,
            biHeight: -32, // Top-down
            biPlanes: 1,
            biBitCount: 32,
            biCompression: 0, // BI_RGB
            ..Default::default()
        },
        bmiColors: [RGBQUAD::default(); 1],
    };

    let mut buffer: Vec<u8> = vec![0; 32 * 32 * 4];
    let lines = GetDIBits(hdc_mem, icon_info.hbmColor, 0, 32, Some(buffer.as_mut_ptr() as *mut _), &mut bmi, DIB_RGB_COLORS);

    // Cleanup
    SelectObject(hdc_mem, h_old_obj);
    let _ = DeleteDC(hdc_mem);
    ReleaseDC(None, hdc_screen);
    let _ = DeleteObject(icon_info.hbmColor);
    let _ = DeleteObject(icon_info.hbmMask);

    if lines == 0 { return None; }

    // BGRA to RGBA
    for i in (0..buffer.len()).step_by(4) {
        let b = buffer[i];
        let r = buffer[i + 2];
        buffer[i] = r;
        buffer[i + 2] = b;
    }

    let img = RgbaImage::from_raw(32, 32, buffer)?;
    let mut image_data = Vec::new();
    let mut cursor = Cursor::new(&mut image_data);
    img.write_to(&mut cursor, ImageFormat::Png).ok()?;

    Some(general_purpose::STANDARD.encode(image_data))
}
