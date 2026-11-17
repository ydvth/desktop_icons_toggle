use windows::core::{w, PCWSTR};
use windows::Win32::Foundation::{BOOL, HWND, LPARAM};
use windows::Win32::UI::WindowsAndMessaging::{
    EnumWindows, FindWindowExW, FindWindowW, GetForegroundWindow, IsWindowVisible, ShowWindow, SW_HIDE, SW_SHOW,
};

use crate::util::class_name;

/// Находит окно SHELLDLL_DefView рабочего стола.
/// На большинстве систем оно лежит в Progman; при включённом слайд-шоу обоев
/// оно переезжает в одно из окон WorkerW.
fn defview() -> Option<HWND> {
    unsafe {
        if let Ok(progman) = FindWindowW(w!("Progman"), PCWSTR::null()) {
            if let Ok(dv) = FindWindowExW(progman, None, w!("SHELLDLL_DefView"), PCWSTR::null()) {
                return Some(dv);
            }
        }

        let mut found = HWND::default();
        let _ = EnumWindows(Some(enum_workers), LPARAM(&mut found as *mut HWND as isize));
        if found.0.is_null() {
            None
        } else {
            Some(found)
        }
    }
}

unsafe extern "system" fn enum_workers(hwnd: HWND, lparam: LPARAM) -> BOOL {
    if let Ok(dv) = FindWindowExW(hwnd, None, w!("SHELLDLL_DefView"), PCWSTR::null()) {
        *(lparam.0 as *mut HWND) = dv;
        return BOOL(0);
    }
    BOOL(1)
}

/// Список значков рабочего стола (SysListView32).
fn listview() -> Option<HWND> {
    let dv = defview()?;
    unsafe { FindWindowExW(dv, None, w!("SysListView32"), PCWSTR::null()).ok() }
}

/// Some(true) — значки видны, Some(false) — скрыты, None — рабочий стол не найден.
pub fn icons_visible() -> Option<bool> {
    let lv = listview()?;
    Some(unsafe { IsWindowVisible(lv) }.as_bool())
}

/// Скрывает или показывает значки. Ярлыки не удаляются и не перемещаются.
pub fn set_icons_visible(visible: bool) -> bool {
    match listview() {
        Some(lv) => {
            unsafe {
                let _ = ShowWindow(lv, if visible { SW_SHOW } else { SW_HIDE });
            }
            true
        }
        None => false,
    }
}

/// Истина только если активное окно — сам рабочий стол (Progman или WorkerW с DefView).
/// Окна Проводника (CabinetWClass), программы, панель задач и свёрнутые окна дают false.
pub fn is_desktop_foreground() -> bool {
    unsafe {
        let fg = GetForegroundWindow();
        if fg.0.is_null() {
            return false;
        }
        let cls = class_name(fg);
        if cls != "Progman" && cls != "WorkerW" {
            return false;
        }
        FindWindowExW(fg, None, w!("SHELLDLL_DefView"), PCWSTR::null()).is_ok()
    }
}
