use crate::crypto::ripemd160::{hash160, pubkey_hash_to_address};
use crate::primitives::hash::Hash;
use k256::ecdsa::signature::hazmat::{PrehashSigner, PrehashVerifier};
use k256::ecdsa::{Signature, SigningKey, VerifyingKey};
use rand::rngs::OsRng;

/// Represents a Secp256k1 key pair for creating transactions.
#[derive(Clone)]
pub struct KeyPair {
    signing_key: SigningKey,
    verifying_key: VerifyingKey,
}

impl KeyPair {
    /// Generates a new cryptographically secure random Secp256k1 key pair.
    pub fn generate() -> Self {
        let signing_key = SigningKey::random(&mut OsRng);
        let verifying_key = VerifyingKey::from(&signing_key);
        Self {
            signing_key,
            verifying_key,
        }
    }

    /// Creates a key pair from 32-byte secret key bytes.
    pub fn from_secret_bytes(bytes: &[u8]) -> Result<Self, String> {
        let signing_key = SigningKey::from_slice(bytes)
            .map_err(|e| format!("Invalid private key: {e}"))?;
        let verifying_key = VerifyingKey::from(&signing_key);
        Ok(Self {
            signing_key,
            verifying_key,
        })
    }

    /// Returns the 32-byte private key as raw bytes.
    pub fn private_key_bytes(&self) -> [u8; 32] {
        let mut bytes = [0u8; 32];
        bytes.copy_from_slice(&self.signing_key.to_bytes());
        bytes
    }

    /// Returns the 33-byte SEC1 compressed public key.
    pub fn public_key_bytes(&self) -> Vec<u8> {
        self.verifying_key.to_encoded_point(true).as_bytes().to_vec()
    }

    /// Derives the 20-byte HASH160 (RIPEMD-160 of SHA-256) of the compressed public key.
    pub fn pubkey_hash(&self) -> [u8; 20] {
        hash160(&self.public_key_bytes())
    }

    /// Derives the standard Base58Check address (version 0x00).
    pub fn address(&self) -> String {
        pubkey_hash_to_address(&self.pubkey_hash(), 0x00)
    }

    /// Signs a 32-byte message digest (e.g. sighash) using ECDSA.
    /// Returns DER-encoded signature bytes.
    pub fn sign_digest(&self, digest: &Hash) -> Result<Vec<u8>, String> {
        let signature: Signature = self
            .signing_key
            .sign_prehash(digest.as_bytes())
            .map_err(|e| format!("Signing failed: {e}"))?;
        Ok(signature.to_der().as_bytes().to_vec())
    }
}

/// Verifies an ECDSA signature against a 32-byte digest and a 33-byte compressed public key.
pub fn verify_signature(
    public_key_bytes: &[u8],
    digest: &Hash,
    der_signature_bytes: &[u8],
) -> Result<bool, String> {
    let verifying_key = VerifyingKey::from_sec1_bytes(public_key_bytes)
        .map_err(|e| format!("Invalid public key encoding: {e}"))?;

    let signature = Signature::from_der(der_signature_bytes)
        .map_err(|e| format!("Invalid DER signature: {e}"))?;

    match verifying_key.verify_prehash(digest.as_bytes(), &signature) {
        Ok(()) => Ok(true),
        Err(_) => Ok(false),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sign_and_verify() {
        let kp = KeyPair::generate();
        let digest = Hash([0xAB; 32]);
        let sig = kp.sign_digest(&digest).expect("Failed to sign");
        let pk = kp.public_key_bytes();

        let valid = verify_signature(&pk, &digest, &sig).expect("Verification failed");
        assert!(valid, "Signature must be valid");

        // Test with corrupted digest
        let corrupted_digest = Hash([0xCD; 32]);
        let invalid = verify_signature(&pk, &corrupted_digest, &sig).expect("Verification error");
        assert!(!invalid, "Signature must fail with different digest");
    }
}
