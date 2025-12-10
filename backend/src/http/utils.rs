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
        let err = ValidationError::new("format").with_message(Cow::Borrowed(""));
        Err(err)
    } else {
        Ok(())
    }
}

pub fn validate_phone(phone: &str) -> Result<(), ValidationError> {
    if !meet_phone_format(phone) {
        let err = ValidationError::new("format");
        Err(err)
    } else {
        Ok(())
    }
}

/// ## 验证密码复杂度
///
/// 三种字符，字母、数字、特殊字符，此函数将统计字母数、数字数、特殊字符数
///
/// 每种字符如果总数大于2，将被统计进字符种类数，如密码 "01234567891a" 就只算**一种字符**，因为只有**一个字母 'a'**
///
/// 通过校验需要满足两个条件：
///
/// - 这三种字符中必须有**两种以上**
/// - 密码整体长度大于等于 12
///
/// 同时注意：密码使用 Unicode 字符集，所以基本所有的字符都能作为密码的一部分
///
pub fn validate_passwd(val: &str) -> Result<(), ValidationError> {
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

    if alphas + numerics + specials < 8 {
        Err(ValidationError::new("password").with_message(Cow::Borrowed("Password is too short!")))
    } else if count < 2 {
        Err(ValidationError::new("password").with_message(Cow::Borrowed("Password is too simple!")))
    } else {
        Ok(())
    }
}
