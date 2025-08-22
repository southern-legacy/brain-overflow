use std::fmt::Display;

const RESET: &str = "\x1B[0m";
const ESCAPE_BEGIN: &str = "\x1B[";
const ESCAPE_OVER: &str = "m";

pub mod constants {
    pub const BOLD: u8 = 1;
    pub const DIMMED: u8 = 2;
    pub const ITALIC: u8 = 3;
    pub const UNDERLINE: u8 = 4;
    pub const BLINK_SLOWLY: u8 = 5;
    pub const BLINK_RAPIDLY: u8 = 6;
    pub const REVERSE: u8 = 7;
    pub const HIDDEN: u8 = 8;
    pub const STRIKE_THROUGH: u8 = 9;
}

#[derive(Clone, Copy, Default)]
pub struct Bitmap {
    val: u16,
}

#[derive(Clone, Copy, Default)]
pub struct FontStyle {
    options: Bitmap,
}

#[derive(Clone, Copy)]
#[allow(dead_code)]
pub enum AnsiColor {
    Black = 30,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,
    BrightBlack = 90,
    BrightRed,
    BrightGreen,
    BrightYellow,
    BrightBlue,
    BrightMagenta,
    BrightCyan,
    BrightWhite,
}

#[derive(Clone, Copy, Default)]
pub struct AnsiStyle {
    fore: Option<AnsiColor>,
    back: Option<AnsiColor>,
    font_style: FontStyle,
}

#[derive(Clone, Copy)]
pub struct AnsiString<'a> {
    style: AnsiStyle,
    is_vanilla: bool,
    content: &'a str,
}

#[allow(dead_code)]
impl<'a> AnsiString<'a> {
    pub fn new(content: &'a str) -> Self {
        Self {
            style: AnsiStyle::new(),
            is_vanilla: true,
            content,
        }
    }

    pub fn reset(self) -> Self {
        Self::new(self.content)
    }

    pub fn get_content(self) -> &'a str {
        self.content
    }
}

impl AnsiColor {
    #[inline(always)]
    pub fn into_fore(self) -> u8 {
        self as u8
    }

    #[inline(always)]
    pub fn into_back(self) -> u8 {
        self as u8 + 10
    }
}

impl<'a> Display for AnsiString<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_vanilla {
            f.pad(self.content)
        } else {
            f.write_fmt(format_args!("{}", self.style))?;
            f.pad(self.content)?;
            f.write_str(RESET)
        }
    }
}

impl From<u16> for Bitmap {
    fn from(val: u16) -> Self {
        Self { val }
    }
}

impl Into<u16> for Bitmap {
    fn into(self) -> u16 {
        self.val
    }
}

impl Bitmap {
    fn new() -> Self {
        Self { val: 0 }
    }

    fn set(&mut self, idx: u8, set: bool) {
        debug_assert!((idx as usize) < std::mem::size_of::<u16>() * 8);
        if set {
            self.val |= 1 << idx
        } else {
            self.val &= !(1 << idx)
        }
    }

    fn get(self, idx: usize) -> bool {
        debug_assert!((idx as usize) < std::mem::size_of::<u16>() * 8);
        (self.val & (1 << idx)) != 0
    }

    fn merge(self, rhs: Bitmap) -> Bitmap {
        Bitmap::from(self.val | <Bitmap as Into<u16>>::into(rhs))
    }
}

impl FontStyle {
    pub fn new() -> Self {
        Self {
            options: Bitmap::new(),
        }
    }

    pub fn merge(self, rhs: FontStyle) -> FontStyle {
        Self {
            options: self.options.merge(rhs.options),
        }
    }

    pub fn enabled(self, idx: usize) -> bool {
        self.options.get(idx)
    }
}

// switches
#[allow(dead_code)]
impl FontStyle {
    pub fn bold(mut self, enabled: bool) -> Self {
        self.options.set(constants::BOLD, enabled);
        self
    }

    pub fn dimmed(mut self, enabled: bool) -> Self {
        self.options.set(constants::DIMMED, enabled);
        self
    }

    pub fn italic(mut self, enabled: bool) -> Self {
        self.options.set(constants::ITALIC, enabled);
        self
    }

    pub fn underline(mut self, enabled: bool) -> Self {
        self.options.set(constants::UNDERLINE, enabled);
        self
    }

    pub fn blink_slowly(mut self, enabled: bool) -> Self {
        self.options.set(constants::BLINK_SLOWLY, enabled);
        self
    }

    pub fn blink_rapidly(mut self, enabled: bool) -> Self {
        self.options.set(constants::BLINK_RAPIDLY, enabled);
        self
    }

    pub fn reverse(mut self, enabled: bool) -> Self {
        self.options.set(constants::REVERSE, enabled);
        self
    }

    pub fn hidden(mut self, enabled: bool) -> Self {
        self.options.set(constants::HIDDEN, enabled);
        self
    }

    pub fn strike_through(mut self, enabled: bool) -> Self {
        self.options.set(constants::STRIKE_THROUGH, enabled);
        self
    }
}

impl AnsiStyle {
    pub fn new() -> Self {
        Self {
            fore: None,
            back: None,
            font_style: FontStyle::new(),
        }
    }

    #[inline(always)]
    pub fn new_vanilla() -> Self {
        Self::new()
    }

    pub fn merge_style(mut self, other: FontStyle) -> Self {
        self.font_style = self.font_style.merge(other);
        self
    }

    pub fn with_fore(mut self, color: AnsiColor) -> Self {
        self.fore = Some(color);
        self
    }

    pub fn with_back(mut self, color: AnsiColor) -> Self {
        self.back = Some(color);
        self
    }

    pub fn is_vanilla(self) -> bool {
        self.fore.is_none() && self.back.is_none()
    }

    pub fn decorate<'a>(self, content: &'a str) -> AnsiString<'a> {
        AnsiString {
            style: self,
            is_vanilla: self.is_vanilla(),
            content,
        }
    }
}

#[allow(dead_code)]
impl AnsiStyle {
    pub fn bold(mut self, enabled: bool) -> Self {
        self.font_style = self.font_style.bold(enabled);
        self
    }

    pub fn dimmed(mut self, enabled: bool) -> Self {
        self.font_style = self.font_style.dimmed(enabled);
        self
    }

    pub fn italic(mut self, enabled: bool) -> Self {
        self.font_style = self.font_style.italic(enabled);
        self
    }

    pub fn underline(mut self, enabled: bool) -> Self {
        self.font_style = self.font_style.underline(enabled);
        self
    }

    pub fn blink_slowly(mut self, enabled: bool) -> Self {
        self.font_style = self.font_style.blink_slowly(enabled);
        self
    }

    pub fn blink_rapidly(mut self, enabled: bool) -> Self {
        self.font_style = self.font_style.blink_rapidly(enabled);
        self
    }

    pub fn reverse(mut self, enabled: bool) -> Self {
        self.font_style = self.font_style.reverse(enabled);
        self
    }

    pub fn hidden(mut self, enabled: bool) -> Self {
        self.font_style = self.font_style.hidden(enabled);
        self
    }

    pub fn strike_through(mut self, enabled: bool) -> Self {
        self.font_style = self.font_style.strike_through(enabled);
        self
    }
}

impl Display for AnsiStyle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_vanilla() {
            Ok(())
        } else {
            f.write_str(ESCAPE_BEGIN)?;

            for code in 0..16usize {
                if self.font_style.enabled(code) {
                    f.write_fmt(format_args!(";{code}"))?;
                }
            }

            if self.fore.is_some() {
                f.write_fmt(format_args!(";{}", self.fore.unwrap().into_fore()))?;
            }

            if self.back.is_some() {
                f.write_fmt(format_args!(";{}", self.back.unwrap().into_back()))?;
            }

            f.write_str(ESCAPE_OVER)
        }
    }
}
