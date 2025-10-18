#[derive(Clone, Debug, PartialEq)]
pub struct User {
    pub email: String,
    pub password: String,
    pub requires_2fa: bool,
    // Add other fields as necessary
}





impl User {
    pub fn new(email: String, password: String, requires_2fa: bool) -> Self {
        let email = email.trim().to_string();
        let password = password.trim().to_string();
        Self { email, password, requires_2fa }
    }
    
    pub fn new_with_2fa(email: String, password: String) -> Self {
        Self::new(email, password, true)
    }

    pub fn new_without_2fa(email: String, password: String) -> Self {
        Self::new(email, password, false)
    }
}