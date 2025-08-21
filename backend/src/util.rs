use std::fmt::Display;

const RESET: &str = "\x1B[0m";
const ESCAPE_BEGIN: &str = "\x1B[";
const ESCAPE_OVER: &str = "m";

#[derive(Clone, Copy)]
pub struct AnsiStyle {
    fore: Option<AnsiColor>,
    back: Option<AnsiColor>,
}

impl AnsiStyle {
    pub fn new() -> Self {
        Self {
            fore: None,
            back: None,
        }
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

impl Display for AnsiStyle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_vanilla() {
            Ok(())
        } else {
            let mut begined = false;
            if self.fore.is_some() {
                f.write_str(ESCAPE_BEGIN)?;
                begined = true;
                f.write_fmt(format_args!("{}", self.fore.unwrap().into_fore()))?;
            }

            if self.back.is_some() {
                if !begined {
                    f.write_str(ESCAPE_BEGIN)?;
                }
                let _val = self.back.unwrap().into_back();
                f.write_fmt(format_args!(";{}", self.back.unwrap().into_back()))?;
            }

            f.write_str(ESCAPE_OVER)
        }
    }
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

    pub fn with_fore(mut self, color: AnsiColor) -> Self {
        self.style.fore = Some(color);
        self.is_vanilla = false;
        self
    }

    pub fn with_back(mut self, color: AnsiColor) -> Self {
        self.style.back = Some(color);
        self.is_vanilla = false;
        self
    }

    pub fn reset(self) -> Self {
        Self::new(self.content)
    }
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
