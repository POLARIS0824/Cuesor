use super::{native, native_hwnd, SettingsView};
use gpui::{prelude::*, *};
use gpui_kit::{
    assets::IconName,
    component::{
        button::{Button, ButtonGroup, ButtonVariants},
        scroll::ScrollableElement,
        switch::Switch,
        ActiveTheme, Disableable, Icon, Selectable, Sizable, Theme, ThemeMode,
    },
};
use terminal_ime::{
    model::Color,
    settings::{default_schemes, Appearance, SCHEME_NAMES},
};

// Kit's default asset source embeds its component icons, not the full catalog.
gpui_kit::assets::icon_assets!(
    CursorIcons,
    [
        Terminal,
        Monitor,
        Pause,
        ShieldAlert,
        CircleAlert,
        CircleCheck,
        RotateCcw
    ]
);

pub(super) struct AppAssets;

impl AssetSource for AppAssets {
    fn load(&self, path: &str) -> Result<Option<std::borrow::Cow<'static, [u8]>>> {
        match CursorIcons.load(path)? {
            Some(bytes) => Ok(Some(bytes)),
            None => gpui_kit::assets::Assets.load(path),
        }
    }

    fn list(&self, path: &str) -> Result<Vec<SharedString>> {
        let mut paths = gpui_kit::assets::Assets.list(path)?;
        paths.extend(CursorIcons.list(path)?);
        Ok(paths)
    }
}

impl SettingsView {
    pub(super) fn apply_appearance(&self, window: &mut Window, cx: &mut Context<Self>) {
        match self.options.appearance {
            Appearance::System => Theme::sync_system_appearance(Some(window), cx),
            Appearance::Light => Theme::change(ThemeMode::Light, Some(window), cx),
            Appearance::Dark => Theme::change(ThemeMode::Dark, Some(window), cx),
        }
    }

    fn appearance_control(&self, cx: &mut Context<Self>) -> impl IntoElement {
        ButtonGroup::new("appearance")
            .small()
            .outline()
            .children(
                [
                    ("系统", IconName::Monitor, Appearance::System),
                    ("浅色", IconName::Sun, Appearance::Light),
                    ("深色", IconName::Moon, Appearance::Dark),
                ]
                .into_iter()
                .enumerate()
                .map(|(index, (label, icon, mode))| {
                    Button::new(("appearance", index))
                        .icon(icon)
                        .label(label)
                        .accessibility_label(format!("外观：{label}"))
                        .selected(self.options.appearance == mode)
                        .toggled(self.options.appearance == mode)
                }),
            )
            .on_click(cx.listener(|view, indices: &Vec<usize>, window, cx| {
                let mut next = view.options.clone();
                next.appearance = match indices.first() {
                    Some(1) => Appearance::Light,
                    Some(2) => Appearance::Dark,
                    _ => Appearance::System,
                };
                view.persist(next, cx);
                view.apply_appearance(window, cx);
            }))
    }

    fn color_editor(&self, chinese: bool, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme().clone();
        let (label, sample, value) = if chinese {
            ("中文输入", "输入一些文字", &self.options.chinese_color)
        } else {
            ("英文输入", "echo hello", &self.options.english_color)
        };
        let color = Color::parse(value).expect("validated settings color").0;
        div()
            .flex_1()
            .min_w_0()
            .flex()
            .flex_col()
            .gap_2()
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .child(div().font_weight(FontWeight::MEDIUM).child(label))
                    .child(
                        div()
                            .text_xs()
                            .text_color(theme.muted_foreground)
                            .child(if chinese {
                                "微软拼音 · 中文"
                            } else {
                                "英文 / 美式键盘"
                            }),
                    ),
            )
            .child(
                div()
                    .rounded(theme.radius_lg)
                    .border_1()
                    .border_color(theme.border)
                    .bg(theme.muted)
                    .p_3()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .text_xs()
                            .text_color(theme.muted_foreground)
                            .child(Icon::new(IconName::Terminal).small())
                            .child("Windows Terminal")
                            .child(div().flex_1())
                            .child("预览"),
                    )
                    .child(
                        div()
                            .h_8()
                            .flex()
                            .items_center()
                            .gap_2()
                            .font_family(theme.mono_font_family.clone())
                            .child(div().text_color(theme.muted_foreground).child(">"))
                            .child(sample)
                            .child(div().w_2().h_5().bg(rgb(color))),
                    ),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .gap_3()
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(div().size_3().rounded(theme.radius).bg(rgb(color)))
                            .child(
                                div()
                                    .font_family(theme.mono_font_family.clone())
                                    .child(value.clone()),
                            ),
                    )
                    .child(
                        Button::new(if chinese { "cn-custom" } else { "en-custom" })
                            .small()
                            .label("自定义颜色…")
                            .accessibility_label(format!("自定义{label}光标颜色"))
                            .disabled(self.color_dialog_open || self.elevating)
                            .on_click(cx.listener(move |view, _, window, cx| {
                                if !view.color_dialog_open {
                                    view.color_dialog_open = true;
                                    native::choose_color(chinese, color, native_hwnd(window));
                                    cx.notify();
                                }
                            })),
                    ),
            )
    }

    fn scheme_selector(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme().clone();
        div()
            .flex()
            .flex_col()
            .gap_2()
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_3()
                    .child(section_title("配色方案", cx))
                    .child(help("左：中文 · 右：英文；修改上方颜色可编辑当前组。", cx)),
            )
            .child(
                div()
                    .flex()
                    .gap_2()
                    .children(self.options.color_schemes.iter().enumerate().map(
                        |(index, scheme)| {
                            let cn = Color::parse(&scheme.chinese_color)
                                .expect("validated scheme")
                                .0;
                            let en = Color::parse(&scheme.english_color)
                                .expect("validated scheme")
                                .0;
                            Button::new(("scheme", index))
                                .flex_1()
                                .min_w_0()
                                .outline()
                                .label(SCHEME_NAMES[index])
                                .accessibility_label(format!(
                                    "配色{}：{}",
                                    index + 1,
                                    SCHEME_NAMES[index]
                                ))
                                .tooltip(format!(
                                    "{} · 中文 {} / 英文 {}",
                                    SCHEME_NAMES[index], scheme.chinese_color, scheme.english_color
                                ))
                                .selected(self.options.active_scheme == index)
                                .toggled(self.options.active_scheme == index)
                                .disabled(self.color_dialog_open || self.elevating)
                                // These RGB values are user-editable cursor data.
                                .child(
                                    div()
                                        .flex()
                                        .gap_1()
                                        .child(div().size_3().rounded(theme.radius).bg(rgb(cn)))
                                        .child(div().size_3().rounded(theme.radius).bg(rgb(en))),
                                )
                                .on_click(cx.listener(move |view, _, _, cx| {
                                    let mut next = view.options.clone();
                                    next.select_scheme(index);
                                    view.persist(next, cx);
                                }))
                        },
                    )),
            )
    }

    fn connection_status(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme().clone();
        let count = self.runtime.sessions.len();
        let blocked = !self.runtime.elevated_terminal_pids.is_empty();
        let has_errors = !self.runtime.errors.is_empty();
        let (icon, color, title, detail) = if self.controller_stopped {
            (
                IconName::CircleAlert,
                theme.danger,
                "后台连接已停止".to_owned(),
                "请从系统托盘退出，再重新打开 Terminal IME。",
            )
        } else if blocked {
            (
                IconName::ShieldAlert,
                theme.warning,
                "管理员终端需要授权".to_owned(),
                "软件与终端权限不一致。以管理员身份重新启动后即可接入。",
            )
        } else if has_errors {
            (
                IconName::CircleAlert,
                theme.warning,
                "部分会话未能接入".to_owned(),
                "请检查终端权限，或退出后重新打开 Terminal IME。",
            )
        } else if !self.options.enabled {
            (
                IconName::Pause,
                theme.muted_foreground,
                "光标指示已暂停".to_owned(),
                "终端使用默认光标颜色。重新启用后继续跟随输入状态。",
            )
        } else if count == 0 {
            (
                IconName::Terminal,
                theme.muted_foreground,
                "等待 Windows Terminal".to_owned(),
                "打开终端即可自动接入，无需修改 Nushell 或 PowerShell 配置。",
            )
        } else {
            (
                IconName::CircleCheck,
                theme.success,
                format!("已接入 {count} 个终端会话"),
                "新会话会在数秒内自动接入。切换中英文，光标颜色随之变化。",
            )
        };
        div().flex_shrink_0().flex().flex_col().gap_2().child(
            div()
                .flex()
                .items_start()
                .gap_3()
                .child(Icon::new(icon).small().text_color(color))
                .child(
                    div()
                        .flex_1()
                        .min_w_0()
                        .flex()
                        .flex_col()
                        .gap_2()
                        .child(div().font_weight(FontWeight::MEDIUM).child(title))
                        .child(help(detail, cx))
                        .children(self.runtime.sessions.iter().take(3).map(|session| {
                            let shell = match session.shell.as_str() {
                                "nu.exe" => "Nushell",
                                "powershell.exe" | "pwsh.exe" => "PowerShell",
                                "cmd.exe" => "命令提示符",
                                other => other,
                            };
                            div()
                                .flex()
                                .items_center()
                                .gap_2()
                                .text_xs()
                                .text_color(theme.muted_foreground)
                                .child(shell.to_owned())
                                .child(format!("· PID {} · 已接入", session.shell_pid))
                        }))
                        .when(count > 3, |d| {
                            d.child(help(format!("另有 {} 个会话已接入", count - 3), cx))
                        }),
                )
                .when(!self.elevated, |d| {
                    d.child(
                        Button::new("admin-mode")
                            .small()
                            .label(if self.elevating {
                                "正在请求授权…"
                            } else {
                                "以管理员身份重新启动"
                            })
                            .disabled(self.elevating || self.color_dialog_open)
                            .on_click(cx.listener(|view, _, _, cx| view.elevate(cx))),
                    )
                }),
        )
    }
}

fn section_title(title: &'static str, cx: &App) -> impl IntoElement {
    div()
        .text_sm()
        .font_weight(FontWeight::SEMIBOLD)
        .text_color(cx.theme().foreground)
        .child(title)
}

fn help(text: impl Into<SharedString>, cx: &App) -> impl IntoElement {
    div()
        .text_xs()
        .text_color(cx.theme().muted_foreground)
        .child(text.into())
}

impl Render for SettingsView {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme().clone();
        let default_colors = self.options.color_schemes[self.options.active_scheme]
            == default_schemes()[self.options.active_scheme];
        div()
            .size_full()
            .flex()
            .flex_col()
            .bg(theme.background)
            .text_color(theme.foreground)
            .font_family(theme.font_family.clone())
            .text_sm()
            .child(
                div()
                    .flex_shrink_0()
                    .px_5()
                    .py_3()
                    .flex()
                    .items_center()
                    .gap_3()
                    .border_b_1()
                    .border_color(theme.border)
                    .child(Icon::new(IconName::Terminal).text_color(theme.muted_foreground))
                    .child(div().font_weight(FontWeight::SEMIBOLD).child("Terminal IME"))
                    .child(help("设置", cx))
                    .child(div().flex_1())
                    .child(self.appearance_control(cx)),
            )
            .child(
                div()
                    .id("settings-scroll")
                    .flex_1()
                    .min_h_0()
                    .child(
                        div()
                            .p_5()
                            .flex()
                            .flex_col()
                            .gap_4()
                            .child(
                                div()
                                    .flex_shrink_0()
                                    .flex()
                                    .items_center()
                                    .gap_3()
                                    .child(
                                        div()
                                            .flex_1()
                                            .min_w_0()
                                            .flex()
                                            .flex_col()
                                            .gap_1()
                                            .child(div().text_xl().font_weight(FontWeight::SEMIBOLD).child("光标颜色指示"))
                                            .child(help("在 Windows Terminal 中，用光标颜色区分中英文输入。", cx)),
                                    )
                                    .child(
                                        Switch::new("enable")
                                            .checked(self.options.enabled)
                                            .label(if self.options.enabled { "已启用" } else { "已暂停" })
                                            .accessibility_label("启用光标颜色指示")
                                            .on_change(cx.listener(|view, enabled: &bool, _, cx| {
                                                let mut next = view.options.clone();
                                                next.enabled = *enabled;
                                                view.persist(next, cx);
                                            })),
                                    ),
                            )
                            .child(
                                div()
                                    .flex_shrink_0()
                                    .flex()
                                    .flex_col()
                                    .gap_3()
                                    .child(div().flex().gap_4().child(self.color_editor(true, cx)).child(self.color_editor(false, cx)))
                                    .child(self.scheme_selector(cx)),
                            )
                            .child(
                                div()
                                    .flex_shrink_0()
                                    .pt_4()
                                    .border_t_1()
                                    .border_color(theme.border)
                                    .child(self.connection_status(cx)),
                            )
                            .child(
                                div()
                                    .flex_shrink_0()
                                    .pt_4()
                                    .border_t_1()
                                    .border_color(theme.border)
                                    .flex()
                                    .items_center()
                                    .gap_4()
                                    .child(
                                        div()
                                            .flex_1()
                                            .min_w_0()
                                            .flex()
                                            .flex_col()
                                            .gap_1()
                                            .child("登录 Windows 时启动")
                                            .child(help("启动后驻留系统托盘，自动接入终端。", cx)),
                                    )
                                    .child(
                                        Switch::new("autostart")
                                            .checked(self.options.auto_start)
                                            .accessibility_label("登录 Windows 时启动")
                                            .on_change(cx.listener(|view, enabled: &bool, _, cx| {
                                                let mut next = view.options.clone();
                                                next.auto_start = *enabled;
                                                view.persist(next, cx);
                                            })),
                                    ),
                            )
                    )
                    // The scrollbar belongs to the unpadded region; content owns its inset.
                    .overflow_y_scrollbar(),
            )
            .child(
                div()
                    .flex_shrink_0()
                    .px_5()
                    .py_3()
                    .border_t_1()
                    .border_color(theme.border)
                    .flex()
                    .items_center()
                    .gap_4()
                    .child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .text_xs()
                            .text_color(if self.notice_is_error { theme.danger } else { theme.muted_foreground })
                            .child(format!("{} · 关闭窗口后驻留托盘", self.notice)),
                    )
                    .child(
                        Button::new("reset-defaults")
                            .small()
                            .ghost()
                            .icon(IconName::RotateCcw)
                            .label("重置当前配色")
                            .tooltip("将当前组恢复为推荐配色，保留其他组的自定义颜色")
                            .disabled(default_colors || self.color_dialog_open)
                            .on_click(cx.listener(|view, _, _, cx| {
                                let mut next = view.options.clone();
                                next.reset_active_scheme();
                                view.persist(next, cx);
                            })),
                    ),
            )
    }
}
