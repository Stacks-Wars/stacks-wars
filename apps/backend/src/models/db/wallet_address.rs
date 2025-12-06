use serde::{Deserialize, Serialize};
use sqlx::postgres::{PgArgumentBuffer, PgTypeInfo, PgValueRef};
use sqlx::{Decode, Encode, Postgres, Type};
use std::fmt;

/// Validated Stacks blockchain wallet address or contract identifier.
///
/// Supports three formats:
/// 1. Simple address: SP1AY6K3PQV5MRT6R4S671NWW2FRVPKM0BR162CT6
/// 2. Contract identifier: SP1AY6K3PQV5MRT6R4S671NWW2FRVPKM0BR162CT6.leo-token
/// 3. Fully qualified: SP1AY6K3PQV5MRT6R4S671NWW2FRVPKM0BR162CT6.leo-token::LEO
///
/// Format validation:
/// - Prefix: SP/SM/ST/SN (network identifier)
/// - Characters: C32 alphabet only (0-9, A-Z excluding O, I, L)
/// - Length: 35-45 characters for address part
/// - Contract name: alphanumeric, hyphens, underscores (after '.')
/// - Trait/asset: alphanumeric, hyphens, underscores (after '::')
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct WalletAddress(String);

impl WalletAddress {
    const C32_ALPHABET: &'static str = "0123456789ABCDEFGHJKMNPQRSTVWXYZ";
    const VALID_PREFIXES: &'static [&'static str] = &["SP", "SM", "ST", "SN"];
    const MIN_LENGTH: usize = 35; // Minimum address length
    const MAX_LENGTH: usize = 45; // Maximum address length

    /// Create a new validated wallet address or contract identifier.
    pub fn new(address: impl AsRef<str>) -> Result<Self, WalletAddressError> {
        let address = address.as_ref().trim();

        // Split by '::' to separate trait/asset part
        let (main_part, trait_part) = if let Some(idx) = address.find("::") {
            let (main, trait_str) = address.split_at(idx);
            (main, Some(&trait_str[2..])) // Skip '::'
        } else {
            (address, None)
        };

        // Split main part by '.' to separate address and contract name
        let (addr_part, contract_name) = if let Some(idx) = main_part.find('.') {
            let (addr, contract) = main_part.split_at(idx);
            (addr, Some(&contract[1..])) // Skip '.'
        } else {
            (main_part, None)
        };

        // Validate address part
        let addr_upper = addr_part.to_uppercase();

        if addr_upper.len() < Self::MIN_LENGTH || addr_upper.len() > Self::MAX_LENGTH {
            return Err(WalletAddressError::InvalidLength {
                min: Self::MIN_LENGTH,
                max: Self::MAX_LENGTH,
                actual: addr_upper.len(),
            });
        }

        // Validate prefix
        let prefix = &addr_upper[..2];
        if !Self::VALID_PREFIXES.contains(&prefix) {
            return Err(WalletAddressError::InvalidPrefix {
                prefix: prefix.to_string(),
            });
        }

        // Validate C32 alphabet for address
        for (idx, ch) in addr_upper.chars().enumerate() {
            if !Self::C32_ALPHABET.contains(ch) {
                return Err(WalletAddressError::InvalidCharacter {
                    character: ch,
                    position: idx,
                });
            }
        }

        // Validate contract name if present (alphanumeric, hyphens, underscores)
        if let Some(contract) = contract_name {
            if contract.is_empty() {
                return Err(WalletAddressError::InvalidContractName {
                    reason: "Contract name cannot be empty".to_string(),
                });
            }
            for ch in contract.chars() {
                if !ch.is_alphanumeric() && ch != '-' && ch != '_' {
                    return Err(WalletAddressError::InvalidContractName {
                        reason: format!("Invalid character '{}' in contract name", ch),
                    });
                }
            }
        }

        // Validate trait/asset part if present
        if let Some(trait_name) = trait_part {
            if trait_name.is_empty() {
                return Err(WalletAddressError::InvalidTraitName {
                    reason: "Trait/asset name cannot be empty".to_string(),
                });
            }
            for ch in trait_name.chars() {
                if !ch.is_alphanumeric() && ch != '-' && ch != '_' {
                    return Err(WalletAddressError::InvalidTraitName {
                        reason: format!("Invalid character '{}' in trait/asset name", ch),
                    });
                }
            }
        }

        // Reconstruct the full identifier with normalized address
        let mut result = addr_upper;
        if let Some(contract) = contract_name {
            result.push('.');
            result.push_str(contract);
        }
        if let Some(trait_name) = trait_part {
            result.push_str("::");
            result.push_str(trait_name);
        }

        Ok(Self(result))
    }
    /// Get address as string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Get the base address part (before '.' if present).
    pub fn address(&self) -> &str {
        self.0.split('.').next().unwrap_or(&self.0)
    }

    /// Get contract name if present (part after '.' and before '::').
    pub fn contract_name(&self) -> Option<&str> {
        if let Some(after_dot) = self.0.split('.').nth(1) {
            // Handle case with trait: contract.name::trait
            if let Some(before_colon) = after_dot.split("::").next() {
                Some(before_colon)
            } else {
                Some(after_dot)
            }
        } else {
            None
        }
    }

    /// Get trait/asset name if present (part after '::').
    pub fn trait_name(&self) -> Option<&str> {
        self.0.split("::").nth(1)
    }

    /// Check if this is a simple address (no contract or trait parts).
    pub fn is_simple_address(&self) -> bool {
        !self.0.contains('.') && !self.0.contains("::")
    }

    /// Check if this is a contract identifier (has contract name).
    pub fn is_contract_identifier(&self) -> bool {
        self.0.contains('.')
    }

    /// Check if this is fully qualified (has trait/asset name).
    pub fn is_fully_qualified(&self) -> bool {
        self.0.contains("::")
    }

    /// Get network prefix.
    pub fn prefix(&self) -> &str {
        &self.address()[..2]
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

    #[error("Invalid contract name: {reason}")]
    InvalidContractName { reason: String },

    #[error("Invalid trait/asset name: {reason}")]
    InvalidTraitName { reason: String },
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

    // Contract identifier tests
    #[test]
    fn test_contract_identifier() {
        let addr = WalletAddress::new("SP1AY6K3PQV5MRT6R4S671NWW2FRVPKM0BR162CT6.leo-token");
        assert!(addr.is_ok(), "Valid contract identifier should parse");

        let addr = addr.unwrap();
        assert_eq!(addr.address(), "SP1AY6K3PQV5MRT6R4S671NWW2FRVPKM0BR162CT6");
        assert_eq!(addr.contract_name(), Some("leo-token"));
        assert_eq!(addr.trait_name(), None);
        assert!(!addr.is_simple_address());
        assert!(addr.is_contract_identifier());
        assert!(!addr.is_fully_qualified());
    }

    #[test]
    fn test_fully_qualified_contract() {
        let addr = WalletAddress::new("SP1AY6K3PQV5MRT6R4S671NWW2FRVPKM0BR162CT6.leo-token::LEO");
        assert!(addr.is_ok(), "Valid fully qualified contract should parse");

        let addr = addr.unwrap();
        assert_eq!(addr.address(), "SP1AY6K3PQV5MRT6R4S671NWW2FRVPKM0BR162CT6");
        assert_eq!(addr.contract_name(), Some("leo-token"));
        assert_eq!(addr.trait_name(), Some("LEO"));
        assert!(!addr.is_simple_address());
        assert!(addr.is_contract_identifier());
        assert!(addr.is_fully_qualified());
    }

    #[test]
    fn test_contract_with_underscore() {
        let addr = WalletAddress::new("SP1AY6K3PQV5MRT6R4S671NWW2FRVPKM0BR162CT6.test_token_v2");
        assert!(addr.is_ok());
        assert_eq!(addr.unwrap().contract_name(), Some("test_token_v2"));
    }

    #[test]
    fn test_invalid_contract_name_empty() {
        let addr = WalletAddress::new("SP1AY6K3PQV5MRT6R4S671NWW2FRVPKM0BR162CT6.");
        assert!(matches!(
            addr,
            Err(WalletAddressError::InvalidContractName { .. })
        ));
    }

    #[test]
    fn test_invalid_contract_name_special_char() {
        let addr = WalletAddress::new("SP1AY6K3PQV5MRT6R4S671NWW2FRVPKM0BR162CT6.leo@token");
        assert!(matches!(
            addr,
            Err(WalletAddressError::InvalidContractName { .. })
        ));
    }

    #[test]
    fn test_invalid_trait_name_empty() {
        let addr = WalletAddress::new("SP1AY6K3PQV5MRT6R4S671NWW2FRVPKM0BR162CT6.leo-token::");
        assert!(matches!(
            addr,
            Err(WalletAddressError::InvalidTraitName { .. })
        ));
    }

    #[test]
    fn test_simple_address_helpers() {
        let addr = WalletAddress::new("SPF0V8KWBS70F0WDKTMY65B3G591NN52PTHHN51D").unwrap();
        assert!(addr.is_simple_address());
        assert!(!addr.is_contract_identifier());
        assert!(!addr.is_fully_qualified());
        assert_eq!(addr.contract_name(), None);
        assert_eq!(addr.trait_name(), None);
    }
}
