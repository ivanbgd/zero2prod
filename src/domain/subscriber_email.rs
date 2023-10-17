use validator::validate_email;

#[derive(Debug)]
pub struct SubscriberEmail(String);

/// This is a tuple-struct with a single private anonymous `String` field.
/// We purposefully don't want to make the field public.
impl SubscriberEmail {
    /// Checks validity of a new user's email
    ///
    /// Returns an instance of `SubscriberEmail` if **ALL** input validation constraints
    /// are satisfied on subscriber email;
    /// `Err<String>` otherwise.
    pub fn parse(email: String) -> Result<SubscriberEmail, String> {
        if validate_email(&email) {
            Ok(SubscriberEmail(email))
        } else {
            Err(format!(r#""{}" is not a valid subscriber email."#, email))
        }
    }
}

impl AsRef<str> for SubscriberEmail {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

/// We shouldn't (have to) test a third-party library,
/// but we are doing it here for the sake of exercise and example.
/// To be fair, we are testing our own `parse` function, and not
/// an external function - at least not directly.
/// So, this may make sense to some extent, after all.
#[cfg(test)]
mod tests {
    use super::SubscriberEmail;

    use claims::{assert_err, assert_ok};
    use fake::faker::internet::en::SafeEmail;
    use fake::Fake;
    use rstest::rstest;

    /// Asserting SUCCESS in three different ways
    #[test]
    fn parse_accepts_valid_email_hard_coded() {
        assert_ok!(SubscriberEmail::parse(String::from("john.doe@domain.yq")));

        assert!(SubscriberEmail::parse(String::from("john.doe@domain.yq")).is_ok());

        assert_eq!(
            String::from("john.doe@domain.yq"),
            SubscriberEmail::parse(String::from("john.doe@domain.yq"))
                .unwrap()
                .as_ref()
        );
    }

    #[test]
    fn parse_accepts_valid_email_using_fake() {
        assert_ok!(SubscriberEmail::parse(SafeEmail().fake()));
    }

    /// Asserting FAILURE in two different ways
    #[rstest(
        email,
        case::empty(""),
        case::single_whitespace(" "),
        case::whitespace(" \t \r \n   "),
        case::contains_whitespace_in_subject("john doe@domain.yq"),
        case::contains_whitespace_in_domain("john_doe@dom ain.yq"),
        case::missing_at_symbol("john.doeATdomain.yq"),
        case::missing_subject("@domain.yq"),
        case::missing_domain("john.doe@")
    )]
    fn parse_rejects_invalid_emails(email: &str) {
        assert_err!(SubscriberEmail::parse(email.to_string()));

        assert!(SubscriberEmail::parse(email.to_string()).is_err());
    }
}
