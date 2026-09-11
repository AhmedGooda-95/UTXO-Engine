use crate::crypto::sha256::dsha256;
use crate::primitives::hash::Hash;
use crate::primitives::txin::TxIn;
use crate::primitives::txout::TxOut;
use crate::transaction::txid::compute_txid;
use serde::{Deserialize, Serialize};
use std::fmt;

/// A complete cryptocurrency Transaction.
#[derive(Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Transaction {
    /// Transaction version number (e.g. 1 or 2).
    pub version: u32,
    /// List of transaction inputs.
    pub inputs: Vec<TxIn>,
    /// List of transaction outputs.
    pub outputs: Vec<TxOut>,
    /// Block height or timestamp before which this transaction cannot be included.
    pub lock_time: u32,
}

impl Transaction {
    pub const DEFAULT_VERSION: u32 = 1;

    pub fn new(inputs: Vec<TxIn>, outputs: Vec<TxOut>, lock_time: u32) -> Self {
        Self {
            version: Self::DEFAULT_VERSION,
            inputs,
            outputs,
            lock_time,
        }
    }

    /// Computes the unique TXID of this transaction.
    pub fn txid(&self) -> Hash {
        compute_txid(self.version, &self.inputs, &self.outputs, self.lock_time)
    }

    /// Returns true if this is a Coinbase transaction.
    pub fn is_coinbase(&self) -> bool {
        self.inputs.len() == 1 && self.inputs[0].is_coinbase()
    }

    /// Returns the sum of all output values in satoshis.
    pub fn total_output_value(&self) -> u64 {
        self.outputs.iter().map(|o| o.value).sum()
    }

    /// Computes the Bitcoin-style SIGHASH digest for a given input index.
    /// This binds the signature to all inputs and outputs of the transaction,
    /// committing to the specific output being spent (prev_pubkey_hash).
    pub fn signature_hash(
        &self,
        input_index: usize,
        prev_pubkey_hash: &[u8; 20],
    ) -> Result<Hash, String> {
        if input_index >= self.inputs.len() {
            return Err(format!(
                "Input index {} out of bounds ({})",
                input_index,
                self.inputs.len()
            ));
        }

        let mut bytes = Vec::new();

        // 1. Version
        bytes.extend_from_slice(&self.version.to_le_bytes());

        // 2. Input count
        bytes.extend_from_slice(&(self.inputs.len() as u32).to_le_bytes());

        // 3. Inputs: the input being signed embeds the previous output's pubkey_hash,
        // while all other inputs are serialized without any signature data.
        for (i, input) in self.inputs.iter().enumerate() {
            bytes.extend_from_slice(input.previous_output.txid.as_bytes());
            bytes.extend_from_slice(&input.previous_output.vout.to_le_bytes());

            if i == input_index {
                // Insert the locking condition of the coin being spent
                bytes.extend_from_slice(&(prev_pubkey_hash.len() as u32).to_le_bytes());
                bytes.extend_from_slice(prev_pubkey_hash);
            } else {
                bytes.extend_from_slice(&0u32.to_le_bytes()); // empty
            }

            bytes.extend_from_slice(&input.sequence.to_le_bytes());
        }

        // 4. Output count
        bytes.extend_from_slice(&(self.outputs.len() as u32).to_le_bytes());

        // 5. Outputs
        for output in &self.outputs {
            bytes.extend_from_slice(&output.value.to_le_bytes());
            bytes.extend_from_slice(&output.pubkey_hash);
        }

        // 6. Lock time
        bytes.extend_from_slice(&self.lock_time.to_le_bytes());

        // SIGHASH_ALL type flag (0x01000000)
        bytes.extend_from_slice(&1u32.to_le_bytes());

        Ok(dsha256(&bytes))
    }

    /// Approximate size in bytes for fee rate calculation (satoshis/byte).
    pub fn size_in_bytes(&self) -> usize {
        let mut size = 4 + 4 + 4 + 4; // version, input count, output count, locktime
        for input in &self.inputs {
            size += 32 + 4 + 4 + input.signature.len() + input.public_key.len();
        }
        for output in &self.outputs {
            size += 8 + 20 + output.recipient_address.len();
        }
        size
    }
}

impl fmt::Display for Transaction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Tx(txid: {}, inputs: {}, outputs: {}, total: {} sat)",
            self.txid(),
            self.inputs.len(),
            self.outputs.len(),
            self.total_output_value()
        )
    }
}

impl fmt::Debug for Transaction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Transaction {{ txid: {}, is_coinbase: {}, inputs: {:?}, outputs: {:?} }}",
            self.txid(),
            self.is_coinbase(),
            self.inputs,
            self.outputs
        )
    }
}
