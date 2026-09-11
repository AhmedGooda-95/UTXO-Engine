use ripemd::{Digest as RipemdDigest, Ripemd160};
use sha2::Sha256;

/// Computes a 20-byte RIPEMD-160 digest.
pub fn ripemd160(data: &[u8]) -> [u8; 20] {
    let mut hasher = Ripemd160::new();
    hasher.update(data);
    let result = hasher.finalize();
    result.into()
}

/// Computes Bitcoin's standard HASH160: RIPEMD-160(SHA-256(data)).
/// Used to derive public key hashes for addresses.
pub fn hash160(data: &[u8]) -> [u8; 20] {
    let mut sha = Sha256::new();
    sha.update(data);
    let sha_result = sha.finalize();

    let mut ripe = Ripemd160::new();
    ripe.update(&sha_result);
    let result = ripe.finalize();
    result.into()
}

/// Converts a 20-byte public key hash into a Base58Check address with a version byte.
/// Default version byte: 0x00 (Mainnet P2PKH standard starting with '1').
pub fn pubkey_hash_to_address(pkh: &[u8; 20], version: u8) -> String {
    let mut payload = Vec::with_capacity(21);
    payload.push(version);
    payload.extend_from_slice(pkh);
    bs58::encode(payload).with_check().into_string()
}

/// Converts an encoded Base58Check address back to its 20-byte public key hash.
pub fn address_to_pubkey_hash(address: &str) -> Result<([u8; 20], u8), String> {
    let decoded = bs58::decode(address)
        .with_check(None)
        .into_vec()
        .map_err(|e| format!("Invalid Base58Check address: {e}"))?;

    if decoded.len() != 21 {
        return Err(format!(
            "Invalid address payload length: expected 21, got {}",
            decoded.len()
        ));
    }

    let version = decoded[0];
    let mut pkh = [0u8; 20];
    pkh.copy_from_slice(&decoded[1..21]);
    Ok((pkh, version))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_address_roundtrip() {
        let pkh = [42u8; 20];
        let addr = pubkey_hash_to_address(&pkh, 0x00);
        assert!(addr.starts_with('1'));

        let (decoded_pkh, ver) = address_to_pubkey_hash(&addr).expect("Decoding failed");
        assert_eq!(decoded_pkh, pkh);
        assert_eq!(ver, 0x00);
    }
}
