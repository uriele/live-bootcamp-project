pub struct User {
    email: String,
    password: String,
    requires_2fa: bool,
    // Add other fields as necessary
}

impl User {
    pub fn new(email: String, password: String, requires_2fa: bool) -> Self {
        Self { email, password, requires_2fa }
    }
    
    pub fn new_with_2fa(email: String, password: String) -> Self {
        Self { email, password, requires_2fa: true }
    }

    pub fn new_without_2fa(email: String, password: String) -> Self {
        Self { email, password, requires_2fa: false }
    }
}