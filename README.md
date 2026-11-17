# Desktop Icons Toggle

A lightweight Windows tray utility that hides and shows desktop icons with a global hotkey. Icons are never deleted or moved: the utility toggles the visibility of the desktop icon list, the same way Windows does through the desktop context menu.

## Features

- Hide or show all desktop icons with **Ctrl + Shift + F1** (configurable).
- The hotkey works only when the desktop itself is the active window. In browsers, games, File Explorer windows, or the taskbar it does nothing.
- Runs in the system tray. Double-click the icon to open settings; right-click for the menu.
- Optional start with Windows, toggled from the tray menu or the settings window.
- Settings are saved between runs.
- Single instance: launching the program a second time opens the settings window of the running copy.
- Written in Rust using only the Win32 API through the `windows` crate. No Electron, no .NET, no Python.

## Requirements

- Windows 10 or Windows 11, x64
- Rust toolchain with the MSVC target:

```
rustup default stable-x86_64-pc-windows-msvc
```

## Build

```
cargo build --release
```

The executable is written to `target\release\desktop_icons_toggle.exe`. It needs no installation.

To run without building a release binary:

```
cargo run
```

## Custom icon (optional)

Place an icon at `assets\app.ico` (recommended sizes: 16, 32, 48, 256 px) and rebuild. `build.rs` embeds it into the executable and the tray. Without the file, the default Windows application icon is used.

## Usage

| Action | Result |
| --- | --- |
| Ctrl + Shift + F1 (desktop active) | Hide or show desktop icons |
| Double-click tray icon | Open settings |
| Right-click tray icon | Menu: open settings, hide/show icons, start with Windows, exit |

The settings window contains:

- Start with Windows
- Hotkey selector and "Apply hotkey" button
- Current icon state
- Hide icons / Show icons buttons

## Configuration files

- Settings: `%APPDATA%\DesktopIconsToggle\settings.ini`
- Autostart entry: `HKCU\Software\Microsoft\Windows\CurrentVersion\Run`, value name `DesktopIconsToggle`

## How it works

- The desktop icon list is found through `FindWindowW` and `FindWindowExW` inside `Progman` or `WorkerW` (`SHELLDLL_DefView` > `SysListView32`), and its visibility is changed with `ShowWindow`.
- The active window is checked with `GetForegroundWindow` and `GetClassNameW`, not by mouse position.
- The global hotkey is registered with `RegisterHotKey`.
- The tray icon is created with `Shell_NotifyIconW`, without a third-party tray library.
- Single instance is enforced with a named mutex (`CreateMutexW`).

## Project layout

```
src/main.rs          entry point, single instance, message loop
src/tray.rs          tray icon, menu, hotkey handling
src/desktop.rs       desktop window lookup and icon visibility
src/settings_ui.rs   settings window
src/config.rs        loading and saving settings
src/autostart.rs     "start with Windows" registry entry
src/util.rs          constants and helpers
build.rs             embeds the icon when assets\app.ico exists
```

## Notes

- Explorer restarts recreate the desktop icon window, so the icon state may reset after an Explorer restart.
- Some hotkey combinations are already used by other programs. If registration fails, a message appears and another combination can be chosen in settings.
