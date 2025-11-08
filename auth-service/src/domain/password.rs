#[derive(Clone, Debug, PartialEq, Eq, Hash)]

#[derive(sqlx::Type)]
#[sqlx(transparent)]
pub struct Password (String);


impl AsRef<str> for Password {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl From<String> for Password {
    fn from(s: String) -> Self {
        Password(s)
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
    // dummy workaround for hashed_passwords
    pub fn hashed(password:String) -> Result<Self,String>{
        let password_regex = fancy_regex::Regex::new(r"^(?=.{8,})").unwrap();
        if password_regex.is_match(&password).unwrap() {
            Ok(Password(password))
        } else {
            Err(format!("Hashed Password is too short"))
        }
    }
}