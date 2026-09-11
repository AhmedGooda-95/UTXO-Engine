use crate::crypto::sha256::dsha256;
use crate::primitives::hash::Hash;
use crate::primitives::txin::TxIn;
use crate::primitives::txout::TxOut;

/// Computes the unique Transaction Identifier (TXID) by hashing the serialized
/// non-witness transaction structure with Double-SHA256.
pub fn compute_txid(
    version: u32,
    inputs: &[TxIn],
    outputs: &[TxOut],
    lock_time: u32,
) -> Hash {
    let mut bytes = Vec::new();

    // 1. Version (4 bytes, little-endian)
    bytes.extend_from_slice(&version.to_le_bytes());

    // 2. Input count (4 bytes, little-endian)
    bytes.extend_from_slice(&(inputs.len() as u32).to_le_bytes());

    // 3. Inputs (without signature / witness script)
    for input in inputs {
        bytes.extend_from_slice(input.previous_output.txid.as_bytes());
        bytes.extend_from_slice(&input.previous_output.vout.to_le_bytes());
        if input.is_coinbase() {
            bytes.extend_from_slice(&(input.signature.len() as u32).to_le_bytes());
            bytes.extend_from_slice(&input.signature);
        }
        bytes.extend_from_slice(&input.sequence.to_le_bytes());
    }

    // 4. Output count (4 bytes, little-endian)
    bytes.extend_from_slice(&(outputs.len() as u32).to_le_bytes());

    // 5. Outputs
    for output in outputs {
        bytes.extend_from_slice(&output.value.to_le_bytes());
        bytes.extend_from_slice(&output.pubkey_hash);
    }

    // 6. Lock time (4 bytes, little-endian)
    bytes.extend_from_slice(&lock_time.to_le_bytes());

    dsha256(&bytes)
}
