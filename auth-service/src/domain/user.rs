use crate::domain::email::Email;
use crate::domain::password::Password;
#[derive(Clone, Debug, PartialEq)]
pub struct User {
    pub email: Email,
    pub password: Password,
    pub requires_2fa: bool,
    // Add other fields as necessary
}


impl User {
    pub fn new(email: Email, password: Password, requires_2fa: bool) -> Self {
        Self { email, password, requires_2fa }
    }

    pub fn new_with_2fa(email: Email, password: Password) -> Self {
        Self::new(email, password, true)
    }

    pub fn new_without_2fa(email: Email, password: Password) -> Self {
        Self::new(email, password, false)
    }
}