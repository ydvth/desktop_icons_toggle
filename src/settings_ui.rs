use std::cell::Cell;
use std::ffi::c_void;
use std::mem::{size_of, zeroed};

use windows::core::{w, HSTRING, PCWSTR};
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::Graphics::Gdi::{GetStockObject, COLOR_WINDOW, DEFAULT_GUI_FONT, HBRUSH};
use windows::Win32::UI::Controls::{InitCommonControlsEx, ICC_HOTKEY_CLASS, INITCOMMONCONTROLSEX};
use windows::Win32::UI::Input::KeyboardAndMouse::{MOD_ALT, MOD_CONTROL, MOD_SHIFT};
use windows::Win32::UI::WindowsAndMessaging::*;

use crate::config::{self, Config};
use crate::{autostart, desktop, tray, util};

const ID_AUTOSTART: i32 = 201;
const ID_HOTKEY: i32 = 203;
const ID_APPLY_HOTKEY: i32 = 204;
const ID_HIDE: i32 = 205;
const ID_SHOW: i32 = 206;
const ID_STATUS: i32 = 207;
const SS_LEFT: i32 = 0;

#[derive(Clone, Copy, Default)]
struct Ctrls {
    autostart: HWND,
    hotkey: HWND,
    status: HWND,
}

thread_local! {
    static SETTINGS_HWND: Cell<HWND> = Cell::new(HWND::default());
    static CTRLS: Cell<Ctrls> = Cell::new(Ctrls::default());
}

/// Регистрирует класс общих контролов, нужный для элемента горячей клавиши.
pub fn init() {
    unsafe {
        let icc = INITCOMMONCONTROLSEX {
            dwSize: size_of::<INITCOMMONCONTROLSEX>() as u32,
            dwICC: ICC_HOTKEY_CLASS,
        };
        let _ = InitCommonControlsEx(&icc);
    }
}

/// Открывает окно настроек или показывает уже открытое.
pub fn open() {
    unsafe {
        let existing = SETTINGS_HWND.with(|c| c.get());
        if !existing.0.is_null() {
            let _ = ShowWindow(existing, SW_SHOW);
            let _ = SetForegroundWindow(existing);
            refresh_ui();
            return;
        }

        let hinst = util::instance();
        let mut wc: WNDCLASSEXW = zeroed();
        wc.cbSize = size_of::<WNDCLASSEXW>() as u32;
        wc.lpfnWndProc = Some(settings_proc);
        wc.hInstance = hinst;
        wc.hCursor = LoadCursorW(None, IDC_ARROW).unwrap_or_default();
        wc.hbrBackground = HBRUSH((COLOR_WINDOW.0 + 1) as usize as *mut c_void);
        wc.lpszClassName = w!("DesktopIconsToggle.Settings");
        RegisterClassExW(&wc);

        let hwnd = match CreateWindowExW(
            WINDOW_EX_STYLE(0),
            w!("DesktopIconsToggle.Settings"),
            w!("Настройки ярлыков"),
            WS_OVERLAPPED | WS_CAPTION | WS_SYSMENU,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            360,
            260,
            None,
            None,
            hinst,
            None,
        ) {
            Ok(h) => h,
            Err(_) => {
                tray::error_box("Не удалось открыть окно настроек.");
                return;
            }
        };
        SETTINGS_HWND.with(|c| c.set(hwnd));
        let _ = ShowWindow(hwnd, SW_SHOW);
        let _ = SetForegroundWindow(hwnd);
    }
}

/// Обновляет строку состояния, если окно настроек создано.
pub fn refresh_status() {
    let ctrls = CTRLS.with(|c| c.get());
    if !ctrls.status.0.is_null() {
        unsafe { update_status(ctrls.status) }
    }
}

/// Синхронизирует все элементы окна настроек с текущей конфигурацией (если окно открыто).
pub fn refresh_all() {
    if !SETTINGS_HWND.with(|c| c.get()).0.is_null() {
        unsafe { refresh_ui() }
    }
}

unsafe extern "system" fn settings_proc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    match msg {
        WM_CREATE => {
            create_controls(hwnd);
            refresh_ui();
            LRESULT(0)
        }
        WM_COMMAND => {
            let notify_code = ((wparam.0 >> 16) & 0xFFFF) as u32;
            if notify_code == util::BN_CLICKED {
                on_command(hwnd, util::loword(wparam.0) as i32);
            }
            LRESULT(0)
        }
        WM_CLOSE => {
            // Окно скрывается, а не уничтожается: состояние остаётся в памяти.
            let _ = ShowWindow(hwnd, SW_HIDE);
            LRESULT(0)
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

unsafe fn create_controls(parent: HWND) {
    let autostart = child(parent, w!("BUTTON"), w!("Запускать вместе с Windows"), BS_AUTOCHECKBOX, 16, 14, 320, 22, ID_AUTOSTART);
    child(parent, w!("STATIC"), w!("Горячая клавиша:"), SS_LEFT, 16, 50, 130, 22, 0);
    let hotkey = child(parent, w!("msctls_hotkey32"), w!(""), 0, 150, 46, 186, 24, ID_HOTKEY);
    child(parent, w!("BUTTON"), w!("Применить сочетание"), BS_PUSHBUTTON, 150, 76, 186, 26, ID_APPLY_HOTKEY);
    let status = child(parent, w!("STATIC"), w!(""), SS_LEFT, 16, 120, 320, 22, ID_STATUS);
    child(parent, w!("BUTTON"), w!("Скрыть ярлыки"), BS_PUSHBUTTON, 16, 156, 156, 32, ID_HIDE);
    child(parent, w!("BUTTON"), w!("Показать ярлыки"), BS_PUSHBUTTON, 180, 156, 156, 32, ID_SHOW);

    CTRLS.with(|c| c.set(Ctrls { autostart, hotkey, status }));
}

#[allow(clippy::too_many_arguments)]
unsafe fn child(
    parent: HWND,
    class: PCWSTR,
    text: PCWSTR,
    style: i32,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    id: i32,
) -> HWND {
    let hinst = util::instance();
    let hwnd = CreateWindowExW(
        WINDOW_EX_STYLE(0),
        class,
        text,
        WS_CHILD | WS_VISIBLE | WS_TABSTOP | WINDOW_STYLE(style as u32),
        x,
        y,
        width,
        height,
        parent,
        HMENU(id as usize as *mut c_void),
        hinst,
        None,
    )
    .unwrap_or_default();
    let font = GetStockObject(DEFAULT_GUI_FONT);
    SendMessageW(hwnd, WM_SETFONT, WPARAM(font.0 as usize), LPARAM(1));
    hwnd
}

/// Переносит текущие настройки в элементы окна.
unsafe fn refresh_ui() {
    let cfg = config::get();
    let c = CTRLS.with(|c| c.get());
    set_check(c.autostart, cfg.autostart);
    let packed = pack_hotkey(cfg.hotkey_mods, cfg.hotkey_vk);
    SendMessageW(c.hotkey, util::HKM_SETHOTKEY, WPARAM(packed as usize), LPARAM(0));
    if !c.status.0.is_null() {
        update_status(c.status);
    }
}

unsafe fn update_status(status: HWND) {
    let text = match desktop::icons_visible() {
        Some(true) => "Состояние ярлыков: показаны",
        Some(false) => "Состояние ярлыков: скрыты",
        None => "Рабочий стол не найден",
    };
    let _ = SetWindowTextW(status, &HSTRING::from(text));
}

unsafe fn on_command(hwnd: HWND, id: i32) {
    let c = CTRLS.with(|c| c.get());
    match id {
        ID_AUTOSTART => {
            let on = is_checked(c.autostart);
            if autostart::set_enabled(on).is_err() {
                set_check(c.autostart, !on);
                error(hwnd, "Не удалось изменить автозапуск.");
                return;
            }
            save(hwnd, |cfg| cfg.autostart = on);
        }
        ID_APPLY_HOTKEY => apply_hotkey(hwnd, c),
        ID_HIDE => tray::set_visibility(false),
        ID_SHOW => tray::set_visibility(true),
        _ => {}
    }
}

unsafe fn apply_hotkey(hwnd: HWND, c: Ctrls) {
    let packed = SendMessageW(c.hotkey, util::HKM_GETHOTKEY, WPARAM(0), LPARAM(0)).0 as u32;
    let vk = packed & 0xFF;
    let mods = unpack_mods(packed);

    if vk == 0 || mods == 0 {
        error(hwnd, "Сочетание должно содержать Ctrl, Shift или Alt и клавишу.");
        return;
    }

    if tray::register_hotkey(mods, vk).is_err() {
        // Возвращаем прежнее сочетание, чтобы не остаться без горячей клавиши.
        let old = config::get();
        let _ = tray::register_hotkey(old.hotkey_mods, old.hotkey_vk);
        let old_packed = pack_hotkey(old.hotkey_mods, old.hotkey_vk);
        SendMessageW(c.hotkey, util::HKM_SETHOTKEY, WPARAM(old_packed as usize), LPARAM(0));
        error(hwnd, "Это сочетание уже занято другой программой.");
        return;
    }

    save(hwnd, |cfg| {
        cfg.hotkey_mods = mods;
        cfg.hotkey_vk = vk;
    });
}

fn save(hwnd: HWND, change: impl FnOnce(&mut Config)) {
    if config::update(change).is_err() {
        unsafe { error(hwnd, "Не удалось сохранить настройки в файл.") }
    }
}

unsafe fn error(hwnd: HWND, text: &str) {
    MessageBoxW(
        hwnd,
        &HSTRING::from(text),
        w!("Ярлыки рабочего стола"),
        MB_ICONWARNING,
    );
}

unsafe fn set_check(ctrl: HWND, on: bool) {
    SendMessageW(ctrl, util::BM_SETCHECK, WPARAM(on as usize), LPARAM(0));
}

unsafe fn is_checked(ctrl: HWND) -> bool {
    SendMessageW(ctrl, util::BM_GETCHECK, WPARAM(0), LPARAM(0)).0 == 1
}

/// Формат элемента hotkey: младший байт — vk, старший — флаги HOTKEYF_*.
fn pack_hotkey(mods: u32, vk: u32) -> u32 {
    let mut flags = 0;
    if (mods & MOD_SHIFT.0) != 0 {
        flags |= util::HOTKEYF_SHIFT;
    }
    if (mods & MOD_CONTROL.0) != 0 {
        flags |= util::HOTKEYF_CONTROL;
    }
    if (mods & MOD_ALT.0) != 0 {
        flags |= util::HOTKEYF_ALT;
    }
    (vk & 0xFF) | (flags << 8)
}

fn unpack_mods(packed: u32) -> u32 {
    let flags = (packed >> 8) & 0xFF;
    let mut mods = 0;
    if (flags & util::HOTKEYF_SHIFT) != 0 {
        mods |= MOD_SHIFT.0;
    }
    if (flags & util::HOTKEYF_CONTROL) != 0 {
        mods |= MOD_CONTROL.0;
    }
    if (flags & util::HOTKEYF_ALT) != 0 {
        mods |= MOD_ALT.0;
    }
    mods
}
