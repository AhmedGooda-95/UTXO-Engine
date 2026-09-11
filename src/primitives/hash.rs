use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;

/// A 32-byte cryptographic hash representation.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Default)]
pub struct Hash(pub [u8; 32]);

impl Hash {
    pub const ZERO: Hash = Hash([0u8; 32]);

    pub fn new(bytes: [u8; 32]) -> Self {
        Hash(bytes)
    }

    pub fn from_slice(slice: &[u8]) -> Result<Self, String> {
        if slice.len() != 32 {
            return Err(format!("Invalid hash length: expected 32, got {}", slice.len()));
        }
        let mut arr = [0u8; 32];
        arr.copy_from_slice(slice);
        Ok(Hash(arr))
    }

    pub fn from_hex(hex_str: &str) -> Result<Self, String> {
        let clean = hex_str.trim().strip_prefix("0x").unwrap_or(hex_str);
        let bytes = hex::decode(clean).map_err(|e| format!("Hex decoding error: {e}"))?;
        Self::from_slice(&bytes)
    }

    pub fn to_hex(&self) -> String {
        hex::encode(self.0)
    }

    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    pub fn is_zero(&self) -> bool {
        self.0 == [0u8; 32]
    }
}

impl AsRef<[u8]> for Hash {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl From<[u8; 32]> for Hash {
    fn from(arr: [u8; 32]) -> Self {
        Hash(arr)
    }
}

impl fmt::Display for Hash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_hex())
    }
}

impl fmt::Debug for Hash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Hash({})", self.to_hex())
    }
}

impl Serialize for Hash {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if serializer.is_human_readable() {
            serializer.serialize_str(&self.to_hex())
        } else {
            serializer.serialize_bytes(&self.0)
        }
    }
}

impl<'de> Deserialize<'de> for Hash {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        if deserializer.is_human_readable() {
            let s = String::deserialize(deserializer)?;
            Hash::from_hex(&s).map_err(serde::de::Error::custom)
        } else {
            let bytes = <Vec<u8>>::deserialize(deserializer)?;
            Hash::from_slice(&bytes).map_err(serde::de::Error::custom)
        }
    }
}
