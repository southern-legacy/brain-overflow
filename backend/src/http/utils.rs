use regex::Regex;
use std::sync::LazyLock;

pub const EMAIL_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[\w._%+-]+@[\w.-]+\.\w{2,}$").unwrap());
pub const PHONE_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^\+\d{1,15}$").unwrap());

pub fn meet_email_format(email: &str) -> bool {
    EMAIL_REGEX.captures(email).is_some()
}

pub fn meet_phone_format(phone: &str) -> bool {
    PHONE_REGEX.captures(phone).is_some()
}
