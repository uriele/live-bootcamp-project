#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Password (String);


impl AsRef<str> for Password {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl Password {
    pub fn parse(password: String) -> Result<Self, String> {
        let password_regex = fancy_regex::Regex::new(r"^(?!.*\s)(?=.*[0-9])(?=.*[!@#$%^&*])(?=.{8,})").unwrap();
        if password_regex.is_match(&password).unwrap() {
            Ok(Password(password))
        } else {
            Err(format!("Invalid password format"))
        }
    }
}