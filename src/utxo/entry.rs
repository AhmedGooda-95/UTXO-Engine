use crate::primitives::txout::TxOut;
use serde::{Deserialize, Serialize};

/// Represents an unspent transaction output with metadata about its creation.
#[derive(Clone, Eq, PartialEq, Serialize, Deserialize, Debug)]
pub struct UtxoEntry {
    /// The actual transaction output (value and locking pubkey hash).
    pub output: TxOut,
    /// The block height at which this output was confirmed.
    pub block_height: u64,
    /// Flag indicating whether this output was created by a Coinbase transaction.
    pub is_coinbase: bool,
}

impl UtxoEntry {
    pub fn new(output: TxOut, block_height: u64, is_coinbase: bool) -> Self {
        Self {
            output,
            block_height,
            is_coinbase,
        }
    }
}
