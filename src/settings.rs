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

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct Settings {
    pub enabled: bool,
    pub chinese_color: String,
    pub english_color: String,
    pub auto_start: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            enabled: true,
            chinese_color: "#FF7800".into(),
            english_color: "#0077FF".into(),
            auto_start: false,
        }
    }
}

impl Settings {
    pub fn colors(&self) -> AppResult<Colors> {
        Ok(Colors {
            chinese: Color::parse(&self.chinese_color)?,
            latin: Color::parse(&self.english_color)?,
        })
    }

    pub fn load(directory: &Path) -> AppResult<Self> {
        match std::fs::read(directory.join("settings.json")) {
            Ok(bytes) => {
                let value: Self = serde_json::from_slice(&bytes)?;
                value.colors()?;
                Ok(value)
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(Self::default()),
            Err(error) => Err(error.into()),
        }
    }

    pub fn save(&self, directory: &Path) -> AppResult<()> {
        self.colors()?;
        write_json(&directory.join("settings.json"), self)
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
}
