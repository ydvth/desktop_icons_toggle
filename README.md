Desktop Icons Toggle is a lightweight Windows tray utility that hides and shows your desktop icons with a single hotkey, Ctrl+Shift+F1. Icons are never deleted or moved. The program only toggles the visibility of the desktop icon list, the same way Windows does through the desktop context menu, so your layout is preserved exactly as it was.

The hotkey works only when the desktop itself is the active window. In a browser, a game, File Explorer, or any other application, the shortcut does nothing, so it will not interfere with normal work.

The app runs quietly in the system tray. Double-click the icon to open the settings window, where you can change the hotkey, enable start with Windows, and see whether the icons are currently visible. The tray menu gives quick access to hiding or showing icons, toggling autostart, and exiting.

It is written in Rust and uses only the Win32 API through the windows crate, without Electron, .NET, or third-party tray libraries. This keeps memory use and CPU load low, and the executable needs no installation. It targets Windows 10 and Windows 11 on x64.
