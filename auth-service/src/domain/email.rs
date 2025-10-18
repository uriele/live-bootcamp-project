#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Email(String);



impl AsRef<str> for Email {
    fn as_ref(&self) -> &str {
        &self.0
    }
}


impl Email {
    pub fn parse(address: String) -> Result<Self, String> {
        // remove leading and trailing whitespaces first
        let address = address.trim().to_string();
        let email_regex = fancy_regex::Regex::new(r"^[\w\.-]+@[\w\.-]+\.\w+$").unwrap();
        if email_regex.is_match(&address).unwrap() {
            Ok(Email(address))
        } else {
            Err(format!("Invalid email format: {}", address))
        }
    }       
}