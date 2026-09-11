use super::hash::Hash;
use serde::{Deserialize, Serialize};
use std::fmt;

/// An OutPoint identifies a specific transaction output by its TxId and output index (`vout`).
#[derive(Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct OutPoint {
    /// The transaction hash containing the output.
    pub txid: Hash,
    /// The zero-based index of the output in the transaction.
    pub vout: u32,
}

impl OutPoint {
    pub fn new(txid: Hash, vout: u32) -> Self {
        Self { txid, vout }
    }

    /// Creates a null OutPoint, traditionally used in Coinbase transactions.
    pub fn null() -> Self {
        Self {
            txid: Hash::ZERO,
            vout: u32::MAX,
        }
    }

    pub fn is_null(&self) -> bool {
        self.txid.is_zero() && self.vout == u32::MAX
    }
}

impl fmt::Display for OutPoint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.txid, self.vout)
    }
}

impl fmt::Debug for OutPoint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "OutPoint({}:{})", self.txid, self.vout)
    }
}
