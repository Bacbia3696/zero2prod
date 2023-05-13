use std::fmt::Display;

use validator::validate_email;

#[derive(Debug, Clone)]
pub struct SubscriberEmail(String);

impl Display for SubscriberEmail {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl AsRef<str> for SubscriberEmail {
    fn as_ref(&self) -> &str {
        &self.0
    }
}
impl SubscriberEmail {
    pub fn parse(email: String) -> Result<Self, String> {
        if validate_email(&email) {
            Ok(Self(email))
        } else {
            Err(format!("{} is invalid email format", email))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use claims::{assert_err, assert_ok};
    use fake::{faker::internet::en::SafeEmail, Fake};

    #[test]
    fn empty_is_rejected() {
        let email = "".to_string();
        assert_err!(SubscriberEmail::parse(email));
    }

    #[test]
    fn email_symbol_missing_is_rejected() {
        let email = "datnguyen.com".to_string();
        assert_err!(SubscriberEmail::parse(email));
    }

    #[test]
    fn missing_object_is_rejected() {
        let email = "@gmail.com".to_string();
        assert_err!(SubscriberEmail::parse(email));
    }

    #[test]
    fn valid_email_is_accepted() {
        let email = SafeEmail().fake();
        assert_ok!(SubscriberEmail::parse(email));
    }

    // TODO: make this work!
    // Both `Clone` and `Debug` are required by `quickcheck`
    #[derive(Debug, Clone)]
    struct ValidEmailFixture(pub String);
    impl quickcheck::Arbitrary for ValidEmailFixture {
        fn arbitrary(_g: &mut quickcheck::Gen) -> Self {
            todo!()
        }
    }

    // #[quickcheck_macros::quickcheck]
    // fn valid_emails_are_parsed_successfully(valid_email: ValidEmailFixture) -> bool {
    //     SubscriberEmail::parse(valid_email.0).is_ok()
    // }
}
