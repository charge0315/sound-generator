use base64::{engine::general_purpose, Engine as _};
use image::{ImageBuffer, RgbaImage};
use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use std::ptr;
use windows::{
    core::PCWSTR,
    Win32::{
        Foundation::HWND,
        Graphics::Gdi::{
            CreateCompatibleDC, DeleteDC, DeleteObject, GetDC, GetDIBits, ReleaseDC, BITMAPINFO,
            BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS,
        },
        Storage::FileSystem::FILE_FLAGS_AND_ATTRIBUTES,
        UI::Shell::{SHGetFileInfoW, SHFILEINFOW, SHGFI_ICON, SHGFI_LARGEICON},
        UI::WindowsAndMessaging::{DestroyIcon, GetIconInfo, HICON},
    },
};

/// 実行ファイルのパスからアプリアイコンを抽出し、Base64文字列で返す
pub fn extract_icon_base64(executable_path: &str) -> Option<String> {
    // パスをワイド文字列 (UTF-16) に変換
    let mut path_wide: Vec<u16> = OsStr::new(executable_path).encode_wide().collect();
    path_wide.push(0); // Null terminator

    let mut shfi: SHFILEINFOW = unsafe { std::mem::zeroed() };
    let result = unsafe {
        SHGetFileInfoW(
            PCWSTR(path_wide.as_ptr()),
            FILE_FLAGS_AND_ATTRIBUTES(0),
            Some(&mut shfi),
            std::mem::size_of::<SHFILEINFOW>() as u32,
            SHGFI_ICON | SHGFI_LARGEICON,
        )
    };

    if result == 0 || shfi.hIcon.is_invalid() {
        return None;
    }

    let hicon = shfi.hIcon;
    let base64_img = hicon_to_base64_png(hicon);

    // アイコンのハンドルを解放
    unsafe {
        let _ = DestroyIcon(hicon);
    }

    base64_img
}

/// 実行ファイルのバージョン情報から ProductName を抽出する
pub fn extract_product_name(executable_path: &str) -> Option<String> {
    let mut path_wide: Vec<u16> = OsStr::new(executable_path).encode_wide().collect();
    path_wide.push(0); // Null terminator

    let size = unsafe {
        windows::Win32::Storage::FileSystem::GetFileVersionInfoSizeW(
            PCWSTR(path_wide.as_ptr()),
            Some(ptr::null_mut()),
        )
    };
    if size == 0 {
        return None;
    }

    let mut data = vec![0u8; size as usize];
    if unsafe {
        windows::Win32::Storage::FileSystem::GetFileVersionInfoW(
            PCWSTR(path_wide.as_ptr()),
            0,
            size,
            data.as_mut_ptr() as _,
        )
    }
    .is_err()
    {
        return None;
    }

    unsafe {
        // 対象言語のコードページを取得 (\VarFileInfo\Translation)
        let mut len = 0;
        let mut trans_ptr: *mut std::ffi::c_void = ptr::null_mut();

        let trans_query: Vec<u16> = OsStr::new("\\VarFileInfo\\Translation")
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();
        if windows::Win32::Storage::FileSystem::VerQueryValueW(
            data.as_ptr() as _,
            PCWSTR(trans_query.as_ptr()),
            &mut trans_ptr,
            &mut len,
        )
        .as_bool()
            && len >= 4
        {
            let translation = *(trans_ptr as *const u32);
            let lang_id = (translation & 0xFFFF) as u16;
            let code_page = (translation >> 16) as u16;

            let sub_block = format!(
                "\\StringFileInfo\\{:04x}{:04x}\\FileDescription",
                lang_id, code_page
            );
            let sub_block_w: Vec<u16> = OsStr::new(&sub_block)
                .encode_wide()
                .chain(std::iter::once(0))
                .collect();

            let mut val_ptr: *mut std::ffi::c_void = ptr::null_mut();
            let mut val_len = 0;
            if windows::Win32::Storage::FileSystem::VerQueryValueW(
                data.as_ptr() as _,
                PCWSTR(sub_block_w.as_ptr()),
                &mut val_ptr,
                &mut val_len,
            )
            .as_bool()
                && val_len > 0
            {
                let chars =
                    std::slice::from_raw_parts(val_ptr as *const u16, (val_len - 1) as usize);
                if let Ok(name) = String::from_utf16(chars) {
                    return Some(name);
                }
            }
        }
    }
    None
}

/// HICON を PNG ベースの Base64 文字列に変換する
fn hicon_to_base64_png(hicon: HICON) -> Option<String> {
    unsafe {
        let mut icon_info = std::mem::zeroed();
        if GetIconInfo(hicon, &mut icon_info).is_err() {
            return None;
        }

        let mut width = 0;
        let mut height = 0;
        let mut pixels = Vec::new();

        // 常に32ビットのDIBとしてビットマップを取得する
        let hdc_screen = GetDC(HWND(ptr::null_mut()));
        let hdc_mem = CreateCompatibleDC(hdc_screen);

        let mut bmi: BITMAPINFO = std::mem::zeroed();
        bmi.bmiHeader.biSize = std::mem::size_of::<BITMAPINFOHEADER>() as u32;

        // まずサイズの取得
        if GetDIBits(
            hdc_mem,
            icon_info.hbmColor,
            0,
            0,
            None,
            &mut bmi,
            DIB_RGB_COLORS,
        ) != 0
        {
            width = bmi.bmiHeader.biWidth as u32;
            height = bmi.bmiHeader.biHeight.abs() as u32;

            // 下から上へのDIBに対処するため高さを負に設定
            bmi.bmiHeader.biHeight = -(height as i32);
            bmi.bmiHeader.biPlanes = 1;
            bmi.bmiHeader.biBitCount = 32;
            bmi.bmiHeader.biCompression = BI_RGB.0 as u32;

            let buffer_size = (width * height * 4) as usize;
            pixels.resize(buffer_size, 0u8);

            GetDIBits(
                hdc_mem,
                icon_info.hbmColor,
                0,
                height,
                Some(pixels.as_mut_ptr() as *mut _),
                &mut bmi,
                DIB_RGB_COLORS,
            );
        }

        // リソースの解放
        let _ = DeleteDC(hdc_mem);
        let _ = ReleaseDC(HWND(ptr::null_mut()), hdc_screen);
        if !icon_info.hbmColor.is_invalid() {
            let _ = DeleteObject(icon_info.hbmColor);
        }
        if !icon_info.hbmMask.is_invalid() {
            let _ = DeleteObject(icon_info.hbmMask);
        }

        if width == 0 || height == 0 || pixels.is_empty() {
            return None;
        }

        // BGRA から RGBA への変換
        let mut rgba_image: RgbaImage = ImageBuffer::new(width, height);
        for y in 0..height {
            for x in 0..width {
                let i = ((y * width + x) * 4) as usize;
                let b = pixels[i];
                let g = pixels[i + 1];
                let r = pixels[i + 2];
                let a = pixels[i + 3];
                // 透明度が正しく含まれていない場合は補正するなどの追加処理が必要かもしれないが
                // ここでは標準的な32bbpとして処理する
                rgba_image.put_pixel(x, y, image::Rgba([r, g, b, a]));
            }
        }

        // メモリ上でPNGとしてエンコード
        let mut png_bytes: Vec<u8> = Vec::new();
        let mut cursor = std::io::Cursor::new(&mut png_bytes);
        if let Err(_) = rgba_image.write_to(&mut cursor, image::ImageFormat::Png) {
            return None;
        }

        // Base64に変換
        let b64 = general_purpose::STANDARD.encode(&png_bytes);
        Some(b64)
    }
}
