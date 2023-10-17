use validator::validate_email;

/// `SubscriberEmail` either contains a valid email address (`String`),
/// or it yields an error.
///
/// A struct cannot yield an error itself, on its own,
/// but it can have an associated function that can do that for it.
///
/// The point is in keeping the wrapped `String` field **private**, so that we cannot
/// access it directly, but only through the associated `parse` function.
/// The `parse` function will first check for validity of the provided email address,
/// and then decide to either keep it (if valid) or discard it with an error (if invalid).
///
/// The input email field (e.g., from a web form) is parsed (validated) at the very moment
/// it enters our system (e.g., from the web form, for example in the `subscribe` handler),
/// and it is either contained as a valid email address of the `String` type for the rest
/// of execution, or it is immediately discarded at that moment, so it doesn't enter our
/// system as an invalid value, and the user of the `SubscriberEmail::parse` function
/// will be notified of the error and should handle it properly (as desired).
#[derive(Debug)]
pub struct SubscriberEmail(String);

impl SubscriberEmail {
    /// Checks validity of a new user's email
    ///
    /// Returns an instance of `SubscriberEmail` if **ALL** input validation constraints
    /// are satisfied on subscriber email;
    /// `Err<String>` otherwise.
    ///
    /// We are using an external crate named `validator` and its `validate_email`
    /// function to perform email validation for us.
    pub fn parse(email: String) -> Result<SubscriberEmail, String> {
        if validate_email(&email) {
            Ok(SubscriberEmail(email))
        } else {
            Err(format!(r#""{}" is not a valid subscriber email."#, email))
        }
    }
}

/// Needed so we can extract the contained private `String` field.
impl AsRef<str> for SubscriberEmail {
    /// Gets the private inner value of `SubscriberEmail`, which is a `String`
    /// that holds the subscriber's email; gets it as `&str`.
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
    use quickcheck::{Arbitrary, Gen};
    use quickcheck_macros::quickcheck;
    use rstest::rstest;

    /// Represents a **valid** email `String`
    ///
    /// Needed for tests that use `quickcheck`.
    ///
    /// Namely, we can't just use randomly generated `String`s, because they most likely
    /// won't conform to the email address specification, and will thus be rejected
    /// by our email validator, which is the `SubscriberEmail::parse()` function.
    /// This would make our tests fail.
    ///
    /// We need valid randomly-generated email addresses for our tests.
    ///
    /// `SafeEmail().fake()` generates a *single* random valid email address.
    /// We could loop over multiple calls of it to have a more comprehensive test suite
    /// in an effort to catch a potential *edge case*, but a better option is to use a
    /// crate that supports the **property-based** testing.
    ///
    /// We are using `quickcheck` for that reason.
    /// It implements looping over test values automatically.
    /// The number of iterations is configurable, and is 100 by default.
    ///
    /// We have implemented the `quickcheck::Arbitrary` trait for `ValidEmailFixture`.
    /// We have implemented the `Arbitrary::arbitrary()` function in which
    /// we are using `SafeEmail().fake_with_rng(g)`, where `g` is a "generator",
    /// or a source of randomness.
    /// This allows us to combine the `quickcheck` and the `fake` crates,
    /// which lets us have multiple automatically generated random valid email addresses
    /// for our testing purpose.
    /// We wrap `fake` in `quickcheck`.
    /// The `Arbitrary::shrink()` function is optional, as it has a default implementation.
    /// `shrink()` returns a sequence of progressively “smaller” instances of the type
    /// to help `quickcheck` find the smallest possible failure case.
    #[derive(Clone, Debug)]
    struct ValidEmailFixture(pub String);

    impl Arbitrary for ValidEmailFixture {
        /// Returns an instance of `ValidEmailFixture`
        /// given the source of randomness, `g`, which is a "generator".
        /// We can just pass `g` from `Arbitrary::arbitrary()` as the random number generator
        /// for `fake_with_rng()` and everything just works!
        fn arbitrary<G: Gen>(g: &mut G) -> Self {
            Self(SafeEmail().fake_with_rng(g))
        }
    }

    /// Asserting **SUCCESS** in three different ways
    ///
    /// We are using a single hard-coded value of a valid email address.
    ///
    /// We are using three different assertion types for the sake of example.
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

    /// We are using the `fake::faker::internet::en::SafeEmail()` function,
    /// and the `fake::Fake` trait, and its `fake()` function.
    /// `SafeEmail().fake()` generates a *single* random valid email address.
    #[test]
    fn parse_accepts_valid_email_using_fake() {
        assert_ok!(SubscriberEmail::parse(SafeEmail().fake()));
    }

    /// We combine `quickcheck` and `fake` through our `ValidEmailFixture`
    /// and the `quickcheck::Arbitrary` trait, and its `arbitrary()` function.
    ///
    /// The `valid_email` parameter **can't** be a `String`, because, in that case,
    /// `quickcheck` will generate random strings, starting with `""`, and, naturally,
    /// almost none of them will be a valid email address.
    ///
    /// That's why `valid_email` needs to be a `String` wrapped in our type (struct),
    /// which we chose to name `ValidEmailFixture`.
    ///
    /// We have implemented `quickcheck::Arbitrary` for `ValidEmailFixture`,
    /// by implementing the `arbitrary()` function, inside which we use
    /// the `fake::faker::internet::en::SafeEmail()` function and
    /// the `fake::Fake` trait, and its `fake_with_rng()` function.
    ///
    /// The `quickcheck::Arbitrary` trait has another function, `shrink()`, but we haven't
    /// implemented it, as we've decided to use its default implementation which
    /// doesn't shrink the generated data, which is good enough for us in this case.
    /// So, it is optional.
    #[quickcheck]
    fn parse_accepts_valid_email_using_quickcheck(valid_email: ValidEmailFixture) -> bool {
        dbg!(&valid_email);
        SubscriberEmail::parse(valid_email.0).is_ok()
    }

    /// Asserting **FAILURE** in two different ways
    ///
    /// We are using two different assertion types for the sake of example.
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
