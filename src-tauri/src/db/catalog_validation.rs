use crate::error::{AppError, ErrorCode};
pub const MAX_SAFE_MINOR: i64 = 9_007_199_254_740_991;
pub fn invalid(message: &str) -> AppError {
    AppError::new(ErrorCode::Validation, message)
}
/// Display values are never normalized. Only ASCII case and Unicode whitespace change in matching keys.
pub fn normalize(value: &str) -> String {
    value
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_ascii_lowercase()
}
pub fn required(value: &str) -> Result<(), AppError> {
    if normalize(value).is_empty() || value.chars().count() > 4000 || value.contains('\0') {
        Err(invalid(
            "A non-empty value of at most 4000 characters is required.",
        ))
    } else {
        Ok(())
    }
}
pub fn optional(value: &Option<String>) -> Result<(), AppError> {
    if let Some(value) = value {
        required(value)?;
    }
    Ok(())
}
pub fn id(value: &str) -> Result<(), AppError> {
    uuid::Uuid::parse_str(value)
        .map(|_| ())
        .map_err(|_| invalid("A valid UUID is required."))
}
/// Strict decimal text conversion: no floats, exponent syntax, rounding or guessed separators.
pub fn egp_to_minor(value: &str) -> Result<i64, AppError> {
    let parts: Vec<_> = value.trim().split('.').collect();
    if parts.is_empty()
        || parts.len() > 2
        || parts[0].is_empty()
        || !parts[0].bytes().all(|b| b.is_ascii_digit())
    {
        return Err(invalid("Use a non-negative decimal EGP amount."));
    }
    let fraction = if parts.len() == 2 { parts[1] } else { "" };
    if (parts.len() == 2 && fraction.is_empty())
        || fraction.len() > 2
        || !fraction.bytes().all(|b| b.is_ascii_digit())
    {
        return Err(invalid("EGP supports at most two decimal places."));
    }
    let whole = parts[0]
        .parse::<i64>()
        .map_err(|_| invalid("Amount is too large."))?;
    let cents = match fraction.len() {
        0 => 0,
        1 => i64::from(fraction.as_bytes()[0] - b'0') * 10,
        _ => {
            i64::from(fraction.as_bytes()[0] - b'0') * 10 + i64::from(fraction.as_bytes()[1] - b'0')
        }
    };
    whole
        .checked_mul(100)
        .and_then(|v| v.checked_add(cents))
        .filter(|v| *v <= MAX_SAFE_MINOR)
        .ok_or_else(|| invalid("Amount is too large."))
}
