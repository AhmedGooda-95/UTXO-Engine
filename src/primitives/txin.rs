use super::outpoint::OutPoint;
use serde::{Deserialize, Serialize};
use std::fmt;

/// A Transaction Input (TxIn) references an unspent transaction output and provides proof of authorization.
#[derive(Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct TxIn {
    /// Reference to the specific output being spent.
    pub previous_output: OutPoint,
    /// Cryptographic signature unlocking the previous output.
    pub signature: Vec<u8>,
    /// Public key corresponding to the private key used to generate the signature.
    pub public_key: Vec<u8>,
    /// Sequence number (used for locktime opt-ins / RBF). Default is 0xFFFFFFFF.
    pub sequence: u32,
}

impl TxIn {
    pub const FINAL_SEQUENCE: u32 = 0xFFFF_FFFF;

    pub fn new(previous_output: OutPoint, sequence: u32) -> Self {
        Self {
            previous_output,
            signature: Vec::new(),
            public_key: Vec::new(),
            sequence,
        }
    }

    pub fn with_witness(
        previous_output: OutPoint,
        signature: Vec<u8>,
        public_key: Vec<u8>,
        sequence: u32,
    ) -> Self {
        Self {
            previous_output,
            signature,
            public_key,
            sequence,
        }
    }

    /// Creates a dummy input for a Coinbase transaction.
    pub fn coinbase(extra_data: &[u8]) -> Self {
        Self {
            previous_output: OutPoint::null(),
            signature: extra_data.to_vec(),
            public_key: Vec::new(),
            sequence: Self::FINAL_SEQUENCE,
        }
    }

    pub fn is_coinbase(&self) -> bool {
        self.previous_output.is_null()
    }
}

impl fmt::Display for TxIn {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_coinbase() {
            write!(f, "TxIn(Coinbase, data: {} bytes)", self.signature.len())
        } else {
            write!(f, "TxIn(spending: {})", self.previous_output)
        }
    }
}

impl fmt::Debug for TxIn {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "TxIn {{ prev: {:?}, sig_len: {}, pubkey_len: {}, seq: {:08x} }}",
            self.previous_output,
            self.signature.len(),
            self.public_key.len(),
            self.sequence
        )
    }
}
