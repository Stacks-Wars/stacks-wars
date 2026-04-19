use serde::{Deserialize, Serialize};
use sqlx::postgres::{PgArgumentBuffer, PgTypeInfo, PgValueRef};
use sqlx::{Decode, Encode, Postgres, Type};
use std::fmt;

/// Validated username.
///
/// Constraints:
/// - Length: 3-20 characters
/// - Characters: alphanumeric (a-z, A-Z, 0-9) and underscore (_) only
/// - Case-insensitive storage (handled by DB CITEXT)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct Username(String);

impl Username {
    const MIN_LENGTH: usize = 3;
    const MAX_LENGTH: usize = 20;

    /// Create a new validated username.
    pub fn new(username: impl AsRef<str>) -> Result<Self, UsernameError> {
        let username = username.as_ref().trim();

        // Validate length
        if username.len() < Self::MIN_LENGTH || username.len() > Self::MAX_LENGTH {
            return Err(UsernameError::InvalidLength {
                min: Self::MIN_LENGTH,
                max: Self::MAX_LENGTH,
                actual: username.len(),
            });
        }

        // Validate characters (alphanumeric + underscore only)
        for (idx, ch) in username.chars().enumerate() {
            if !ch.is_ascii_alphanumeric() && ch != '_' {
                return Err(UsernameError::InvalidCharacter {
                    character: ch,
                    position: idx,
                });
            }
        }

        Ok(Self(username.to_string()))
    }

    /// Get username as string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Consume and return inner String.
    pub fn into_inner(self) -> String {
        self.0
    }
}

impl fmt::Display for Username {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl AsRef<str> for Username {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl From<Username> for String {
    fn from(username: Username) -> Self {
        username.0
    }
}

impl TryFrom<String> for Username {
    type Error = UsernameError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Username::new(value)
    }
}

impl TryFrom<&str> for Username {
    type Error = UsernameError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Username::new(value)
    }
}

// SQLx integration
impl Type<Postgres> for Username {
    fn type_info() -> PgTypeInfo {
        <String as Type<Postgres>>::type_info()
    }
}

impl<'q> Encode<'q, Postgres> for Username {
    fn encode_by_ref(
        &self,
        buf: &mut PgArgumentBuffer,
    ) -> Result<sqlx::encode::IsNull, Box<dyn std::error::Error + Send + Sync + 'static>> {
        <String as Encode<Postgres>>::encode_by_ref(&self.0, buf)
    }
}

impl<'r> Decode<'r, Postgres> for Username {
    fn decode(value: PgValueRef<'r>) -> Result<Self, sqlx::error::BoxDynError> {
        let s = <String as Decode<Postgres>>::decode(value)?;
        Ok(Username::new(s)?)
    }
}

/// Username validation errors.
#[derive(Debug, Clone, thiserror::Error)]
pub enum UsernameError {
    #[error("Invalid username length: expected {min}-{max} characters, got {actual}")]
    InvalidLength {
        min: usize,
        max: usize,
        actual: usize,
    },

    #[error(
        "Invalid character '{character}' at position {position}: username must contain only letters, numbers, and underscores"
    )]
    InvalidCharacter { character: char, position: usize },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_username() {
        let valid = vec![
            "alice",
            "bob123",
            "user_name",
            "Test_User_1",
            "abc",                  // min length
            "a1234567890123456789", // max length (20 chars)
        ];

        for username in valid {
            let result = Username::new(username);
            assert!(
                result.is_ok(),
                "Username '{}' should be valid, got: {:?}",
                username,
                result.err()
            );
        }
    }

    #[test]
    fn test_invalid_length_too_short() {
        let result = Username::new("ab");
        assert!(matches!(
            result,
            Err(UsernameError::InvalidLength {
                min: 3,
                max: 20,
                actual: 2
            })
        ));
    }

    #[test]
    fn test_invalid_length_too_long() {
        let result = Username::new("a12345678901234567890");
        assert!(matches!(
            result,
            Err(UsernameError::InvalidLength {
                min: 3,
                max: 20,
                actual: 21
            })
        ));
    }

    #[test]
    fn test_invalid_characters() {
        let invalid = vec![
            ("user-name", '-'),
            ("user.name", '.'),
            ("user@name", '@'),
            ("user name", ' '),
            ("user#name", '#'),
            ("user$name", '$'),
        ];

        for (username, expected_char) in invalid {
            let result = Username::new(username);
            assert!(
                matches!(
                    result,
                    Err(UsernameError::InvalidCharacter { character, .. }) if character == expected_char
                ),
                "Username '{}' should fail with invalid character '{}', got: {:?}",
                username,
                expected_char,
                result
            );
        }
    }

    #[test]
    fn test_trim_whitespace() {
        let result = Username::new("  alice  ");
        assert!(result.is_ok());
        assert_eq!(result.unwrap().as_str(), "alice");
    }

    #[test]
    fn test_display() {
        let username = Username::new("test_user").unwrap();
        assert_eq!(format!("{}", username), "test_user");
    }

    #[test]
    fn test_serialization() {
        let username = Username::new("test_user").unwrap();
        let json = serde_json::to_string(&username).unwrap();
        assert_eq!(json, "\"test_user\"");

        let deserialized: Username = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, username);
    }

    #[test]
    fn test_into_string() {
        let username = Username::new("test_user").unwrap();
        let s: String = username.into();
        assert_eq!(s, "test_user");
    }
}
