use windows::core::PCWSTR;
use windows::Win32::Foundation::{HINSTANCE, HWND};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::WindowsAndMessaging::{GetClassNameW, WM_APP};

pub const APP_NAME: &str = "Ярлыки рабочего стола";

pub const WM_TRAYICON: u32 = WM_APP + 1;
pub const WM_ALREADY_RUNNING: u32 = WM_APP + 2;

pub const HOTKEY_ID: i32 = 1;
pub const TRAY_ICON_ID: u32 = 1;

pub const IDM_OPEN_SETTINGS: usize = 1001;
pub const IDM_TOGGLE_ICONS: usize = 1002;
pub const IDM_EXIT: usize = 1003;
pub const IDM_AUTOSTART: usize = 1004;

pub const BN_CLICKED: u32 = 0;
pub const BM_GETCHECK: u32 = 0x00F0;
pub const BM_SETCHECK: u32 = 0x00F1;

const WM_USER: u32 = 0x0400;
pub const HKM_SETHOTKEY: u32 = WM_USER + 1;
pub const HKM_GETHOTKEY: u32 = WM_USER + 2;

pub const HOTKEYF_SHIFT: u32 = 0x01;
pub const HOTKEYF_CONTROL: u32 = 0x02;
pub const HOTKEYF_ALT: u32 = 0x04;

pub fn instance() -> HINSTANCE {
    let module = unsafe { GetModuleHandleW(PCWSTR::null()) }.unwrap_or_default();
    HINSTANCE(module.0)
}

pub fn loword(value: usize) -> usize {
    value & 0xFFFF
}

pub fn class_name(hwnd: HWND) -> String {
    let mut buf = [0u16; 256];
    let len = unsafe { GetClassNameW(hwnd, &mut buf) };
    String::from_utf16_lossy(&buf[..len.max(0) as usize])
}

/// Копирует строку в фиксированный буфер WinAPI с завершающим нулём.
pub fn copy_wide(dst: &mut [u16], src: &str) {
    let units: Vec<u16> = src.encode_utf16().collect();
    let n = units.len().min(dst.len().saturating_sub(1));
    dst[..n].copy_from_slice(&units[..n]);
    dst[n] = 0;
}
