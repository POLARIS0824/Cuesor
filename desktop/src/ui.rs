use super::{native, native_hwnd, SettingsView};
use cuesor::{
    model::Color,
    settings::{default_schemes, Appearance, SCHEME_NAMES},
};
use gpui::{prelude::*, *};
use gpui_kit::{
    assets::IconName,
    base::{Switch, SwitchThumb, SwitchTrack},
    component::{
        button::{Button, ButtonCustomVariant, ButtonGroup, ButtonVariants},
        scroll::ScrollableElement,
        ActiveTheme, Disableable, Icon, Selectable, Sizable, Theme, ThemeMode,
    },
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
            .children(
                [
                    ("系统", IconName::Monitor, Appearance::System),
                    ("浅色", IconName::Sun, Appearance::Light),
                    ("深色", IconName::Moon, Appearance::Dark),
                ]
                .into_iter()
                .enumerate()
                .map(|(index, (label, icon, mode))| {
                    radix_button(
                        ("appearance", index),
                        self.options.appearance == mode,
                        false,
                        cx,
                    )
                    .when(index > 0, |button| button.border_l(px(0.)))
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
            .child(div().font_weight(FontWeight::MEDIUM).child(label))
            .child(
                div()
                    .rounded(theme.radius_lg)
                    .border(px(0.5))
                    .border_color(theme.border)
                    .bg(theme.muted)
                    .p_3()
                    .flex()
                    .flex_col()
                    .gap_2()
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
                        radix_button(
                            if chinese { "cn-custom" } else { "en-custom" },
                            false,
                            self.color_dialog_open || self.elevating,
                            cx,
                        )
                        .small()
                        .label("自定义颜色…")
                        .accessibility_label(format!("自定义{label}光标颜色"))
                        .disabled(self.color_dialog_open || self.elevating)
                        .on_click(cx.listener(
                            move |view, _, window, cx| {
                                if !view.color_dialog_open {
                                    view.color_dialog_open = true;
                                    native::choose_color(chinese, color, native_hwnd(window));
                                    cx.notify();
                                }
                            },
                        )),
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
                    .child(help("左侧为中文，右侧为英文", cx))
                    .child(div().flex_1())
                    .child(
                        radix_button(
                            "reset-defaults",
                            false,
                            self.options.color_schemes[self.options.active_scheme]
                                == default_schemes()[self.options.active_scheme]
                                || self.color_dialog_open,
                            cx,
                        )
                        .small()
                        .icon(IconName::RotateCcw)
                        .label("重置当前配色")
                        .on_click(cx.listener(|view, _, _, cx| {
                            let mut next = view.options.clone();
                            next.reset_active_scheme();
                            view.persist(next, cx);
                        })),
                    ),
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
                            radix_button(
                                ("scheme", index),
                                self.options.active_scheme == index,
                                self.color_dialog_open || self.elevating,
                                cx,
                            )
                            .flex_1()
                            .min_w_0()
                            .label(SCHEME_NAMES[index])
                            .accessibility_label(format!(
                                "配色{}：{}",
                                index + 1,
                                SCHEME_NAMES[index]
                            ))
                            .selected(self.options.active_scheme == index)
                            .toggled(self.options.active_scheme == index)
                            .disabled(self.color_dialog_open || self.elevating)
                            .child(
                                div()
                                    .flex()
                                    .gap_1()
                                    .child(div().size_3().rounded(theme.radius).bg(rgb(cn)))
                                    .child(div().size_3().rounded(theme.radius).bg(rgb(en))),
                            )
                            .on_click(cx.listener(
                                move |view, _, _, cx| {
                                    let mut next = view.options.clone();
                                    next.select_scheme(index);
                                    view.persist(next, cx);
                                },
                            ))
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
                Some("请从系统托盘退出，再重新打开 Cuesor。"),
            )
        } else if blocked {
            (
                IconName::ShieldAlert,
                theme.warning,
                "管理员终端需要授权".to_owned(),
                Some("以管理员身份重新启动后即可接入。"),
            )
        } else if has_errors {
            (
                IconName::CircleAlert,
                theme.warning,
                "部分会话未能接入".to_owned(),
                Some("请检查终端权限，或退出后重新打开 Cuesor。"),
            )
        } else if !self.options.enabled {
            (
                IconName::Pause,
                theme.muted_foreground,
                "光标指示已暂停".to_owned(),
                None,
            )
        } else if count == 0 {
            (
                IconName::Terminal,
                theme.muted_foreground,
                "等待 Windows Terminal".to_owned(),
                Some("打开 Windows Terminal 即可接入"),
            )
        } else {
            (
                IconName::CircleCheck,
                theme.success,
                format!("已接入 {count} 个终端会话"),
                None,
            )
        };
        div().flex_shrink_0().flex().flex_col().gap_2().child(
            div()
                .flex()
                .items_start()
                .gap_3()
                .child(
                    div()
                        .h(px(20.))
                        .flex()
                        .items_center()
                        .child(Icon::new(icon).small().text_color(color)),
                )
                .child(
                    div()
                        .flex_1()
                        .min_w_0()
                        .flex()
                        .flex_col()
                        .gap_2()
                        .child(div().font_weight(FontWeight::MEDIUM).child(title))
                        .when_some(detail, |d, detail| d.child(help(detail, cx)))
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
                                .child(format!("PID {}", session.shell_pid))
                        }))
                        .when(count > 3, |d| {
                            d.child(help(format!("另有 {} 个会话已接入", count - 3), cx))
                        }),
                )
                .when(blocked && !self.elevated, |d| {
                    d.child(
                        radix_button(
                            "admin-mode",
                            false,
                            self.elevating || self.color_dialog_open,
                            cx,
                        )
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

fn radix_button(id: impl Into<ElementId>, selected: bool, disabled: bool, cx: &App) -> Button {
    let theme = cx.theme();
    let (background, hover, active, foreground, border) = if selected {
        (
            theme.accent,
            theme.tab_active,
            theme.tab_active,
            theme.accent_foreground,
            theme.list_active_border,
        )
    } else {
        (
            theme.button,
            theme.button_hover,
            theme.button_active,
            theme.button_foreground,
            theme.input,
        )
    };
    Button::new(id)
        .custom(
            ButtonCustomVariant::new(cx)
                .color(background)
                .hover(hover)
                .active(active)
                .foreground(foreground)
                .shadow(false),
        )
        .selected(selected)
        .disabled(disabled)
        .border(px(0.5))
        .border_color(border)
        .focus_visible(|style| style.border_color(theme.ring))
        .when(disabled, |button| {
            button
                .bg(theme.muted)
                .text_color(theme.skeleton)
                .border_color(theme.border)
        })
}

fn radix_switch(
    id: &'static str,
    checked: bool,
    label: &'static str,
    show_label: bool,
    window: &mut Window,
    cx: &mut App,
) -> Switch {
    let theme = cx.theme().clone();
    let focus = window
        .use_keyed_state(id, cx, |_, cx| cx.focus_handle())
        .read(cx)
        .clone();
    let border = if focus.is_focused(window) {
        theme.ring
    } else {
        theme.border
    };
    Switch::new(id)
        .checked(checked)
        .accessibility_label(label)
        .track_focus(&focus)
        .flex()
        .items_center()
        .gap_2()
        .child(
            SwitchTrack::new((id, 0usize))
                .checked(checked)
                .w(px(36.))
                .h(px(20.))
                .p(px(1.5))
                .rounded_full()
                .border(px(0.5))
                .border_color(border)
                .bg(if checked { theme.primary } else { theme.switch })
                .child(
                    SwitchThumb::new(checked)
                        .size(px(16.))
                        .rounded_full()
                        .bg(theme.switch_thumb)
                        .when(checked, |thumb| thumb.ml(px(16.))),
                ),
        )
        .when(show_label, |switch| {
            switch.child(
                div()
                    .text_xs()
                    .child(if checked { "已启用" } else { "已暂停" }),
            )
        })
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
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme().clone();
        let enable_view = cx.entity().downgrade();
        let autostart_view = enable_view.clone();
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
                    .border_b(px(0.5))
                    .border_color(theme.border)
                    .child(Icon::new(IconName::Terminal).text_color(theme.muted_foreground))
                    .child(div().font_weight(FontWeight::SEMIBOLD).child("Cuesor"))
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
                                        div().flex_1().min_w_0().flex().flex_col().child(
                                            div()
                                                .text_xl()
                                                .font_weight(FontWeight::SEMIBOLD)
                                                .child("光标颜色指示"),
                                        ),
                                    )
                                    .child(
                                        radix_switch(
                                            "enable",
                                            self.options.enabled,
                                            "启用光标颜色指示",
                                            true,
                                            window,
                                            cx,
                                        )
                                        .on_change(
                                            move |enabled, _, _, cx| {
                                                _ = enable_view.update(cx, |view, cx| {
                                                    let mut next = view.options.clone();
                                                    next.enabled = enabled;
                                                    view.persist(next, cx);
                                                });
                                            },
                                        ),
                                    ),
                            )
                            .child(
                                div()
                                    .flex_shrink_0()
                                    .flex()
                                    .flex_col()
                                    .gap_3()
                                    .child(
                                        div()
                                            .flex()
                                            .gap_4()
                                            .child(self.color_editor(true, cx))
                                            .child(self.color_editor(false, cx)),
                                    )
                                    .child(self.scheme_selector(cx)),
                            )
                            .child(
                                div()
                                    .flex_shrink_0()
                                    .pt_4()
                                    .border_t(px(0.5))
                                    .border_color(theme.border)
                                    .child(self.connection_status(cx)),
                            )
                            .child(
                                div()
                                    .flex_shrink_0()
                                    .pt_4()
                                    .border_t(px(0.5))
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
                                            .child("登录 Windows 时启动"),
                                    )
                                    .child(
                                        radix_switch(
                                            "autostart",
                                            self.options.auto_start,
                                            "登录 Windows 时启动",
                                            false,
                                            window,
                                            cx,
                                        )
                                        .on_change(
                                            move |enabled, _, _, cx| {
                                                _ = autostart_view.update(cx, |view, cx| {
                                                    let mut next = view.options.clone();
                                                    next.auto_start = enabled;
                                                    view.persist(next, cx);
                                                });
                                            },
                                        ),
                                    ),
                            ),
                    )
                    .overflow_y_scrollbar(),
            )
            .child(
                div()
                    .flex_shrink_0()
                    .px_5()
                    .py_3()
                    .border_t(px(0.5))
                    .border_color(theme.border)
                    .flex()
                    .items_center()
                    .gap_4()
                    .child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .text_xs()
                            .text_color(if self.notice_is_error {
                                theme.danger
                            } else {
                                theme.muted_foreground
                            })
                            .child(self.notice.clone()),
                    ),
            )
    }
}
