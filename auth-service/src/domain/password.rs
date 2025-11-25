use color_eyre::eyre::{eyre,Result};
use secrecy::{ExposeSecret, Secret};

#[derive(Clone, Debug)]
pub struct Password(Secret<String>);


impl PartialEq for Password {
    fn eq(&self, other: &Self) -> bool {
        self.0.expose_secret() == other.0.expose_secret()
    }
}


impl AsRef<Secret<String>> for Password {
    fn as_ref(&self) -> &Secret<String> {
        &self.0
    }
}


impl From<Secret<String>> for Password {
    fn from(s: Secret<String>) -> Self {
        Password(s)
    }
}


pub fn validate_password(password: &Secret<String>) -> bool {
    let password_regex =
        fancy_regex::Regex::new(r"^(?!.*\s)(?=.*[0-9])(?=.*[!@#$%^&*])(?=.{8,})").unwrap();
    password_regex.is_match(password.expose_secret()).unwrap_or(false)
}


impl Password {
    pub fn parse(password: Secret<String>) -> Result<Self> {
        if validate_password(&password) {
            Ok(Password(password))
        } else {
            Err(eyre!("Failed to parse string to a Passwrord type"))
        }
    }
    // dummy workaround for hashed_passwords
    pub fn hashed(password: Secret<String>) -> Result<Self> {
        let password_regex = fancy_regex::Regex::new(r"^(?=.{8,})").unwrap();
        if password_regex.is_match(password.expose_secret())? {
            Ok(Password(password))
        } else {
            Err(eyre!("Hashed Password is too short"))
        }
    }
}


#[cfg(test)]
mod tests {
    use super::{Password,validate_password};

    use secrecy::Secret; // New!
    use rand::{seq::SliceRandom, Rng};

    #[test]
    fn empty_string_is_rejected() {
        let password = Secret::new("".to_string()); // Updated!
        assert!(Password::parse(password).is_err());
    }
    #[test]
    fn string_less_than_8_characters_is_rejected() {
        let password = Secret::new("1234567".to_string()); // Updated!
        assert!(Password::parse(password).is_err());
    }

    #[derive(Debug, Clone)]
    struct ValidPasswordFixture(pub Secret<String>); // Updated!

    impl ValidPasswordFixture {
        fn arbitrary<G: rand::Rng>(g: &mut G) -> Self {
            let password = generate_valid_password(g);
            Self(Secret::new(password)) // Updated!
        }
    }
    #[test]
    fn valid_passwords_are_parsed_successfully(){
        let mut rng = rand::thread_rng();
        for _ in 0..100 {
            let valid_password = ValidPasswordFixture::arbitrary(&mut rng);
            assert!(validate_password(&valid_password.0));
        }
    }

    fn generate_valid_password<R: Rng + ?Sized>(rng: &mut R) -> String {
        const SPECIAL_CHARS: &[u8] = b"!@#$%^&*";
        const CHARSET: &[u8] =
            b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789!@#$%^&*";

        let len = rng.gen_range(8..=30);
        let mut bytes = Vec::with_capacity(len);

        // Ensure required character classes are present
        bytes.push(*SPECIAL_CHARS.choose(rng).expect("special chars not empty"));
        bytes.push(b'0' + rng.gen_range(0..10));

        while bytes.len() < len {
            bytes.push(*CHARSET.choose(rng).expect("charset not empty"));
        }

        bytes.shuffle(rng);
        String::from_utf8(bytes).expect("generated password should be valid UTF-8")
    }
}
