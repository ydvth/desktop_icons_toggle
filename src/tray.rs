use std::cell::Cell;
use std::mem::{size_of, zeroed};

use windows::core::{w, Error, HSTRING, Result, PCWSTR};
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, POINT, WPARAM};
use windows::Win32::UI::Input::KeyboardAndMouse::{RegisterHotKey, UnregisterHotKey, HOT_KEY_MODIFIERS, MOD_NOREPEAT};
use windows::Win32::UI::Shell::{
    Shell_NotifyIconW, NIF_ICON, NIF_MESSAGE, NIF_TIP, NIM_ADD, NIM_DELETE, NIM_MODIFY,
    NOTIFYICONDATAW,
};
use windows::Win32::UI::WindowsAndMessaging::*;

use crate::{autostart, config, desktop, settings_ui, util};

thread_local! {
    static TRAY_HWND: Cell<HWND> = Cell::new(HWND::default());
    static TASKBAR_MSG: Cell<u32> = Cell::new(0);
}

pub fn tray_hwnd() -> HWND {
    TRAY_HWND.with(|c| c.get())
}

/// Создаёт скрытое окно, иконку в трее и регистрирует горячую клавишу.
pub fn init() -> Result<()> {
    unsafe {
        let hinst = util::instance();

        let mut wc: WNDCLASSEXW = zeroed();
        wc.cbSize = size_of::<WNDCLASSEXW>() as u32;
        wc.lpfnWndProc = Some(tray_proc);
        wc.hInstance = hinst;
        wc.hIcon = load_app_icon();
        wc.hCursor = LoadCursorW(None, IDC_ARROW).unwrap_or_default();
        wc.lpszClassName = w!("DesktopIconsToggle.Tray");
        if RegisterClassExW(&wc) == 0 {
            return Err(Error::from_win32());
        }

        let hwnd = CreateWindowExW(
            WINDOW_EX_STYLE(0),
            w!("DesktopIconsToggle.Tray"),
            w!("Ярлыки рабочего стола"),
            WINDOW_STYLE(0),
            0,
            0,
            0,
            0,
            None,
            None,
            hinst,
            None,
        )?;
        TRAY_HWND.with(|c| c.set(hwnd));
        TASKBAR_MSG.with(|c| c.set(RegisterWindowMessageW(w!("TaskbarCreated"))));

        add_icon(hwnd)?;

        let cfg = config::get();
        if register_hotkey(cfg.hotkey_mods, cfg.hotkey_vk).is_err() {
            error_box("Горячая клавиша занята другой программой. Выберите другое сочетание в настройках.");
        }
    }
    Ok(())
}

/// Регистрирует глобальную горячую клавишу, предварительно снимая старую.
pub fn register_hotkey(mods: u32, vk: u32) -> Result<()> {
    unsafe {
        let hwnd = tray_hwnd();
        let _ = UnregisterHotKey(hwnd, util::HOTKEY_ID);
        RegisterHotKey(
            hwnd,
            util::HOTKEY_ID,
            HOT_KEY_MODIFIERS(mods | MOD_NOREPEAT.0),
            vk,
        )
    }
}

/// Переключает видимость ярлыков. Вызывается из меню трея и из горячей клавиши.
pub fn toggle_visibility() {
    match desktop::icons_visible() {
        Some(visible) => set_visibility(!visible),
        None => error_box("Рабочий стол не найден."),
    }
}

pub fn set_visibility(visible: bool) {
    if !desktop::set_icons_visible(visible) {
        error_box("Не удалось изменить видимость ярлыков.");
        return;
    }
    settings_ui::refresh_status();
}

/// Переключает автозапуск с Windows: пишет или удаляет запись в реестре и сохраняет настройку.
fn toggle_autostart() {
    let want = !config::get().autostart;
    if autostart::set_enabled(want).is_err() {
        error_box("Не удалось изменить автозапуск.");
        return;
    }
    if config::update(|cfg| cfg.autostart = want).is_err() {
        error_box("Не удалось сохранить настройки.");
    }
    settings_ui::refresh_all();
}

/// Окно с сообщением об ошибке. Используется только когда действие не выполнилось.
pub fn error_box(text: &str) {
    unsafe {
        let _ = MessageBoxW(
            None,
            &HSTRING::from(text),
            w!("Ярлыки рабочего стола"),
            MB_ICONWARNING,
        );
    }
}

unsafe fn add_icon(hwnd: HWND) -> Result<()> {
    let mut nid: NOTIFYICONDATAW = zeroed();
    nid.cbSize = size_of::<NOTIFYICONDATAW>() as u32;
    nid.hWnd = hwnd;
    nid.uID = util::TRAY_ICON_ID;
    nid.uFlags = NIF_ICON | NIF_MESSAGE | NIF_TIP;
    nid.uCallbackMessage = util::WM_TRAYICON;
    nid.hIcon = load_app_icon();
    util::copy_wide(&mut nid.szTip, util::APP_NAME);
    Shell_NotifyIconW(NIM_ADD, &nid).ok()
}

unsafe fn remove_icon(hwnd: HWND) {
    let mut nid: NOTIFYICONDATAW = zeroed();
    nid.cbSize = size_of::<NOTIFYICONDATAW>() as u32;
    nid.hWnd = hwnd;
    nid.uID = util::TRAY_ICON_ID;
    let _ = Shell_NotifyIconW(NIM_DELETE, &nid);
}

/// Иконка из ресурса с ID 1 (assets/app.ico через build.rs), иначе стандартная.
unsafe fn load_app_icon() -> HICON {
    LoadIconW(util::instance(), PCWSTR(1usize as *const u16))
        .or_else(|_| LoadIconW(None, IDI_APPLICATION))
        .unwrap_or_default()
}

unsafe fn show_menu(hwnd: HWND) {
    let Ok(menu) = CreatePopupMenu() else {
        return;
    };
    let visible = desktop::icons_visible().unwrap_or(true);
    let toggle_label = if visible {
        w!("Скрыть ярлыки")
    } else {
        w!("Показать ярлыки")
    };
    // Галочка рядом с пунктом автозапуска отражает текущее состояние.
    let autostart_flags = if config::get().autostart {
        MF_STRING | MF_CHECKED
    } else {
        MF_STRING
    };

    let _ = AppendMenuW(menu, MF_STRING, util::IDM_OPEN_SETTINGS, w!("Открыть настройки"));
    let _ = AppendMenuW(menu, MF_STRING, util::IDM_TOGGLE_ICONS, toggle_label);
    let _ = AppendMenuW(menu, autostart_flags, util::IDM_AUTOSTART, w!("Запускать с Windows"));
    let _ = AppendMenuW(menu, MF_SEPARATOR, 0, PCWSTR::null());
    let _ = AppendMenuW(menu, MF_STRING, util::IDM_EXIT, w!("Выход"));

    let mut pt = POINT::default();
    let _ = GetCursorPos(&mut pt);
    // Без SetForegroundWindow меню не закрывается при клике мимо него.
    let _ = SetForegroundWindow(hwnd);
    let _ = TrackPopupMenu(menu, TPM_RIGHTBUTTON, pt.x, pt.y, 0, hwnd, None);
    let _ = PostMessageW(hwnd, WM_NULL, WPARAM(0), LPARAM(0));
    let _ = DestroyMenu(menu);
}

unsafe extern "system" fn tray_proc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    // Explorer перезапущен: иконку нужно добавить заново.
    let taskbar = TASKBAR_MSG.with(|c| c.get());
    if msg == taskbar && taskbar != 0 {
        let _ = add_icon(hwnd);
        return LRESULT(0);
    }

    match msg {
        util::WM_TRAYICON => {
            let event = ((lparam.0 as usize) & 0xFFFF) as u32;
            match event {
                WM_LBUTTONDBLCLK => settings_ui::open(),
                WM_RBUTTONUP => show_menu(hwnd),
                _ => {}
            }
            LRESULT(0)
        }
        WM_COMMAND => {
            match util::loword(wparam.0) {
                util::IDM_OPEN_SETTINGS => settings_ui::open(),
                util::IDM_TOGGLE_ICONS => toggle_visibility(),
                util::IDM_AUTOSTART => toggle_autostart(),
                util::IDM_EXIT => {
                    let _ = DestroyWindow(hwnd);
                }
                _ => {}
            }
            LRESULT(0)
        }
        WM_HOTKEY => {
            // RegisterHotKey срабатывает глобально, поэтому проверяем активное окно.
            if wparam.0 == util::HOTKEY_ID as usize && desktop::is_desktop_foreground() {
                toggle_visibility();
            }
            LRESULT(0)
        }
        util::WM_ALREADY_RUNNING => {
            settings_ui::open();
            LRESULT(0)
        }
        WM_DESTROY => {
            remove_icon(hwnd);
            let _ = UnregisterHotKey(hwnd, util::HOTKEY_ID);
            PostQuitMessage(0);
            LRESULT(0)
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}
