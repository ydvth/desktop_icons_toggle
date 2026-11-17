use std::cell::RefCell;
use std::fs;
use std::io;
use std::path::PathBuf;

use windows::Win32::UI::Input::KeyboardAndMouse::{MOD_CONTROL, MOD_SHIFT};

#[derive(Clone, Copy, Debug)]
pub struct Config {
    pub autostart: bool,
    /// Модификаторы в формате MOD_* (MOD_ALT=1, MOD_CONTROL=2, MOD_SHIFT=4).
    pub hotkey_mods: u32,
    /// Код виртуальной клавиши (по умолчанию VK_F1 = 0x70).
    pub hotkey_vk: u32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            autostart: false,
            hotkey_mods: MOD_CONTROL.0 | MOD_SHIFT.0,
            hotkey_vk: 0x70,
        }
    }
}

thread_local! {
    static CONFIG: RefCell<Config> = RefCell::new(Config::load());
}

pub fn get() -> Config {
    CONFIG.with(|c| *c.borrow())
}

pub fn update(change: impl FnOnce(&mut Config)) -> io::Result<()> {
    CONFIG.with(|c| {
        let mut cfg = c.borrow_mut();
        change(&mut *cfg);
        cfg.save()
    })
}

fn path() -> PathBuf {
    let base = std::env::var_os("APPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));
    base.join("DesktopIconsToggle").join("settings.ini")
}

impl Config {
    fn load() -> Self {
        let mut cfg = Config::default();
        let Ok(text) = fs::read_to_string(path()) else {
            return cfg;
        };
        for line in text.lines() {
            let Some((key, value)) = line.split_once('=') else {
                continue;
            };
            let value = value.trim();
            match key.trim() {
                "autostart" => cfg.autostart = value == "1",
                "hotkey_mods" => {
                    if let Ok(v) = value.parse::<u32>() {
                        cfg.hotkey_mods = v;
                    }
                }
                "hotkey_vk" => {
                    if let Ok(v) = value.parse::<u32>() {
                        cfg.hotkey_vk = v;
                    }
                }
                _ => {}
            }
        }
        cfg
    }

    fn save(&self) -> io::Result<()> {
        let file = path();
        if let Some(dir) = file.parent() {
            fs::create_dir_all(dir)?;
        }
        let text = format!(
            "autostart={}\nhotkey_mods={}\nhotkey_vk={}\n",
            self.autostart as u8,
            self.hotkey_mods,
            self.hotkey_vk
        );
        fs::write(file, text)
    }
}
