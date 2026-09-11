use serde::{Deserialize, Serialize};
use std::fmt;

/// A Transaction Output (TxOut) defines an amount of cryptocurrency and the recipient.
#[derive(Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct TxOut {
    /// The value transferred in satoshis (1 BTC = 100,000,000 satoshis).
    pub value: u64,
    /// The human-readable recipient address (e.g. Base58Check encoded).
    pub recipient_address: String,
    /// The 20-byte public key hash (RIPEMD-160 of SHA-256(pubkey)) locking this output.
    pub pubkey_hash: [u8; 20],
}

impl TxOut {
    pub fn new(value: u64, recipient_address: String, pubkey_hash: [u8; 20]) -> Self {
        Self {
            value,
            recipient_address,
            pubkey_hash,
        }
    }
}

impl fmt::Display for TxOut {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "TxOut(value: {} sat, recipient: {})",
            self.value, self.recipient_address
        )
    }
}

impl fmt::Debug for TxOut {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "TxOut {{ value: {}, recipient: {}, pkh: {} }}",
            self.value,
            self.recipient_address,
            hex::encode(self.pubkey_hash)
        )
    }
}
