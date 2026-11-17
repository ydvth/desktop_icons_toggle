#![windows_subsystem = "windows"]

mod autostart;
mod config;
mod desktop;
mod settings_ui;
mod tray;
mod util;

use windows::core::{w, Result, HSTRING, PCWSTR};
use windows::Win32::Foundation::{CloseHandle, GetLastError, ERROR_ALREADY_EXISTS, HANDLE, LPARAM, WPARAM};
use windows::Win32::System::Threading::CreateMutexW;
use windows::Win32::UI::WindowsAndMessaging::{
    DispatchMessageW, FindWindowW, GetMessageW, MessageBoxW, PostMessageW, TranslateMessage, MB_ICONERROR, MSG,
};

fn main() {
    // Mutex держим в переменной до конца работы процесса.
    let Some(_mutex) = acquire_single_instance() else {
        return;
    };
    if let Err(err) = run() {
        unsafe {
            MessageBoxW(
                None,
                &HSTRING::from(format!("Ошибка запуска: {err}")),
                w!("Ярлыки рабочего стола"),
                MB_ICONERROR,
            );
        }
    }
}

/// Возвращает дескриптор мьютекса, если это первый экземпляр.
/// Иначе сообщает уже запущенному экземпляру показать уведомление и возвращает None.
fn acquire_single_instance() -> Option<HANDLE> {
    unsafe {
        let mutex = CreateMutexW(None, false, w!("Local\\DesktopIconsToggle.SingleInstance")).ok()?;
        if GetLastError() == ERROR_ALREADY_EXISTS {
            if let Ok(tray) = FindWindowW(w!("DesktopIconsToggle.Tray"), PCWSTR::null()) {
                let _ = PostMessageW(tray, util::WM_ALREADY_RUNNING, WPARAM(0), LPARAM(0));
            }
            let _ = CloseHandle(mutex);
            return None;
        }
        Some(mutex)
    }
}

fn run() -> Result<()> {
    settings_ui::init();
    tray::init()?;

    let cfg = config::get();
    if cfg.autostart {
        // Обновляем путь в реестре на случай перемещения exe-файла.
        let _ = autostart::set_enabled(true);
    }

    unsafe {
        let mut msg: MSG = std::mem::zeroed();
        while GetMessageW(&mut msg, None, 0, 0).0 > 0 {
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }
    Ok(())
}
