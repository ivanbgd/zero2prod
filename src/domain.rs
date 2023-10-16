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
#[derive(Debug)]
pub struct SubscriberName(String);

impl SubscriberName {
    /// Checks validity of a new user's name
    ///
    /// Returns an instance of `SubscriberName` if **ALL** input validation constraints
    /// are satisfied on subscriber name;
    /// # Panics otherwise.
    pub fn parse(name: String) -> Result<SubscriberName, String> {
        if !is_valid_name(&name) {
            panic!(r#""{}" is not a valid subscriber name."#, name)
            // Err(format!("'{}' is not a valid subscriber name.", name))
        } else {
            Ok(SubscriberName(name))
        }
    }
}

impl AsRef<str> for SubscriberName {
    /// Get the inner value of `SubscriberName`, which is a `String`
    /// that holds the subscriber's name; gets it as `&str`
    fn as_ref(&self) -> &str {
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

    use claims::{assert_err, assert_ok};
    use once_cell::sync::Lazy;
    use rstest::rstest;

    static VALID_MAX_LONG_NAME: Lazy<String> = Lazy::new(|| "å".repeat(MAX_NAME_LEN));
    static TOO_LONG_NAME: Lazy<String> = Lazy::new(|| "a".repeat(MAX_NAME_LEN + 1));

    /// Test `is_valid_name` with valid names (positive test cases) - Implementation 1
    ///
    /// An example of the use of `case`.
    /// These test cases can be and are named in definition and in the stdout, where they are additionally numbered.
    /// But, we don't have to name them. We can name some of them.
    /// The `'static` lifetime for `valid_name` is not required.
    #[rstest(
        valid_name,
        case::first_name("John"),
        case::first_last("John Doe"),
        case::first_last_whitespace("  \t \n  John  \t \n  Doe \t \n  "),
        case::non_ascii("å"),
        case::non_ascii_max_long(&VALID_MAX_LONG_NAME),
        case::punctuation(". , ? ! : ; - _"),
    )]
    fn is_valid_name_passes_valid_names_cases_a(valid_name: &'static str) {
        let is_valid = is_valid_name(valid_name);
        assert_eq!(true, is_valid, "Rejected a valid name '{}'.", valid_name);
    }

    /// Test `is_valid_name` with valid names (positive test cases) - Implementation 2
    ///
    /// An example of an alternative use of `case`.
    /// These test cases don't have to be named in definition, and consequently won't be named in the stdout,
    /// but they will be automatically numbered.
    /// But, we can name some or all of them if we want to.
    #[rstest]
    #[case("John")]
    #[case::first_last("John Doe")]
    #[case("  \t \n  John  \t \n  Doe \t \n  ")]
    #[case("å")]
    #[case("&VALID_MAX_LONG_NAME")]
    #[case::punctuation(". , ? ! : ; - _")]
    fn is_valid_name_passes_valid_names_cases_b(#[case] valid_name: &str) {
        let is_valid = is_valid_name(valid_name);
        assert_eq!(true, is_valid, "Rejected a valid name '{}'.", valid_name);
    }

    /// Test `is_valid_name` with valid names (positive test cases) - Implementation 3
    ///
    /// This is yet another alternative, but this time using a `values` list instead of `case`.
    /// These tests are numbered and mostly named. Names are generated automatically for us in the output.
    /// The punctuation case is not named.
    #[rstest]
    fn is_valid_name_passes_valid_names_values_list(
        #[values("John", "John Doe", "  \t \n  John  \t \n  Doe \t \n  ", "å", &VALID_MAX_LONG_NAME, ". , ? ! : ; - _")]
        valid_name: &str,
    ) {
        let is_valid = is_valid_name(valid_name);
        assert_eq!(true, is_valid, "Rejected a valid name '{}'.", valid_name);
    }

    /// Test `is_valid_name` with invalid names (negative test cases) - Implementation 1
    ///
    /// These tests are numbered and can be named.
    #[rstest(
        invalid_name,
        error_message,
        case::empty_name("", "empty"),
        case::single_whitespace_name(" ", "an empty space"),
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
    fn is_valid_name_rejects_invalid_names_case_a(invalid_name: &str, error_message: &str) {
        let is_valid = is_valid_name(invalid_name);
        assert_eq!(
            false, is_valid,
            "Didn't reject the invalid name '{}' (name is {}).",
            invalid_name, error_message
        );
    }

    /// Test `is_valid_name` with invalid names (negative test cases) - Implementation 2
    ///
    /// These tests are numbered, and not named by default, but can be named.
    /// The `'static` lifetime for `invalid_name` is not required.
    #[rstest]
    #[case("", "empty")]
    #[case::single_whitespace_name(" ", "an empty space")]
    #[case(" \t \r \n   ", "whitespace")]
    #[case(&TOO_LONG_NAME, "too long")]
    #[case("/", "forward slash")]
    #[case("(", "open parenthesis")]
    #[case(")", "close parenthesis")]
    #[case::double_quote(r#"""#, "double quote")]
    #[case("<", "open angle bracket")]
    #[case(">", "close angle bracket")]
    #[case("\\", "back slash")]
    #[case("{", "open curly brace")]
    #[case("}", "close curly brace")]
    fn is_valid_name_rejects_invalid_names_case_b(
        #[case] invalid_name: &'static str,
        #[case] error_message: &str,
    ) {
        let is_valid = is_valid_name(invalid_name);
        assert_eq!(
            false, is_valid,
            "Didn't reject the invalid name '{}' (name is {}).",
            invalid_name, error_message
        );
    }

    /// Test `is_valid_name` with invalid names (negative test cases) - Implementation 3
    ///
    /// These tests are numbered, but not automatically named in these particular cases.
    /// Namely, these concrete characters don't translate to proper test names.
    /// But, letters or strings would be converted to names.
    #[rstest]
    fn is_valid_name_rejects_invalid_names_values_list_direct(
        #[values('/', '(', ')', '"', '<', '>', '\\', '{', '}')] invalid_name: char,
    ) {
        let is_valid = is_valid_name(invalid_name.to_string().as_str());
        assert_eq!(
            false, is_valid,
            "Didn't reject the invalid name '{}'.",
            invalid_name
        );
    }

    #[test]
    fn parse_valid_name() {
        let valid_name = String::from("  \t \n  John-å  \t \n  å_Doe ?! \t \n  ");
        let subscriber_name = SubscriberName::parse(valid_name.clone()).unwrap();
        assert_eq!(
            valid_name,
            subscriber_name.as_ref(),
            "Rejected a valid name '{}'.",
            valid_name
        );
        assert_ok!(
            SubscriberName::parse(valid_name.clone()),
            "Rejected a valid name '{}'.",
            valid_name
        );
    }

    #[test]
    #[should_panic]
    fn parse_rejects_empty_name() {
        let invalid_name = String::from("");
        assert_err!(
            SubscriberName::parse(invalid_name.clone()),
            "Didn't reject the invalid name '{}'.",
            invalid_name
        );
    }

    /// Non-parameterized, so this is considered only a single test.
    /// This means that we only have a single test name for all test cases inside it.
    /// But, we are still able to use a different, customized, error message for each test case,
    /// and that's what we are doing here. They are customized by the invalid name.
    #[test]
    #[should_panic]
    fn parse_rejects_names_with_invalid_characters() {
        for invalid_name in FORBIDDEN_NAME_CHARACTERS {
            assert_err!(
                SubscriberName::parse(invalid_name.to_string()),
                "Didn't reject the invalid name '{}'.",
                invalid_name
            );
        }
    }
}
