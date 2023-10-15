use crate::consts::{FORBIDDEN_NAME_CHARACTERS, MAX_NAME_LEN};
use unicode_segmentation::UnicodeSegmentation;

pub struct NewSubscriber {
    pub email: String,
    pub name: SubscriberName,
}

/// This is a tuple-struct with a single private anonymous `String` field.
/// We deliberately don't want to make the field public.
/// Instead, we want to create instances of `SubscriberName` through the
/// `parse` method, which performs input validation of the name, and **only**
/// outputs the name if it is **valid**, and panics if it is not valid
/// according to our constraints.
///
/// So, whenever we want to create a `SubscriberName`, validation will be
/// performed automatically for us, and therefore we **cannot forget** to do it.
/// We are leveraging the Rust's type system to eliminate the possibility of
/// forgetting to validate the input whenever/wherever the validation should be done.
/// An incorrect usage pattern is simply impossible, because it will
/// not compile, thanks to the Rust compiler.
/// This is known as "type-driven development".
///
/// This further means that we can rest assured that *all* instances of
/// `SubscriberName` satisfy *all* our validation constraints.
/// We have effectively made it *impossible* for an instance of
/// `SubscriberName` to violate those constraints.
pub struct SubscriberName(String);

impl SubscriberName {
    /// Checks validity of a new user's name
    ///
    /// Returns an instance of `SubscriberName` if **ALL** input validation constraints
    /// are satisfied on subscriber name;
    /// # Panics otherwise.
    pub fn parse(name: String) -> SubscriberName {
        if !is_valid_name(&name) {
            panic!("'{}' is not a valid subscriber name.", name)
        } else {
            SubscriberName(name)
        }
    }

    pub fn get_name(&self) -> &str {
        &self.0
    }
}

/// Checks validity of a new user's name
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
        case::punctuation(". , ? ! : ; - _"),
    )]
    fn is_valid_name_passes_valid_names(valid_name: &'static str) {
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
    fn is_valid_name_rejects_invalid_names(invalid_name: &'static str, error_message: &str) {
        let is_valid = is_valid_name(invalid_name);
        assert_eq!(
            false, is_valid,
            "Didn't reject the invalid name '{}' (name is {}).",
            invalid_name, error_message
        );
    }

    #[test]
    fn parse_valid_name() {
        let valid_name = String::from("  \t \n  John-å  \t \n  å_Doe ?! \t \n  ");
        let subscriber_name = SubscriberName::parse(valid_name.clone());
        assert_eq!(
            valid_name, subscriber_name.0,
            "Rejected a valid name '{}'.",
            valid_name
        );
    }

    #[test]
    #[should_panic]
    fn parse_panics_on_invalid_name() {
        let invalid_name = String::from("");
        let _subscriber_name = SubscriberName::parse(invalid_name);
    }
}
