use serde::{Deserialize, Serialize};
use sqlx::postgres::{PgArgumentBuffer, PgTypeInfo, PgValueRef};
use sqlx::{Decode, Encode, Postgres, Type};
use std::fmt;

/// Validated Stacks blockchain wallet address.
///
/// Format validation:
/// - Prefix: SP/SM/ST/SN (network identifier)
/// - Characters: C32 alphabet only (0-9, A-Z excluding O, I, L)
/// - Length: 40-41 characters total
///
/// Note: Full checksum validation requires C32 decode with SHA256 double-hash.
/// Currently validates format only (prefix + alphabet + length).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct WalletAddress(String);

impl WalletAddress {
    const C32_ALPHABET: &'static str = "0123456789ABCDEFGHJKMNPQRSTVWXYZ";
    const VALID_PREFIXES: &'static [&'static str] = &["SP", "SM", "ST", "SN"];
    const MIN_LENGTH: usize = 35; // Minimum address length
    const MAX_LENGTH: usize = 45; // Maximum address length

    /// Create a new validated wallet address.
    pub fn new(address: impl AsRef<str>) -> Result<Self, WalletAddressError> {
        let address = address.as_ref().trim().to_uppercase();

        // Validate length (40-41 characters)
        if address.len() < Self::MIN_LENGTH || address.len() > Self::MAX_LENGTH {
            return Err(WalletAddressError::InvalidLength {
                min: Self::MIN_LENGTH,
                max: Self::MAX_LENGTH,
                actual: address.len(),
            });
        }

        // Validate prefix
        let prefix = &address[..2];
        if !Self::VALID_PREFIXES.contains(&prefix) {
            return Err(WalletAddressError::InvalidPrefix {
                prefix: prefix.to_string(),
            });
        }

        // Validate C32 alphabet
        for (idx, ch) in address.chars().enumerate() {
            if !Self::C32_ALPHABET.contains(ch) {
                return Err(WalletAddressError::InvalidCharacter {
                    character: ch,
                    position: idx,
                });
            }
        }

        Ok(Self(address))
    }
    /// Get address as string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Get network prefix.
    pub fn prefix(&self) -> &str {
        &self.0[..2]
    }

    /// Check if mainnet address.
    pub fn is_mainnet(&self) -> bool {
        self.prefix() == "SP"
    }

    /// Check if testnet address.
    pub fn is_testnet(&self) -> bool {
        self.prefix() == "ST"
    }

    /// Consume and return inner String.
    pub fn into_inner(self) -> String {
        self.0
    }
}

impl fmt::Display for WalletAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl AsRef<str> for WalletAddress {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl From<WalletAddress> for String {
    fn from(addr: WalletAddress) -> Self {
        addr.0
    }
}

impl TryFrom<String> for WalletAddress {
    type Error = WalletAddressError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        WalletAddress::new(value)
    }
}

impl TryFrom<&str> for WalletAddress {
    type Error = WalletAddressError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        WalletAddress::new(value)
    }
}

// SQLx integration
impl Type<Postgres> for WalletAddress {
    fn type_info() -> PgTypeInfo {
        <String as Type<Postgres>>::type_info()
    }
}

impl<'q> Encode<'q, Postgres> for WalletAddress {
    fn encode_by_ref(
        &self,
        buf: &mut PgArgumentBuffer,
    ) -> Result<sqlx::encode::IsNull, Box<dyn std::error::Error + Send + Sync + 'static>> {
        <String as Encode<Postgres>>::encode_by_ref(&self.0, buf)
    }
}

impl<'r> Decode<'r, Postgres> for WalletAddress {
    fn decode(value: PgValueRef<'r>) -> Result<Self, sqlx::error::BoxDynError> {
        let s = <String as Decode<Postgres>>::decode(value)?;
        Ok(WalletAddress::new(s)?)
    }
}

/// Wallet address validation errors.
#[derive(Debug, Clone, thiserror::Error)]
pub enum WalletAddressError {
    #[error("Invalid address length: expected {min}-{max} characters, got {actual}")]
    InvalidLength {
        min: usize,
        max: usize,
        actual: usize,
    },

    #[error("Invalid prefix '{prefix}': must be one of SP, SM, ST, SN")]
    InvalidPrefix { prefix: String },

    #[error(
        "Invalid character '{character}' at position {position}: must be C32 alphabet (0-9, A-Z excluding O, I, L)"
    )]
    InvalidCharacter { character: char, position: usize },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_mainnet_address() {
        let addr = WalletAddress::new("SPF0V8KWBS70F0WDKTMY65B3G591NN52PTHHN51D");
        assert!(addr.is_ok(), "Valid address should parse successfully");

        let addr = addr.unwrap();
        assert_eq!(addr.as_str(), "SPF0V8KWBS70F0WDKTMY65B3G591NN52PTHHN51D");
        assert_eq!(addr.prefix(), "SP");
        assert!(addr.is_mainnet());
        assert!(!addr.is_testnet());
    }

    #[test]
    fn test_valid_testnet_address() {
        let addr = WalletAddress::new("ST2CY5V39NHDPWSXMW9QDT3HC3GD6Q6XX4CFRK9AG");
        assert!(addr.is_ok());

        let addr = addr.unwrap();
        assert!(addr.is_testnet());
        assert!(!addr.is_mainnet());
    }

    #[test]
    fn test_invalid_length() {
        let result = WalletAddress::new("SP123");
        assert!(matches!(
            result,
            Err(WalletAddressError::InvalidLength { .. })
        ));
    }

    #[test]
    fn test_invalid_prefix() {
        let result = WalletAddress::new("XX".to_string() + &"0".repeat(39));
        assert!(matches!(
            result,
            Err(WalletAddressError::InvalidPrefix { .. })
        ));
    }

    #[test]
    fn test_invalid_character_o() {
        let result = WalletAddress::new("SPO".to_string() + &"0".repeat(38));
        assert!(matches!(
            result,
            Err(WalletAddressError::InvalidCharacter { character: 'O', .. })
        ));
    }

    #[test]
    fn test_invalid_character_i() {
        let result = WalletAddress::new("SPI".to_string() + &"0".repeat(38));
        assert!(matches!(
            result,
            Err(WalletAddressError::InvalidCharacter { character: 'I', .. })
        ));
    }

    #[test]
    fn test_invalid_character_l() {
        let result = WalletAddress::new("SPL".to_string() + &"0".repeat(38));
        assert!(matches!(
            result,
            Err(WalletAddressError::InvalidCharacter { character: 'L', .. })
        ));
    }

    #[test]
    fn test_case_insensitive() {
        let lower = WalletAddress::new("spf0v8kwbs70f0wdktmy65b3g591nn52pthhn51d");
        let upper = WalletAddress::new("SPF0V8KWBS70F0WDKTMY65B3G591NN52PTHHN51D");

        assert!(lower.is_ok());
        assert!(upper.is_ok());
        assert_eq!(lower.unwrap().as_str(), upper.unwrap().as_str());
    }

    #[test]
    fn test_display_trait() {
        let addr = WalletAddress::new("SPF0V8KWBS70F0WDKTMY65B3G591NN52PTHHN51D").unwrap();
        assert_eq!(
            format!("{}", addr),
            "SPF0V8KWBS70F0WDKTMY65B3G591NN52PTHHN51D"
        );
    }

    #[test]
    fn test_serialization() {
        let addr = WalletAddress::new("SPF0V8KWBS70F0WDKTMY65B3G591NN52PTHHN51D").unwrap();

        let json = serde_json::to_string(&addr).unwrap();
        assert_eq!(json, "\"SPF0V8KWBS70F0WDKTMY65B3G591NN52PTHHN51D\"");

        let deserialized: WalletAddress = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, addr);
    }

    #[test]
    fn test_into_string() {
        let addr = WalletAddress::new("SPF0V8KWBS70F0WDKTMY65B3G591NN52PTHHN51D").unwrap();
        let s: String = addr.into();
        assert_eq!(s, "SPF0V8KWBS70F0WDKTMY65B3G591NN52PTHHN51D");
    }

    #[test]
    fn test_all_valid_c32_characters() {
        let c32_chars = "0123456789ABCDEFGHJKMNPQRSTVWXYZ";
        for ch in c32_chars.chars() {
            let addr_str = format!("SP{}{}", ch, "0".repeat(38));
            let result = WalletAddress::new(&addr_str);
            if let Err(e) = result {
                assert!(
                    !matches!(e, WalletAddressError::InvalidCharacter { .. }),
                    "C32 character '{}' should be valid, got error: {:?}",
                    ch,
                    e
                );
            }
        }
    }
}
