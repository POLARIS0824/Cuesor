//! Internationalization (i18n) module for Cuesor Desktop UI and native shell integration.

use cuesor::i18n::Locale;
use windows::core::{w, PCWSTR};

// --- Native & Tray Strings ---

pub fn tray_tooltip(locale: Locale) -> &'static str {
    match locale {
        Locale::ZhCn => "Cuesor 双击打开设置，右键打开菜单",
        Locale::EnUs => "Cuesor: Double-click to open settings, right-click for menu",
    }
}

pub fn tray_error_icon(locale: Locale) -> &'static str {
    match locale {
        Locale::ZhCn => "无法创建托盘图标",
        Locale::EnUs => "Failed to create tray icon",
    }
}

pub fn error_elevation_failed(locale: Locale) -> &'static str {
    match locale {
        Locale::ZhCn => "未切换到管理员模式（可能取消了授权）",
        Locale::EnUs => "Failed to switch to administrator mode (permission may have been denied)",
    }
}

pub fn tray_menu_open_settings(locale: Locale) -> PCWSTR {
    match locale {
        Locale::ZhCn => w!("打开设置"),
        Locale::EnUs => w!("Open Settings"),
    }
}

pub fn tray_menu_pause(locale: Locale) -> PCWSTR {
    match locale {
        Locale::ZhCn => w!("暂停指示"),
        Locale::EnUs => w!("Pause Indication"),
    }
}

pub fn tray_menu_resume(locale: Locale) -> PCWSTR {
    match locale {
        Locale::ZhCn => w!("恢复指示"),
        Locale::EnUs => w!("Resume Indication"),
    }
}

pub fn tray_menu_exit(locale: Locale) -> PCWSTR {
    match locale {
        Locale::ZhCn => w!("退出 Cuesor"),
        Locale::EnUs => w!("Exit Cuesor"),
    }
}

// --- Window & Notices ---

pub fn window_title(locale: Locale) -> &'static str {
    match locale {
        Locale::ZhCn => "Cuesor 设置",
        Locale::EnUs => "Cuesor Settings",
    }
}

pub fn notice_window_size_save_failed(locale: Locale, error: &impl std::fmt::Display) -> String {
    match locale {
        Locale::ZhCn => format!("窗口尺寸保存失败：{error}"),
        Locale::EnUs => format!("Failed to save window size: {error}"),
    }
}

pub fn notice_controller_stopped(locale: Locale) -> &'static str {
    match locale {
        Locale::ZhCn => "后台已停止，请退出后重新打开；可查看运行记录排查。",
        Locale::EnUs => "Background service has stopped. Please exit and reopen Cuesor; inspect logs to troubleshoot.",
    }
}

pub fn notice_auto_saved(locale: Locale) -> &'static str {
    match locale {
        Locale::ZhCn => "设置自动保存",
        Locale::EnUs => "Settings saved automatically",
    }
}

pub fn notice_saved(locale: Locale) -> &'static str {
    match locale {
        Locale::ZhCn => "设置已保存",
        Locale::EnUs => "Settings saved",
    }
}

pub fn notice_save_failed(locale: Locale, error: &impl std::fmt::Display) -> String {
    match locale {
        Locale::ZhCn => format!("保存失败：{error}"),
        Locale::EnUs => format!("Failed to save settings: {error}"),
    }
}

pub fn notice_elevation_prompt(locale: Locale) -> &'static str {
    match locale {
        Locale::ZhCn => "请处理 Windows 的管理员授权提示。",
        Locale::EnUs => "Please respond to the Windows administrator prompt.",
    }
}

// --- Appearance Controls ---

pub fn appearance_system(locale: Locale) -> &'static str {
    match locale {
        Locale::ZhCn => "系统",
        Locale::EnUs => "System",
    }
}

pub fn appearance_light(locale: Locale) -> &'static str {
    match locale {
        Locale::ZhCn => "浅色",
        Locale::EnUs => "Light",
    }
}

pub fn appearance_dark(locale: Locale) -> &'static str {
    match locale {
        Locale::ZhCn => "深色",
        Locale::EnUs => "Dark",
    }
}

pub fn accessibility_appearance(locale: Locale, label: &str) -> String {
    match locale {
        Locale::ZhCn => format!("外观：{label}"),
        Locale::EnUs => format!("Appearance: {label}"),
    }
}

// --- Color Editor ---

pub fn color_editor_chinese_label(locale: Locale) -> &'static str {
    match locale {
        Locale::ZhCn => "中文输入",
        Locale::EnUs => "Chinese input",
    }
}

pub fn color_editor_chinese_sample(locale: Locale) -> &'static str {
    match locale {
        Locale::ZhCn => "输入一些文字",
        Locale::EnUs => "Type some text",
    }
}

pub fn color_editor_english_label(locale: Locale) -> &'static str {
    match locale {
        Locale::ZhCn => "英文输入",
        Locale::EnUs => "English input",
    }
}

pub fn color_editor_english_sample(locale: Locale) -> &'static str {
    match locale {
        Locale::ZhCn => "echo hello",
        Locale::EnUs => "echo hello",
    }
}

pub fn color_custom_button(locale: Locale) -> &'static str {
    match locale {
        Locale::ZhCn => "自定义颜色…",
        Locale::EnUs => "Custom color…",
    }
}

pub fn accessibility_custom_color(locale: Locale, label: &str) -> String {
    match locale {
        Locale::ZhCn => format!("自定义{label}光标颜色"),
        Locale::EnUs => format!("Customize {label} cursor color"),
    }
}

// --- Scheme Selector ---

pub fn scheme_selector_title(locale: Locale) -> &'static str {
    match locale {
        Locale::ZhCn => "配色方案",
        Locale::EnUs => "Color Schemes",
    }
}

pub fn scheme_selector_hint(locale: Locale) -> &'static str {
    match locale {
        Locale::ZhCn => "左侧为中文，右侧为英文",
        Locale::EnUs => "Left: Chinese, Right: English",
    }
}

pub fn scheme_reset_button(locale: Locale) -> &'static str {
    match locale {
        Locale::ZhCn => "重置当前配色",
        Locale::EnUs => "Reset current scheme",
    }
}

pub fn accessibility_scheme(locale: Locale, index: usize, name: &str) -> String {
    match locale {
        Locale::ZhCn => format!("配色{}：{}", index, name),
        Locale::EnUs => format!("Scheme {}: {}", index, name),
    }
}

// --- Status & Alerts ---

pub fn alert_stopped_title(locale: Locale) -> &'static str {
    match locale {
        Locale::ZhCn => "后台连接已停止",
        Locale::EnUs => "Background connection stopped",
    }
}

pub fn alert_stopped_detail(locale: Locale) -> &'static str {
    match locale {
        Locale::ZhCn => "请从系统托盘退出，再重新打开 Cuesor。",
        Locale::EnUs => "Please exit from system tray and restart Cuesor.",
    }
}

pub fn alert_admin_title(locale: Locale) -> &'static str {
    match locale {
        Locale::ZhCn => "管理员终端需要授权",
        Locale::EnUs => "Administrator terminal requires elevation",
    }
}

pub fn alert_admin_detail(locale: Locale) -> &'static str {
    match locale {
        Locale::ZhCn => "以管理员身份重新启动后即可接入。",
        Locale::EnUs => "Restart as administrator to connect.",
    }
}

pub fn alert_partial_title(locale: Locale) -> &'static str {
    match locale {
        Locale::ZhCn => "部分会话未能接入",
        Locale::EnUs => "Some sessions failed to connect",
    }
}

pub fn alert_partial_detail(locale: Locale) -> &'static str {
    match locale {
        Locale::ZhCn => "请检查终端权限，或退出后重新打开 Cuesor。",
        Locale::EnUs => "Please check terminal permissions, or restart Cuesor.",
    }
}

pub fn alert_paused_title(locale: Locale) -> &'static str {
    match locale {
        Locale::ZhCn => "光标指示已暂停",
        Locale::EnUs => "Cursor indication paused",
    }
}

pub fn alert_waiting_title(locale: Locale) -> &'static str {
    match locale {
        Locale::ZhCn => "等待 Windows Terminal",
        Locale::EnUs => "Waiting for Windows Terminal",
    }
}

pub fn alert_waiting_detail(locale: Locale) -> &'static str {
    match locale {
        Locale::ZhCn => "打开 Windows Terminal 即可接入",
        Locale::EnUs => "Open Windows Terminal to connect",
    }
}

pub fn alert_connected_title(locale: Locale, count: usize) -> String {
    match locale {
        Locale::ZhCn => format!("已接入 {count} 个终端会话"),
        Locale::EnUs => format!("{count} terminal session(s) connected"),
    }
}

pub fn shell_cmd_display(locale: Locale) -> &'static str {
    match locale {
        Locale::ZhCn => "命令提示符",
        Locale::EnUs => "Command Prompt",
    }
}

pub fn more_sessions_connected(locale: Locale, remaining: usize) -> String {
    match locale {
        Locale::ZhCn => format!("另有 {remaining} 个会话已接入"),
        Locale::EnUs => format!("{remaining} more session(s) connected"),
    }
}

pub fn btn_requesting_elevation(locale: Locale) -> &'static str {
    match locale {
        Locale::ZhCn => "正在请求授权…",
        Locale::EnUs => "Requesting elevation…",
    }
}

pub fn btn_restart_elevated(locale: Locale) -> &'static str {
    match locale {
        Locale::ZhCn => "以管理员身份重新启动",
        Locale::EnUs => "Restart as Administrator",
    }
}

// --- Main Settings Sections & Toggles ---

pub fn section_indicator_title(locale: Locale) -> &'static str {
    match locale {
        Locale::ZhCn => "光标颜色指示",
        Locale::EnUs => "Cursor Color Indication",
    }
}

pub fn switch_enabled(locale: Locale) -> &'static str {
    match locale {
        Locale::ZhCn => "已启用",
        Locale::EnUs => "Enabled",
    }
}

pub fn switch_paused(locale: Locale) -> &'static str {
    match locale {
        Locale::ZhCn => "已暂停",
        Locale::EnUs => "Paused",
    }
}

pub fn accessibility_indicator_switch(locale: Locale) -> &'static str {
    match locale {
        Locale::ZhCn => "启用光标颜色指示",
        Locale::EnUs => "Enable cursor color indication",
    }
}

pub fn section_autostart_title(locale: Locale) -> &'static str {
    match locale {
        Locale::ZhCn => "登录 Windows 时启动",
        Locale::EnUs => "Launch on Windows startup",
    }
}

pub fn accessibility_autostart_switch(locale: Locale) -> &'static str {
    match locale {
        Locale::ZhCn => "登录 Windows 时启动",
        Locale::EnUs => "Launch on Windows startup",
    }
}

pub fn section_obsidian_title(locale: Locale) -> &'static str {
    match locale {
        Locale::ZhCn => "Obsidian 联动",
        Locale::EnUs => "Obsidian Integration",
    }
}

pub fn status_obsidian_disabled(locale: Locale) -> &'static str {
    match locale {
        Locale::ZhCn => "已停用",
        Locale::EnUs => "Disabled",
    }
}

pub fn status_obsidian_connected(locale: Locale) -> &'static str {
    match locale {
        Locale::ZhCn => "已连接",
        Locale::EnUs => "Connected",
    }
}

pub fn status_obsidian_listening(locale: Locale, port: u16) -> String {
    match locale {
        Locale::ZhCn => format!("监听中 (端口 {port})"),
        Locale::EnUs => format!("Listening (port {port})"),
    }
}

pub fn section_obsidian_desc(locale: Locale) -> &'static str {
    match locale {
        Locale::ZhCn => "通过本地 WebSocket 向 Obsidian 实时同步 Win32 原生输入法状态",
        Locale::EnUs => "Synchronize Win32 IME status to Obsidian in real time via local WebSocket",
    }
}

pub fn accessibility_obsidian_switch(locale: Locale) -> &'static str {
    match locale {
        Locale::ZhCn => "启用 Obsidian 联动",
        Locale::EnUs => "Enable Obsidian integration",
    }
}

pub fn section_admin_title(locale: Locale) -> &'static str {
    match locale {
        Locale::ZhCn => "默认以管理员身份启动",
        Locale::EnUs => "Run as Administrator by default",
    }
}

pub fn section_admin_desc(locale: Locale) -> &'static str {
    match locale {
        Locale::ZhCn => "开机自启或直接运行时申请管理员权限，提升对管理员终端与高权限窗口的穿透检测",
        Locale::EnUs => "Request administrator privileges at startup to improve detection across elevated terminals and windows",
    }
}

pub fn accessibility_admin_switch(locale: Locale) -> &'static str {
    match locale {
        Locale::ZhCn => "默认以管理员身份启动",
        Locale::EnUs => "Run as Administrator by default",
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn desktop_ui_strings_render_non_empty_for_all_locales() {
        for locale in [Locale::ZhCn, Locale::EnUs] {
            assert!(!window_title(locale).is_empty());
            assert!(!tray_tooltip(locale).is_empty());
            assert!(!tray_error_icon(locale).is_empty());
            assert!(!error_elevation_failed(locale).is_empty());

            assert!(!notice_window_size_save_failed(locale, &"err").is_empty());
            assert!(!notice_controller_stopped(locale).is_empty());
            assert!(!notice_auto_saved(locale).is_empty());
            assert!(!notice_saved(locale).is_empty());
            assert!(!notice_save_failed(locale, &"err").is_empty());
            assert!(!notice_elevation_prompt(locale).is_empty());

            assert!(!appearance_system(locale).is_empty());
            assert!(!appearance_light(locale).is_empty());
            assert!(!appearance_dark(locale).is_empty());
            assert!(!accessibility_appearance(locale, "test").is_empty());

            assert!(!color_editor_chinese_label(locale).is_empty());
            assert!(!color_editor_chinese_sample(locale).is_empty());
            assert!(!color_editor_english_label(locale).is_empty());
            assert!(!color_editor_english_sample(locale).is_empty());
            assert!(!color_custom_button(locale).is_empty());
            assert!(!accessibility_custom_color(locale, "test").is_empty());

            assert!(!scheme_selector_title(locale).is_empty());
            assert!(!scheme_selector_hint(locale).is_empty());
            assert!(!scheme_reset_button(locale).is_empty());
            assert!(!accessibility_scheme(locale, 1, "test").is_empty());

            assert!(!alert_stopped_title(locale).is_empty());
            assert!(!alert_stopped_detail(locale).is_empty());
            assert!(!alert_admin_title(locale).is_empty());
            assert!(!alert_admin_detail(locale).is_empty());
            assert!(!alert_partial_title(locale).is_empty());
            assert!(!alert_partial_detail(locale).is_empty());
            assert!(!alert_paused_title(locale).is_empty());
            assert!(!alert_waiting_title(locale).is_empty());
            assert!(!alert_waiting_detail(locale).is_empty());
            assert!(!alert_connected_title(locale, 2).is_empty());
            assert!(!shell_cmd_display(locale).is_empty());
            assert!(!more_sessions_connected(locale, 1).is_empty());
            assert!(!btn_requesting_elevation(locale).is_empty());
            assert!(!btn_restart_elevated(locale).is_empty());

            assert!(!section_indicator_title(locale).is_empty());
            assert!(!switch_enabled(locale).is_empty());
            assert!(!switch_paused(locale).is_empty());
            assert!(!accessibility_indicator_switch(locale).is_empty());

            assert!(!section_autostart_title(locale).is_empty());
            assert!(!accessibility_autostart_switch(locale).is_empty());

            assert!(!section_obsidian_title(locale).is_empty());
            assert!(!status_obsidian_disabled(locale).is_empty());
            assert!(!status_obsidian_connected(locale).is_empty());
            assert!(!status_obsidian_listening(locale, 23333).is_empty());
            assert!(!section_obsidian_desc(locale).is_empty());
            assert!(!accessibility_obsidian_switch(locale).is_empty());

            assert!(!section_admin_title(locale).is_empty());
            assert!(!section_admin_desc(locale).is_empty());
            assert!(!accessibility_admin_switch(locale).is_empty());
        }
    }
}
