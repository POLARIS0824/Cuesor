use super::{i18n, native, native_hwnd, SettingsView};
use cuesor::{
    model::Color,
    settings::{default_schemes, Appearance},
};
use gpui::{prelude::*, *};
use gpui_kit::{
    assets::IconName,
    component::{
        alert::{Alert, AlertVariant},
        button::{Button, ButtonGroup},
        label::Label,
        scroll::ScrollableElement,
        switch::Switch,
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
        let locale = self.locale;
        ButtonGroup::new("appearance")
            .small()
            .children(
                [
                    (i18n::appearance_system(locale), IconName::Monitor, Appearance::System),
                    (i18n::appearance_light(locale), IconName::Sun, Appearance::Light),
                    (i18n::appearance_dark(locale), IconName::Moon, Appearance::Dark),
                ]
                .into_iter()
                .enumerate()
                .map(move |(index, (label, icon, mode))| {
                    Button::new(("appearance", index))
                        .icon(icon)
                        .label(label)
                        .accessibility_label(i18n::accessibility_appearance(locale, label))
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
        let locale = self.locale;
        let (label, sample, value) = if chinese {
            (
                i18n::color_editor_chinese_label(locale),
                i18n::color_editor_chinese_sample(locale),
                &self.options.active_colors().chinese_color,
            )
        } else {
            (
                i18n::color_editor_english_label(locale),
                i18n::color_editor_english_sample(locale),
                &self.options.active_colors().english_color,
            )
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
                    .border_1()
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
                        Button::new(if chinese { "cn-custom" } else { "en-custom" })
                            .small()
                            .label(i18n::color_custom_button(locale))
                            .accessibility_label(i18n::accessibility_custom_color(locale, label))
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
        let locale = self.locale;
        let scheme_names = cuesor::i18n::messages::scheme_names(locale);
        div()
            .flex()
            .flex_col()
            .gap_2()
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_3()
                    .child(Label::new(i18n::scheme_selector_title(locale)).font_weight(FontWeight::SEMIBOLD))
                    .child(
                        Label::new(i18n::scheme_selector_hint(locale))
                            .text_xs()
                            .text_color(theme.muted_foreground),
                    )
                    .child(div().flex_1())
                    .child(
                        Button::new("reset-defaults")
                            .disabled(
                                self.options.active_colors()
                                    == &default_schemes()[self.options.active_scheme]
                                    || self.color_dialog_open
                                    || self.elevating,
                            )
                            .small()
                            .icon(IconName::RotateCcw)
                            .label(i18n::scheme_reset_button(locale))
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
                            let name = scheme_names[index];
                            Button::new(("scheme", index))
                                .flex_1()
                                .min_w_0()
                                .label(name)
                                .accessibility_label(i18n::accessibility_scheme(locale, index + 1, name))
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
        let count = self.runtime.sessions.len();
        let blocked = !self.runtime.elevated_terminal_pids.is_empty();
        let locale = self.locale;
        let (variant, icon, title, detail) = if self.controller_stopped {
            (
                AlertVariant::Error,
                IconName::CircleAlert,
                i18n::alert_stopped_title(locale).to_owned(),
                i18n::alert_stopped_detail(locale),
            )
        } else if blocked {
            (
                AlertVariant::Warning,
                IconName::ShieldAlert,
                i18n::alert_admin_title(locale).to_owned(),
                i18n::alert_admin_detail(locale),
            )
        } else if !self.runtime.errors.is_empty() {
            (
                AlertVariant::Warning,
                IconName::CircleAlert,
                i18n::alert_partial_title(locale).to_owned(),
                i18n::alert_partial_detail(locale),
            )
        } else if !self.options.enabled {
            (
                AlertVariant::Default,
                IconName::Pause,
                i18n::alert_paused_title(locale).to_owned(),
                "",
            )
        } else if count == 0 {
            (
                AlertVariant::Info,
                IconName::Terminal,
                i18n::alert_waiting_title(locale).to_owned(),
                i18n::alert_waiting_detail(locale),
            )
        } else {
            (
                AlertVariant::Success,
                IconName::CircleCheck,
                i18n::alert_connected_title(locale, count),
                "",
            )
        };
        div()
            .flex()
            .flex_col()
            .gap_2()
            .child(
                Alert::new("connection-status", detail)
                    .title(title)
                    .with_variant(variant)
                    .icon(icon)
                    .small(),
            )
            .children(self.runtime.sessions.iter().take(3).map(|session| {
                let shell = match session.shell.as_str() {
                    "nu.exe" => "Nushell",
                    "powershell.exe" | "pwsh.exe" => "PowerShell",
                    "cmd.exe" => i18n::shell_cmd_display(locale),
                    other => other,
                };
                Label::new(format!("{shell}  PID {}", session.shell_pid))
                    .text_xs()
                    .text_color(cx.theme().muted_foreground)
            }))
            .when(count > 3, |d| {
                d.child(Label::new(i18n::more_sessions_connected(locale, count - 3)))
            })
            .when(blocked && !self.elevated, |d| {
                d.child(
                    Button::new("admin-mode")
                        .small()
                        .label(if self.elevating {
                            i18n::btn_requesting_elevation(locale)
                        } else {
                            i18n::btn_restart_elevated(locale)
                        })
                        .disabled(self.elevating || self.color_dialog_open)
                        .on_click(cx.listener(|view, _, _, cx| view.elevate(cx))),
                )
            })
    }
}

impl Render for SettingsView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme().clone();
        let locale = self.locale;
        let enable_view = cx.entity().downgrade();
        let autostart_view = enable_view.clone();
        let run_as_admin_view = enable_view.clone();
        let obsidian_view = enable_view.clone();
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
                                                .child(i18n::section_indicator_title(locale)),
                                        ),
                                    )
                                    .child(
                                        Switch::new("enable")
                                            .checked(self.options.enabled)
                                            .label(if self.options.enabled {
                                                i18n::switch_enabled(locale)
                                            } else {
                                                i18n::switch_paused(locale)
                                            })
                                            .accessibility_label(i18n::accessibility_indicator_switch(locale))
                                            .on_change(move |enabled, _, cx| {
                                                _ = enable_view.update(cx, |view, cx| {
                                                    let mut next = view.options.clone();
                                                    next.enabled = *enabled;
                                                    view.persist(next, cx);
                                                });
                                            }),
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
                                            .child(i18n::section_autostart_title(locale)),
                                    )
                                    .child(
                                        Switch::new("autostart")
                                            .checked(self.options.auto_start)
                                            .accessibility_label(i18n::accessibility_autostart_switch(locale))
                                            .on_change(move |enabled, _, cx| {
                                                _ = autostart_view.update(cx, |view, cx| {
                                                    let mut next = view.options.clone();
                                                    next.auto_start = *enabled;
                                                    view.persist(next, cx);
                                                });
                                            }),
                                    ),
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
                                            .child(
                                                div()
                                                    .flex()
                                                    .items_center()
                                                    .gap_2()
                                                    .child(div().child(i18n::section_obsidian_title(locale)))
                                                    .child({
                                                        let status_text: SharedString = if !self.options.obsidian_enabled {
                                                            i18n::status_obsidian_disabled(locale).into()
                                                        } else if self.runtime.obsidian_clients > 0 {
                                                            i18n::status_obsidian_connected(locale).into()
                                                        } else {
                                                            i18n::status_obsidian_listening(locale, 23333).into()
                                                        };
                                                        div()
                                                            .text_xs()
                                                            .text_color(if !self.options.obsidian_enabled {
                                                                theme.muted_foreground
                                                            } else if self.runtime.obsidian_clients > 0 {
                                                                rgb(0x30A46C).into()
                                                            } else {
                                                                theme.muted_foreground
                                                            })
                                                            .child(status_text)
                                                    }),
                                            )
                                            .child(
                                                div()
                                                    .text_xs()
                                                    .text_color(theme.muted_foreground)
                                                    .child(i18n::section_obsidian_desc(locale)),
                                            ),
                                    )
                                    .child(
                                        Switch::new("obsidian-toggle")
                                            .checked(self.options.obsidian_enabled)
                                            .accessibility_label(i18n::accessibility_obsidian_switch(locale))
                                            .on_change(move |enabled, _, cx| {
                                                _ = obsidian_view.update(cx, |view, cx| {
                                                    let mut next = view.options.clone();
                                                    next.obsidian_enabled = *enabled;
                                                    view.persist(next, cx);
                                                });
                                            }),
                                    ),
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
                                            .child(div().child(i18n::section_admin_title(locale)))
                                            .child(
                                                div()
                                                    .text_xs()
                                                    .text_color(theme.muted_foreground)
                                                    .child(i18n::section_admin_desc(locale)),
                                            ),
                                    )
                                    .child(
                                        Switch::new("run-as-admin")
                                            .checked(self.options.run_as_admin)
                                            .accessibility_label(i18n::accessibility_admin_switch(locale))
                                            .on_change(move |enabled, _, cx| {
                                                _ = run_as_admin_view.update(cx, |view, cx| {
                                                    let mut next = view.options.clone();
                                                    next.run_as_admin = *enabled;
                                                    view.persist(next, cx);
                                                });
                                            }),
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
