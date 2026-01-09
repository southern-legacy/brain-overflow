#![allow(dead_code)]
use std::sync::LazyLock;

use regex::Regex;
use serde_json::json;

use crate::error::{
    CustomError,
    api::{ApiError, ApiErrorKind},
};

pub struct Email(String);
pub struct Phone(String);
pub struct Password(String);

pub struct SignUpParam {
    name: String,
    email: Option<Email>,
    phone: Option<Phone>,
    password: Password,
}

pub static EMAIL_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[\w._%+-]+@[\w.-]+\.\w{2,}$").unwrap());
pub static PHONE_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^\+\d{1,15}$").unwrap());

impl TryFrom<&str> for Email {
    type Error = ApiError;

    fn try_from(email: &str) -> Result<Self, Self::Error> {
        if EMAIL_REGEX.captures(email).is_some() {
            Ok(Email(email.to_string()))
        } else {
            Err(ApiError::new(ApiErrorKind::BadRequest).with_context(json!(null)))
        }
    }
}

impl TryFrom<&str> for Phone {
    type Error = ApiError;

    fn try_from(phone: &str) -> Result<Self, Self::Error> {
        if PHONE_REGEX.captures(phone).is_some() {
            Ok(Phone(phone.to_string()))
        } else {
            Err(ApiError::new(ApiErrorKind::BadRequest).with_context(json!(null)))
        }
    }
}

impl TryFrom<&str> for Password {
    type Error = ApiError;

    fn try_from(password: &str) -> Result<Self, Self::Error> {
        let mut alphas = 0;
        let mut numerics = 0;
        let mut specials = 0;
        for c in password.chars() {
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

        if alphas + numerics + specials < 8 {
            Err(ApiError::new(ApiErrorKind::BadRequest).with_context("password too short"))
        } else if count < 2 {
            Err(ApiError::new(ApiErrorKind::BadRequest).with_context("password too simple"))
        } else {
            Ok(Password(password.to_string()))
        }
    }
}
