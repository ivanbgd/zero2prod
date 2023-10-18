use crate::domain::{SubscriberEmail, SubscriberName};

/// New subscriber
///
/// `email: SubscriberEmail`
///
/// `name: SubscriberName`
pub struct NewSubscriber {
    pub email: SubscriberEmail,
    pub name: SubscriberName,
}
