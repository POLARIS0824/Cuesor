use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use windows::core::{HRESULT, PCWSTR};
use windows::Win32::Foundation::ERROR_NOT_SAME_DEVICE;
use windows::Win32::Storage::FileSystem::{
    MoveFileExW, MOVEFILE_COPY_ALLOWED, MOVEFILE_REPLACE_EXISTING, MOVEFILE_WRITE_THROUGH,
};

use crate::{
    model::{Color, Colors},
    platform::{wide, AppResult},
};

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Appearance {
    #[default]
    System,
    Light,
    Dark,
}

pub const SCHEME_NAMES: [&str; 6] = ["经典", "珊瑚", "暮紫", "极光", "柔粉", "暖砂"];

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ColorScheme {
    pub chinese_color: String,
    pub english_color: String,
}

// Cursor data selected from Radix Colors, Nord and Catppuccin. Attribution and
// the application-owned pairings are documented in docs/GUI_REDESIGN.md.
pub fn default_schemes() -> [ColorScheme; 6] {
    [
        ("#FF7800", "#0077FF"),
        ("#E54D2E", "#12A594"),
        ("#6E56CF", "#FFC53D"),
        ("#D08770", "#88C0D0"),
        ("#F38BA8", "#94E2D5"),
        ("#A18072", "#3E63DD"),
    ]
    .map(|(chinese, english)| ColorScheme {
        chinese_color: chinese.into(),
        english_color: english.into(),
    })
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct WindowSize {
    pub width: u32,
    pub height: u32,
}

impl Default for WindowSize {
    fn default() -> Self {
        Self {
            width: 880,
            height: 600,
        }
    }
}

impl WindowSize {
    pub const MIN: Self = Self {
        width: 760,
        height: 520,
    };

    pub fn load(directory: &Path) -> Self {
        std::fs::read(directory.join("window.json"))
            .ok()
            .and_then(|bytes| serde_json::from_slice::<Self>(&bytes).ok())
            .filter(|size| size.width >= Self::MIN.width && size.height >= Self::MIN.height)
            .unwrap_or_default()
    }

    pub fn constrained(self, available_width: u32, available_height: u32) -> Self {
        Self {
            width: self
                .width
                .clamp(Self::MIN.width, available_width.max(Self::MIN.width)),
            height: self
                .height
                .clamp(Self::MIN.height, available_height.max(Self::MIN.height)),
        }
    }

    pub fn save(&self, directory: &Path) -> AppResult<()> {
        write_json(&directory.join("window.json"), self)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct Settings {
    pub enabled: bool,
    pub chinese_color: String,
    pub english_color: String,
    pub auto_start: bool,
    pub appearance: Appearance,
    pub color_schemes: [ColorScheme; 6],
    pub active_scheme: usize,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            enabled: true,
            chinese_color: "#FF7800".into(),
            english_color: "#0077FF".into(),
            auto_start: false,
            appearance: Appearance::System,
            color_schemes: default_schemes(),
            active_scheme: 0,
        }
    }
}

impl Settings {
    pub fn select_scheme(&mut self, index: usize) {
        if let Some(scheme) = self.color_schemes.get(index) {
            self.active_scheme = index;
            self.chinese_color = scheme.chinese_color.clone();
            self.english_color = scheme.english_color.clone();
        }
    }

    pub fn customize_color(&mut self, chinese: bool, color: u32) {
        let color = format!("#{color:06X}");
        let scheme = &mut self.color_schemes[self.active_scheme];
        if chinese {
            self.chinese_color = color.clone();
            scheme.chinese_color = color;
        } else {
            self.english_color = color.clone();
            scheme.english_color = color;
        }
    }

    pub fn reset_active_scheme(&mut self) {
        self.color_schemes[self.active_scheme] = default_schemes()[self.active_scheme].clone();
        self.select_scheme(self.active_scheme);
    }

    pub fn colors(&self) -> AppResult<Colors> {
        Ok(Colors {
            chinese: Color::parse(&self.chinese_color)?,
            latin: Color::parse(&self.english_color)?,
        })
    }

    pub fn load(directory: &Path) -> AppResult<Self> {
        match std::fs::read(directory.join("settings.json")) {
            Ok(bytes) => {
                let json: serde_json::Value = serde_json::from_slice(&bytes)?;
                let legacy = json.get("color_schemes").is_none();
                let mut value: Self = serde_json::from_value(json)?;
                if legacy {
                    // Retain both existing cursor colors in the first editable slot.
                    value.active_scheme = 0;
                    value.color_schemes[0] = ColorScheme {
                        chinese_color: value.chinese_color.clone(),
                        english_color: value.english_color.clone(),
                    };
                }
                value.validate()?;
                Ok(value)
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(Self::default()),
            Err(error) => Err(error.into()),
        }
    }

    pub fn save(&self, directory: &Path) -> AppResult<()> {
        self.validate()?;
        write_json(&directory.join("settings.json"), self)
    }

    fn validate(&self) -> AppResult<()> {
        self.colors()?;
        if self.active_scheme >= self.color_schemes.len() {
            return Err("invalid active color scheme".into());
        }
        for scheme in &self.color_schemes {
            Color::parse(&scheme.chinese_color)?;
            Color::parse(&scheme.english_color)?;
        }
        Ok(())
    }
}

pub fn data_directory() -> AppResult<PathBuf> {
    if let Some(value) = std::env::var_os("TERMINAL_IME_DATA_DIR") {
        return Ok(PathBuf::from(value));
    }
    Ok(
        PathBuf::from(std::env::var_os("LOCALAPPDATA").ok_or("LOCALAPPDATA is unavailable")?)
            .join("TerminalIME"),
    )
}

pub fn write_json(path: &Path, value: &impl Serialize) -> AppResult<()> {
    let parent = path.parent().ok_or("settings path has no parent")?;
    std::fs::create_dir_all(parent)?;
    let temporary = path.with_extension(format!("{}.tmp", std::process::id()));
    std::fs::write(&temporary, serde_json::to_vec_pretty(value)?)?;
    let from = wide(temporary.as_os_str());
    let to = wide(path.as_os_str());
    let result = unsafe {
        MoveFileExW(
            PCWSTR(from.as_ptr()),
            PCWSTR(to.as_ptr()),
            MOVEFILE_REPLACE_EXISTING | MOVEFILE_WRITE_THROUGH,
        )
    };
    if let Err(error) = result {
        if error.code() != HRESULT::from_win32(ERROR_NOT_SAME_DEVICE.0) {
            return Err(error.into());
        }
        // Some profile filesystems report a cross-volume move even for sibling
        // paths. Keep the atomic rename on normal filesystems;
        // allow Windows to copy and delete only for this specific error.
        unsafe {
            MoveFileExW(
                PCWSTR(from.as_ptr()),
                PCWSTR(to.as_ptr()),
                MOVEFILE_REPLACE_EXISTING | MOVEFILE_WRITE_THROUGH | MOVEFILE_COPY_ALLOWED,
            )?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn persists_settings_and_rejects_invalid_colors_without_overwriting() {
        let dir =
            std::env::temp_dir().join(format!("terminal-ime-settings-test-{}", std::process::id()));
        let settings = Settings {
            chinese_color: "#AA0011".into(),
            appearance: Appearance::Dark,
            ..Default::default()
        };
        settings.save(&dir).unwrap();
        let next = Settings {
            enabled: false,
            ..settings.clone()
        };
        next.save(&dir).unwrap();
        assert_eq!(Settings::load(&dir).unwrap(), next);
        let invalid = Settings {
            english_color: "#bad\u{1b}".into(),
            ..next.clone()
        };
        assert!(invalid.save(&dir).is_err());
        assert_eq!(Settings::load(&dir).unwrap(), next);
        std::fs::remove_file(dir.join("settings.json")).unwrap();
        std::fs::remove_dir(dir).unwrap();
    }

    #[test]
    fn pre_redesign_settings_keep_colors_and_follow_system_appearance() {
        let dir =
            std::env::temp_dir().join(format!("terminal-ime-legacy-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("settings.json"),
            r##"{"enabled":false,"chinese_color":"#20A78B","english_color":"#0077FF","auto_start":true}"##,
        ).unwrap();
        let settings = Settings::load(&dir).unwrap();
        assert_eq!(settings.appearance, Appearance::System);
        assert_eq!(settings.chinese_color, "#20A78B");
        assert_eq!(settings.color_schemes[0].chinese_color, "#20A78B");
        assert_eq!(settings.color_schemes[0].english_color, "#0077FF");
        assert_eq!(settings.color_schemes[1], default_schemes()[1]);
        assert!(!settings.enabled);
        assert!(settings.auto_start);
        settings.save(&dir).unwrap();
        assert_eq!(Settings::load(&dir).unwrap(), settings);
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn all_six_schemes_keep_independent_colors_across_restart_and_reset() {
        let dir =
            std::env::temp_dir().join(format!("terminal-ime-schemes-test-{}", std::process::id()));
        let mut settings = Settings::default();
        for index in 0..6 {
            settings.select_scheme(index);
            settings.customize_color(true, 0x110022 + index as u32 * 0x10101);
            settings.customize_color(false, 0x005588 + index as u32 * 0x10101);
            settings.save(&dir).unwrap();
            assert_eq!(Settings::load(&dir).unwrap(), settings);
        }
        let custom = settings.color_schemes.clone();
        for (index, scheme) in custom.iter().enumerate() {
            settings.select_scheme(index);
            assert_eq!(settings.chinese_color, scheme.chinese_color);
            assert_eq!(settings.english_color, scheme.english_color);
        }
        settings.select_scheme(2);
        settings.reset_active_scheme();
        for (index, scheme) in custom.iter().enumerate() {
            let expected = if index == 2 {
                default_schemes()[2].clone()
            } else {
                scheme.clone()
            };
            assert_eq!(settings.color_schemes[index], expected);
        }
        settings.color_schemes[5].chinese_color = "invalid".into();
        assert!(settings.save(&dir).is_err());
        assert_eq!(Settings::load(&dir).unwrap().color_schemes, custom);
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn window_size_round_trips_and_restores_within_the_available_display() {
        let dir =
            std::env::temp_dir().join(format!("terminal-ime-window-test-{}", std::process::id()));
        assert_eq!(WindowSize::load(&dir), WindowSize::default());
        let resized = WindowSize {
            width: 1024,
            height: 720,
        };
        resized.save(&dir).unwrap();
        assert_eq!(WindowSize::load(&dir), resized);
        assert_eq!(
            WindowSize::load(&dir).constrained(900, 650),
            WindowSize {
                width: 900,
                height: 650
            }
        );
        std::fs::write(dir.join("window.json"), b"corrupt file").unwrap();
        assert_eq!(WindowSize::load(&dir), WindowSize::default());
        std::fs::remove_dir_all(dir).unwrap();
    }
}
