use crate::primitives::hash::Hash;
use sha2::{Digest, Sha256};

/// Computes a single SHA-256 hash.
pub fn sha256(data: &[u8]) -> Hash {
    let mut hasher = Sha256::new();
    hasher.update(data);
    let result = hasher.finalize();
    Hash(result.into())
}

/// Computes Double-SHA256: SHA-256(SHA-256(data)).
/// This is the standard cryptographic hash used in Bitcoin for TXIDs and Block Hashes
/// to protect against length extension attacks.
pub fn dsha256(data: &[u8]) -> Hash {
    let first = sha256(data);
    sha256(first.as_bytes())
}

/// Computes the Double-SHA256 of two concatenated 32-byte hashes (used in Merkle trees).
pub fn hash256_combine(left: &Hash, right: &Hash) -> Hash {
    let mut combined = [0u8; 64];
    combined[..32].copy_from_slice(left.as_bytes());
    combined[32..].copy_from_slice(right.as_bytes());
    dsha256(&combined)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sha256_empty() {
        let h = sha256(b"");
        assert_eq!(
            h.to_hex(),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }

    #[test]
    fn test_dsha256_hello() {
        let h = dsha256(b"hello world");
        assert!(!h.is_zero());
    }
}
