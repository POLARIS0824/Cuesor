//! Internationalization (i18n) support for Cuesor core engine.

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Locale {
    #[default]
    ZhCn,
    EnUs,
}

impl Locale {
    pub fn current() -> Self {
        if let Ok(val) = std::env::var("CUESOR_LANG") {
            if val.eq_ignore_ascii_case("en") || val.eq_ignore_ascii_case("en-us") {
                return Self::EnUs;
            }
            if val.eq_ignore_ascii_case("zh") || val.eq_ignore_ascii_case("zh-cn") {
                return Self::ZhCn;
            }
        }
        Self::ZhCn
    }
}

pub const SCHEME_NAMES_ZH: [&str; 6] = ["经典", "珊瑚", "暮紫", "极光", "柔粉", "暖砂"];
pub const SCHEME_NAMES_EN: [&str; 6] = ["Classic", "Coral", "Dusk", "Aurora", "Rose", "Sand"];

pub mod messages {
    use super::Locale;

    pub fn scheme_names(locale: Locale) -> [&'static str; 6] {
        match locale {
            Locale::ZhCn => super::SCHEME_NAMES_ZH,
            Locale::EnUs => super::SCHEME_NAMES_EN,
        }
    }

    pub fn engine_missing(locale: Locale) -> &'static str {
        match locale {
            Locale::ZhCn => "同目录下缺少 cuesor-engine.exe，请检查安装完整性。",
            Locale::EnUs => "Missing cuesor-engine.exe in the same directory. Please verify installation integrity.",
        }
    }

    pub fn engine_helper_cli(locale: Locale) -> &'static str {
        match locale {
            Locale::ZhCn => "Cuesor 后台组件，请打开同目录的 Cuesor.exe。",
            Locale::EnUs => "Cuesor background component. Please launch Cuesor.exe in the same directory.",
        }
    }

    pub fn obsidian_service_failed(locale: Locale, error: &impl std::fmt::Display) -> String {
        match locale {
            Locale::ZhCn => format!("Obsidian 联动服务启动失败：{error}"),
            Locale::EnUs => format!("Failed to start Obsidian bridge service: {error}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn messages_render_non_empty_for_all_locales() {
        for locale in [Locale::ZhCn, Locale::EnUs] {
            assert!(!messages::engine_missing(locale).is_empty());
            assert!(!messages::engine_helper_cli(locale).is_empty());
            assert!(!messages::obsidian_service_failed(locale, &"test error").is_empty());
            let names = messages::scheme_names(locale);
            assert_eq!(names.len(), 6);
            for name in names {
                assert!(!name.is_empty());
            }
        }
    }
}
