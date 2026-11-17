use std::ffi::OsStr;
use std::iter::once;
use std::os::windows::ffi::OsStrExt;

use windows::core::{w, Error, Result};
use windows::Win32::Foundation::ERROR_FILE_NOT_FOUND;
use windows::Win32::System::Registry::{
    RegCloseKey, RegDeleteValueW, RegOpenKeyExW, RegSetValueExW, HKEY, HKEY_CURRENT_USER, KEY_SET_VALUE, REG_SZ,
};

/// Включает или выключает запуск вместе с Windows (HKCU\...\Run).
pub fn set_enabled(enabled: bool) -> Result<()> {
    unsafe {
        let mut key = HKEY::default();
        RegOpenKeyExW(
            HKEY_CURRENT_USER,
            w!("Software\\Microsoft\\Windows\\CurrentVersion\\Run"),
            0,
            KEY_SET_VALUE,
            &mut key,
        )
        .ok()?;

        let result = if enabled { write_value(key) } else { remove_value(key) };
        let _ = RegCloseKey(key);
        result
    }
}

unsafe fn write_value(key: HKEY) -> Result<()> {
    let exe = std::env::current_exe().map_err(|_| Error::from_win32())?;
    let cmd = format!("\"{}\"", exe.display());
    let wide: Vec<u16> = OsStr::new(&cmd).encode_wide().chain(once(0)).collect();
    let bytes = std::slice::from_raw_parts(wide.as_ptr().cast::<u8>(), wide.len() * 2);
    RegSetValueExW(key, w!("DesktopIconsToggle"), 0, REG_SZ, Some(bytes)).ok()
}

unsafe fn remove_value(key: HKEY) -> Result<()> {
    let err = RegDeleteValueW(key, w!("DesktopIconsToggle"));
    if err == ERROR_FILE_NOT_FOUND {
        Ok(())
    } else {
        err.ok()
    }
}
