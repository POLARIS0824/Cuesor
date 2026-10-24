use std::rc::Rc;

use gpui::App;
use gpui_kit::component::{Theme, ThemeConfig, ThemeConfigColors, ThemeMode};

const SLATE_LIGHT: [&str; 12] = [
    "#fcfcfd", "#f9f9fb", "#f0f0f3", "#e8e8ec", "#e0e1e6", "#d9d9e0", "#cdced6", "#b9bbc6",
    "#8b8d98", "#80838d", "#60646c", "#1c2024",
];
const SLATE_DARK: [&str; 12] = [
    "#111113", "#18191b", "#212225", "#272a2d", "#2e3135", "#363a3f", "#43484e", "#5a6169",
    "#696e77", "#777b84", "#b0b4ba", "#edeef0",
];
const INDIGO_LIGHT: [&str; 12] = [
    "#fdfdfe", "#f7f9ff", "#edf2fe", "#e1e9ff", "#d2deff", "#c1d0ff", "#abbdf9", "#8da4ef",
    "#3e63dd", "#3358d4", "#3a5bc7", "#1f2d5c",
];
const INDIGO_DARK: [&str; 12] = [
    "#11131f", "#141726", "#182449", "#1d2e62", "#253974", "#304384", "#3a4f97", "#435db1",
    "#3e63dd", "#5472e4", "#9eb1ff", "#d6e1ff",
];

macro_rules! colors {
    ($($field:ident: $value:expr),* $(,)?) => {{
        let mut colors = ThemeConfigColors::default();
        $(colors.$field = $value;)*
        colors
    }};
}

fn config(dark: bool) -> ThemeConfig {
    let s = if dark { SLATE_DARK } else { SLATE_LIGHT };
    let i = if dark { INDIGO_DARK } else { INDIGO_LIGHT };
    let (red, amber, teal) = if dark {
        ("#ff9592", "#ffca16", "#0bd8b6")
    } else {
        ("#ce2c31", "#ab6400", "#008573")
    };
    let colors = colors! {
        background: Some(s[0].into()),
        foreground: Some(s[11].into()),
        muted: Some(s[1].into()),
        muted_foreground: Some(s[10].into()),
        border: Some(s[5].into()),
        input: Some(s[6].into()),
        popover: Some(s[1].into()),
        popover_foreground: Some(s[11].into()),
        accent: Some(i[2].into()),
        accent_foreground: Some(i[10].into()),
        primary: Some(i[8].into()),
        primary_hover: Some(i[9].into()),
        primary_active: Some(i[7].into()),
        primary_foreground: Some(s[0].into()),
        button: Some(s[2].into()),
        button_hover: Some(s[3].into()),
        button_active: Some(s[4].into()),
        button_foreground: Some(s[11].into()),
        button_primary: Some(i[8].into()),
        button_primary_hover: Some(i[9].into()),
        button_primary_active: Some(i[7].into()),
        button_primary_foreground: Some(s[0].into()),
        button_secondary: Some(s[2].into()),
        button_secondary_hover: Some(s[3].into()),
        button_secondary_active: Some(s[4].into()),
        button_secondary_foreground: Some(s[11].into()),
        secondary: Some(s[2].into()),
        secondary_hover: Some(s[3].into()),
        secondary_active: Some(s[4].into()),
        secondary_foreground: Some(s[11].into()),
        switch: Some(s[6].into()),
        switch_thumb: Some(s[0].into()),
        ring: Some(i[7].into()),
        selection: Some(i[2].into()),
        tab_active: Some(i[3].into()),
        tab_active_foreground: Some(i[10].into()),
        list: Some(s[0].into()),
        list_hover: Some(s[3].into()),
        list_active: Some(i[2].into()),
        list_active_border: Some(i[6].into()),
        scrollbar: Some(s[1].into()),
        scrollbar_thumb: Some(s[6].into()),
        scrollbar_thumb_hover: Some(s[7].into()),
        skeleton: Some(s[8].into()),
        danger: Some(red.into()),
        warning: Some(amber.into()),
        success: Some(teal.into()),
    };
    ThemeConfig {
        name: if dark { "Radix dark" } else { "Radix light" }.into(),
        mode: if dark {
            ThemeMode::Dark
        } else {
            ThemeMode::Light
        },
        shadow: Some(false),
        colors,
        ..Default::default()
    }
}

pub fn install(cx: &mut App) {
    Theme::update(cx, |theme| {
        theme.light_theme = Rc::new(config(false));
        theme.dark_theme = Rc::new(config(true));
        theme.focus_ring = false;
    });
}
