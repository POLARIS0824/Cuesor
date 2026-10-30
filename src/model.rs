use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ImeMode {
    Chinese,
    Latin,
    Unknown,
}

impl fmt::Display for ImeMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Chinese => "chinese",
            Self::Latin => "latin",
            Self::Unknown => "unknown",
        })
    }
}

/// A query failure is never evidence of Latin mode.
pub fn classify(language: u16, open: Option<usize>, conversion: Option<usize>) -> ImeMode {
    match language {
        lang if (lang & 0x03FF) == 0x09 => ImeMode::Latin,
        0x0804 => match (open, conversion) {
            (Some(0), _) => ImeMode::Latin,
            (Some(_), Some(flags)) if flags & 1 != 0 => ImeMode::Chinese,
            (Some(_), Some(_)) => ImeMode::Latin,
            _ => ImeMode::Unknown,
        },
        _ => ImeMode::Unknown,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Color(pub u32);

impl Color {
    pub fn parse(value: &str) -> Result<Self, String> {
        let hex = value.strip_prefix('#').unwrap_or(value);
        if hex.len() != 6 || !hex.bytes().all(|c| c.is_ascii_hexdigit()) {
            return Err(format!("invalid color {value:?}; expected #RRGGBB"));
        }
        Ok(Self(u32::from_str_radix(hex, 16).unwrap()))
    }

    pub fn sequence(self) -> String {
        format!("\x1b]12;#{:06X}\x1b\\", self.0)
    }
}

pub const RESET_CURSOR: &str = "\x1b]112\x1b\\";

#[derive(Clone, Copy, Debug)]
pub struct Colors {
    pub chinese: Color,
    pub latin: Color,
}

impl Colors {
    pub fn sequence(self, mode: ImeMode) -> String {
        match mode {
            ImeMode::Chinese => self.chinese.sequence(),
            ImeMode::Latin => self.latin.sequence(),
            ImeMode::Unknown => RESET_CURSOR.to_owned(),
        }
    }
}

/// Only commit a transition after the native output call succeeds.
#[derive(Default)]
pub struct ColorState {
    last: Option<(ImeMode, usize)>,
}

impl ColorState {
    pub fn pending(&self, mode: ImeMode, host: usize) -> Option<ImeMode> {
        if self.last == Some((mode, host)) || (self.last.is_none() && mode == ImeMode::Unknown) {
            None
        } else {
            Some(mode)
        }
    }

    pub fn applied(&mut self, mode: ImeMode, host: usize) {
        self.last = Some((mode, host));
    }

    pub fn is_colored(&self) -> bool {
        self.last.is_some_and(|(m, _)| m != ImeMode::Unknown)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn failed_or_unsupported_queries_stay_unknown() {
        assert_eq!(classify(0x0804, None, None), ImeMode::Unknown);
        assert_eq!(classify(0x0804, Some(1), None), ImeMode::Unknown);
        assert_eq!(classify(0x0411, Some(1), Some(1)), ImeMode::Unknown);
        assert_eq!(classify(0, Some(0), None), ImeMode::Unknown);
    }

    #[test]
    fn distinguishes_layout_and_conversion_modes() {
        assert_eq!(classify(0x0409, None, None), ImeMode::Latin);
        assert_eq!(classify(0x0809, None, None), ImeMode::Latin); // UK English
        assert_eq!(classify(0x0804, Some(0), None), ImeMode::Latin);
        assert_eq!(classify(0x0804, Some(1), Some(1)), ImeMode::Chinese);
        assert_eq!(classify(0x0804, Some(1), Some(0x400)), ImeMode::Latin);
    }

    #[test]
    fn color_input_cannot_inject_terminal_control_sequences() {
        for invalid in ["#fff", "#12345678", "#GG0000", "#123456\x07", "#你好"] {
            assert!(Color::parse(invalid).is_err(), "{invalid:?}");
        }
        assert_eq!(
            Color::parse("#ff7800").unwrap().sequence(),
            "\x1b]12;#FF7800\x1b\\"
        );
        let colors = Colors {
            chinese: Color(0xFF7800),
            latin: Color(0x0077FF),
        };
        assert_eq!(colors.sequence(ImeMode::Unknown), RESET_CURSOR);
    }

    #[test]
    fn focus_loss_restores_once_and_failed_writes_remain_pending() {
        let mut state = ColorState::default();
        assert_eq!(state.pending(ImeMode::Unknown, 1), None);
        assert_eq!(state.pending(ImeMode::Chinese, 1), Some(ImeMode::Chinese));
        assert_eq!(state.pending(ImeMode::Chinese, 1), Some(ImeMode::Chinese));
        state.applied(ImeMode::Chinese, 1);
        assert!(state.is_colored());
        assert_eq!(state.pending(ImeMode::Chinese, 1), None);
        assert_eq!(state.pending(ImeMode::Chinese, 2), Some(ImeMode::Chinese));
        assert_eq!(state.pending(ImeMode::Unknown, 1), Some(ImeMode::Unknown));
        state.applied(ImeMode::Unknown, 1);
        assert!(!state.is_colored());
        assert_eq!(state.pending(ImeMode::Unknown, 1), None);
        assert_eq!(state.pending(ImeMode::Chinese, 1), Some(ImeMode::Chinese));
    }
}
