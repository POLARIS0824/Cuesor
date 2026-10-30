use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use windows::core::{HRESULT, PCWSTR};
use windows::Win32::Foundation::{ERROR_ACCESS_DENIED, ERROR_NOT_SAME_DEVICE};
use windows::Win32::Storage::FileSystem::{
    MoveFileExW, ReplaceFileW, MOVEFILE_COPY_ALLOWED, MOVEFILE_REPLACE_EXISTING,
    MOVEFILE_WRITE_THROUGH, REPLACE_FILE_FLAGS,
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

pub const SCHEME_NAMES: [&str; 6] = crate::i18n::SCHEME_NAMES_ZH;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ColorScheme {
    pub chinese_color: String,
    pub english_color: String,
}

pub fn default_schemes() -> [ColorScheme; 6] {
    [
        ("#F76B15", "#0090FF"),
        ("#E54D2E", "#12A594"),
        ("#6E56CF", "#FFC53D"),
        ("#30A46C", "#00A2C7"),
        ("#D6409F", "#86EAD4"),
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
    pub auto_start: bool,
    pub run_as_admin: bool,
    pub obsidian_enabled: bool,
    pub appearance: Appearance,
    pub color_schemes: [ColorScheme; 6],
    pub active_scheme: usize,
}

impl Default for Settings {
    fn default() -> Self {
        let color_schemes = default_schemes();
        Self {
            enabled: true,
            auto_start: false,
            run_as_admin: false,
            obsidian_enabled: true,
            appearance: Appearance::System,
            color_schemes,
            active_scheme: 0,
        }
    }
}

impl Settings {
    pub fn select_scheme(&mut self, index: usize) {
        if index < self.color_schemes.len() {
            self.active_scheme = index;
        }
    }

    pub fn active_colors(&self) -> &ColorScheme {
        &self.color_schemes[self.active_scheme]
    }

    pub fn customize_color(&mut self, chinese: bool, color: u32) {
        let color = format!("#{color:06X}");
        let scheme = &mut self.color_schemes[self.active_scheme];
        if chinese {
            scheme.chinese_color = color;
        } else {
            scheme.english_color = color;
        }
    }

    pub fn reset_active_scheme(&mut self) {
        self.color_schemes[self.active_scheme] = default_schemes()[self.active_scheme].clone();
    }

    pub fn colors(&self) -> AppResult<Colors> {
        let scheme = self.active_colors();
        Ok(Colors {
            chinese: Color::parse(&scheme.chinese_color)?,
            latin: Color::parse(&scheme.english_color)?,
        })
    }

    pub fn load(directory: &Path) -> AppResult<Self> {
        match std::fs::read(directory.join("settings.json")) {
            Ok(bytes) => {
                let json: serde_json::Value = serde_json::from_slice(&bytes)?;
                let legacy = json.get("color_schemes").is_none();
                let chinese = json
                    .get("chinese_color")
                    .cloned()
                    .map(serde_json::from_value::<String>)
                    .transpose()?;
                let english = json
                    .get("english_color")
                    .cloned()
                    .map(serde_json::from_value::<String>)
                    .transpose()?;
                let mut value: Self = serde_json::from_value(json)?;
                if legacy {
                    value.active_scheme = 0;
                }
                if value.active_scheme >= value.color_schemes.len() {
                    return Err("invalid active color scheme".into());
                }
                let scheme = &mut value.color_schemes[value.active_scheme];
                if let Some(color) = chinese {
                    scheme.chinese_color = color;
                }
                if let Some(color) = english {
                    scheme.english_color = color;
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
    if let Some(value) = std::env::var_os("CUESOR_DATA_DIR") {
        return Ok(PathBuf::from(value));
    }
    Ok(
        PathBuf::from(std::env::var_os("LOCALAPPDATA").ok_or("LOCALAPPDATA is unavailable")?)
            .join("Cuesor"),
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
        if error.code() == HRESULT::from_win32(ERROR_ACCESS_DENIED.0) && path.exists() {
            let backup = path.with_extension(format!("{}.bak", std::process::id()));
            let backup_path = wide(backup.as_os_str());
            unsafe {
                ReplaceFileW(
                    PCWSTR(to.as_ptr()),
                    PCWSTR(from.as_ptr()),
                    PCWSTR(backup_path.as_ptr()),
                    REPLACE_FILE_FLAGS(0),
                    None,
                    None,
                )?;
            }
            let _ = std::fs::remove_file(backup);
            return Ok(());
        }
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
    use std::io::Read;
    use std::os::windows::fs::OpenOptionsExt;
    use windows::Win32::Storage::FileSystem::{
        FILE_SHARE_DELETE, FILE_SHARE_READ, FILE_SHARE_WRITE,
    };

    #[test]
    fn persists_settings_and_rejects_invalid_colors_without_overwriting() {
        let dir = std::env::temp_dir().join(format!("cuesor-settings-test-{}", std::process::id()));
        let mut settings = Settings {
            appearance: Appearance::Dark,
            ..Default::default()
        };
        settings.customize_color(true, 0xAA0011);
        settings.save(&dir).unwrap();
        let next = Settings {
            enabled: false,
            ..settings.clone()
        };
        next.save(&dir).unwrap();
        assert_eq!(Settings::load(&dir).unwrap(), next);
        let mut invalid = next.clone();
        invalid.color_schemes[0].english_color = "#bad\u{1b}".into();
        assert!(invalid.save(&dir).is_err());
        assert_eq!(Settings::load(&dir).unwrap(), next);
        std::fs::remove_file(dir.join("settings.json")).unwrap();
        std::fs::remove_dir(dir).unwrap();
    }

    #[test]
    fn saves_new_settings_while_a_reader_holds_the_previous_snapshot() {
        let dir =
            std::env::temp_dir().join(format!("cuesor-open-reader-test-{}", std::process::id()));
        let original = Settings::default();
        original.save(&dir).unwrap();
        let path = dir.join("settings.json");
        let mut reader = std::fs::OpenOptions::new()
            .read(true)
            .share_mode(FILE_SHARE_READ.0 | FILE_SHARE_WRITE.0 | FILE_SHARE_DELETE.0)
            .open(&path)
            .unwrap();

        let updated = Settings {
            enabled: false,
            appearance: Appearance::Dark,
            ..original.clone()
        };
        updated.save(&dir).unwrap();

        let mut previous_bytes = Vec::new();
        reader.read_to_end(&mut previous_bytes).unwrap();
        assert_eq!(
            serde_json::from_slice::<Settings>(&previous_bytes).unwrap(),
            original
        );
        assert_eq!(Settings::load(&dir).unwrap(), updated);
        assert_eq!(
            std::fs::read_dir(&dir)
                .unwrap()
                .map(|entry| entry.unwrap().file_name())
                .collect::<Vec<_>>(),
            vec![std::ffi::OsString::from("settings.json")]
        );

        drop(reader);
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn pre_redesign_settings_keep_colors_and_follow_system_appearance() {
        let dir = std::env::temp_dir().join(format!("cuesor-legacy-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("settings.json"),
            r##"{"enabled":false,"chinese_color":"#20A78B","english_color":"#0077FF","auto_start":true}"##,
        ).unwrap();
        let settings = Settings::load(&dir).unwrap();
        assert_eq!(settings.appearance, Appearance::System);
        assert_eq!(settings.active_colors().chinese_color, "#20A78B");
        assert_eq!(settings.color_schemes[0].chinese_color, "#20A78B");
        assert_eq!(settings.color_schemes[0].english_color, "#0077FF");
        assert_eq!(settings.color_schemes[1], default_schemes()[1]);
        assert!(!settings.enabled);
        assert!(settings.auto_start);
        assert!(!settings.run_as_admin);
        assert!(settings.obsidian_enabled);
        settings.save(&dir).unwrap();
        assert_eq!(Settings::load(&dir).unwrap(), settings);
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn all_six_schemes_keep_independent_colors_across_restart_and_reset() {
        let dir = std::env::temp_dir().join(format!("cuesor-schemes-test-{}", std::process::id()));
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
            assert_eq!(settings.active_colors(), scheme);
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
    fn imports_effective_legacy_colors_once_and_saves_only_schemes() {
        let dir =
            std::env::temp_dir().join(format!("cuesor-migration-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let mut original = Settings::default();
        original.select_scheme(3);
        let mut json = serde_json::to_value(&original).unwrap();
        json["chinese_color"] = "#123456".into();
        json["english_color"] = "#ABCDEF".into();
        std::fs::write(
            dir.join("settings.json"),
            serde_json::to_vec(&json).unwrap(),
        )
        .unwrap();

        let settings = Settings::load(&dir).unwrap();
        assert_eq!(settings.active_scheme, 3);
        assert_eq!(settings.colors().unwrap().chinese, Color(0x123456));
        assert_eq!(settings.colors().unwrap().latin, Color(0xABCDEF));
        for index in [0, 1, 2, 4, 5] {
            assert_eq!(settings.color_schemes[index], original.color_schemes[index]);
        }
        settings.save(&dir).unwrap();
        let saved = std::fs::read(dir.join("settings.json")).unwrap();
        let canonical: serde_json::Value = serde_json::from_slice(&saved).unwrap();
        assert!(canonical.get("chinese_color").is_none());
        assert!(canonical.get("english_color").is_none());
        let reloaded = Settings::load(&dir).unwrap();
        assert_eq!(reloaded, settings);
        reloaded.save(&dir).unwrap();
        assert_eq!(std::fs::read(dir.join("settings.json")).unwrap(), saved);
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn partial_legacy_pair_keeps_the_other_selected_color() {
        let dir = std::env::temp_dir().join(format!("cuesor-partial-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let mut original = Settings::default();
        original.select_scheme(4);
        let mut json = serde_json::to_value(&original).unwrap();
        json["chinese_color"] = "#123456".into();
        std::fs::write(
            dir.join("settings.json"),
            serde_json::to_vec(&json).unwrap(),
        )
        .unwrap();
        let settings = Settings::load(&dir).unwrap();
        assert_eq!(settings.active_colors().chinese_color, "#123456");
        assert_eq!(
            settings.active_colors().english_color,
            original.active_colors().english_color
        );
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn rejects_invalid_migration_data_without_overwriting_the_file() {
        let dir = std::env::temp_dir().join(format!("cuesor-invalid-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let original = serde_json::to_value(Settings::default()).unwrap();
        let mut bad_index = original.clone();
        bad_index["active_scheme"] = 6.into();
        let mut bad_legacy = original.clone();
        bad_legacy["chinese_color"] = "invalid".into();
        let mut bad_scheme = original.clone();
        bad_scheme["color_schemes"][5]["english_color"] = "invalid".into();
        let mut bad_length = original;
        bad_length["color_schemes"] = serde_json::json!([]);
        for json in [bad_index, bad_legacy, bad_scheme, bad_length] {
            let bytes = serde_json::to_vec(&json).unwrap();
            std::fs::write(dir.join("settings.json"), &bytes).unwrap();
            assert!(Settings::load(&dir).is_err());
            assert_eq!(std::fs::read(dir.join("settings.json")).unwrap(), bytes);
        }
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn window_size_round_trips_and_restores_within_the_available_display() {
        let dir = std::env::temp_dir().join(format!("cuesor-window-test-{}", std::process::id()));
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

    #[test]
    fn obsidian_and_admin_settings_persist_and_default_properly() {
        let dir = std::env::temp_dir().join(format!("cuesor-obsidian-admin-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();

        let default_settings = Settings::load(&dir).unwrap();
        assert!(!default_settings.run_as_admin);
        assert!(default_settings.obsidian_enabled);

        let mut custom = default_settings.clone();
        custom.run_as_admin = true;
        custom.obsidian_enabled = false;
        custom.save(&dir).unwrap();

        let reloaded = Settings::load(&dir).unwrap();
        assert!(reloaded.run_as_admin);
        assert!(!reloaded.obsidian_enabled);

        std::fs::remove_dir_all(dir).unwrap();
    }
}
