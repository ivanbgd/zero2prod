use crate::consts::{FORBIDDEN_NAME_CHARACTERS, MAX_NAME_LEN};
use unicode_segmentation::UnicodeSegmentation;

pub struct SubscriberName(String);

impl SubscriberName {
    /// Checks validity of a new user's name
    ///
    /// Returns an instance of `SubscriberName` if **ALL** input validation constraints are satisfied
    /// on subscriber names;
    /// # Panics otherwise.
    pub fn parse(name: String) -> Self {
        let is_empty_or_whitespace = name.trim().is_empty();

        let is_too_long = name.graphemes(true).count() > MAX_NAME_LEN;

        let contains_a_forbidden_character =
            name.chars().any(|c| FORBIDDEN_NAME_CHARACTERS.contains(&c));

        if is_empty_or_whitespace || is_too_long || contains_a_forbidden_character {
            panic!("'{}' is not a valid subscriber name.", name)
        } else {
            Self(name)
        }
    }
}

/// Checks validity of a new user's name
///
/// **THIS IS MORE OF A PRACTICE AND TO SHOW HOW A SINGLE VALIDATION COULD**
/// **BE PERFORMED AT ANY POINT, BUT WE ARE GIVING UP ON THIS AS WE DON'T WANT**
/// **TO RELY ON HAVING TO REMEMBER TO PERFORM VALIDATION AT VARIOUS POINTS**
/// **IN TIME AND CODE. STILL, A UNIT TEST SUITE IS PROVIDED BELOW.**
///
/// **WE ARE INSTEAD USING A PARSING FUNCTION, WHICH RETURNS A STRUCTURED OUTPUT.**
///
/// Returns `true` if **ALL** input validation constraints are satisfied,
/// `false` otherwise.
fn is_valid_name(name: &str) -> bool {
    let is_empty_or_whitespace = name.trim().is_empty();

    let is_too_long = name.graphemes(true).count() > MAX_NAME_LEN;

    let contains_a_forbidden_character =
        name.chars().any(|c| FORBIDDEN_NAME_CHARACTERS.contains(&c));

    !(is_empty_or_whitespace || is_too_long || contains_a_forbidden_character)
}

#[cfg(test)]
mod tests {
    use super::*;

    use once_cell::sync::Lazy;
    use rstest::rstest;

    static VALID_MAX_LONG_NAME: Lazy<String> = Lazy::new(|| "å".repeat(MAX_NAME_LEN));
    static TOO_LONG_NAME: Lazy<String> = Lazy::new(|| "a".repeat(MAX_NAME_LEN + 1));

    #[rstest(
    valid_name,
    case::first_name("John"),
    case::first_last("John Doe"),
    case::first_last_whitespace("  \t \n  John  \t \n  Doe \t \n  "),
    case::non_ascii("å"),
    case::non_ascii_max_long(&VALID_MAX_LONG_NAME),
    case::punctuation(". , ? ! : ;"),
    )]
    fn _is_valid_name_passes_valid_names(valid_name: &'static str) {
        let is_valid = is_valid_name(valid_name);
        assert_eq!(true, is_valid, "Rejected a valid name '{}'.", valid_name);
    }

    #[rstest(
    invalid_name,
    error_message,
    case::empty_name("", "empty"),
    case::whitespace_name(" \t \r \n   ", "whitespace"),
    case::too_long(&TOO_LONG_NAME, "too long"),
    case::forward_slash("/", "forward slash"),
    case::open_parenthesis("(", "open parenthesis"),
    case::close_parenthesis(")", "close parenthesis"),
    case::double_quote(r#"""#, "double quote"),
    case::open_angle_bracket("<", "open angle bracket"),
    case::close_angle_bracket(">", "close angle bracket"),
    case::back_slash("\\", "back slash"),
    case::open_curly_brace("{", "open curly brace"),
    case::close_curly_brace("}", "close curly brace"),
    )]
    fn _is_valid_name_rejects_invalid_names(invalid_name: &'static str, error_message: &str) {
        let is_valid = is_valid_name(invalid_name);
        assert_eq!(
            false, is_valid,
            "Didn't reject the invalid name '{}' (name is {}).",
            invalid_name, error_message
        );
    }
}
