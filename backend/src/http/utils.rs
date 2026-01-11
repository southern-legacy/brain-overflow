use regex::Regex;
use std::{borrow::Cow, sync::LazyLock};
use validator::ValidationError;

pub static EMAIL_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[\w._%+-]+@[\w.-]+\.\w{2,}$").unwrap());
pub static PHONE_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^\+\d{1,15}$").unwrap());

fn meet_email_format(email: &str) -> bool {
    EMAIL_REGEX.captures(email).is_some()
}

fn meet_phone_format(phone: &str) -> bool {
    PHONE_REGEX.captures(phone).is_some()
}

pub fn validate_email(email: &str) -> Result<(), ValidationError> {
    if !meet_email_format(email) {
        let err = ValidationError::new("format")
            .with_message(Cow::Borrowed("email address format error"));
        Err(err)
    } else {
        Ok(())
    }
}

pub fn validate_phone(phone: &str) -> Result<(), ValidationError> {
    if !meet_phone_format(phone) {
        let err =
            ValidationError::new("format").with_message(Cow::Borrowed("phone number format error"));
        Err(err)
    } else {
        Ok(())
    }
}

pub fn validate_password_length(val: &str) -> Result<(), ValidationError> {
    if val.chars().count() < 8 {
        Err(ValidationError::new("password").with_message(Cow::Borrowed("Password is too short!")))
    } else {
        Ok(())
    }
}

pub fn validate_password_complexity(val: &str) -> Result<(), ValidationError> {
    let mut alphas = 0;
    let mut numerics = 0;
    let mut specials = 0;
    for c in val.chars() {
        if c.is_alphabetic() {
            alphas += 1;
        } else if c.is_numeric() {
            numerics += 1;
        } else {
            specials += 1;
        }
    }

    let count = |val| match val {
        1.. => 1,
        _ => 0,
    };

    let count = count(alphas) + count(numerics) + count(specials);

    if count < 2 {
        Err(ValidationError::new("password").with_message(Cow::Borrowed("Password is too simple!")))
    } else {
        Ok(())
    }
}
