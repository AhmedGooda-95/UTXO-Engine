use crate::primitives::hash::Hash;
use crate::primitives::outpoint::OutPoint;
use crate::transaction::tx::Transaction;
use crate::utxo::set::UtxoSet;
use crate::validation::error::ValidationError;
use crate::validation::validate_transaction;
use std::collections::HashMap;

/// An entry in the memory pool.
#[derive(Clone, Debug)]
pub struct MempoolEntry {
    pub transaction: Transaction,
    pub fee: u64,
    pub size: usize,
    pub fee_rate: f64, // satoshis per byte
}

/// The Mempool manages unconfirmed transactions awaiting inclusion in a block.
#[derive(Clone, Debug, Default)]
pub struct Mempool {
    entries: HashMap<Hash, MempoolEntry>,
    spent_outpoints: HashMap<OutPoint, Hash>, // OutPoint -> TxID of spender
}

impl Mempool {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn contains(&self, txid: &Hash) -> bool {
        self.entries.contains_key(txid)
    }

    /// Adds a transaction to the mempool if it is valid and does not double-spend
    /// any output already spent by another transaction in the mempool or the UTXO set.
    pub fn add_transaction(
        &mut self,
        tx: Transaction,
        utxo_set: &UtxoSet,
        current_height: u64,
    ) -> Result<u64, ValidationError> {
        let txid = tx.txid();

        if self.entries.contains_key(&txid) {
            return Ok(self.entries[&txid].fee);
        }

        // 1. Check for conflicts with other unconfirmed transactions in the mempool
        for input in &tx.inputs {
            if let Some(existing_txid) = self.spent_outpoints.get(&input.previous_output) {
                if *existing_txid != txid {
                    return Err(ValidationError::DuplicateInput(input.previous_output));
                }
            }
        }

        // 2. Validate against current UTXO set
        let fee = validate_transaction(&tx, utxo_set, current_height)?;
        let size = tx.size_in_bytes().max(1);
        let fee_rate = (fee as f64) / (size as f64);

        // 3. Register spent outpoints in mempool
        for input in &tx.inputs {
            self.spent_outpoints.insert(input.previous_output, txid);
        }

        // 4. Store entry
        self.entries.insert(
            txid,
            MempoolEntry {
                transaction: tx,
                fee,
                size,
                fee_rate,
            },
        );

        Ok(fee)
    }

    /// Returns transactions ordered by highest fee rate for inclusion in a new block.
    pub fn get_block_template(&self, max_bytes: usize) -> (Vec<Transaction>, u64) {
        let mut sorted: Vec<&MempoolEntry> = self.entries.values().collect();
        // Sort descending by fee_rate, then descending by absolute fee
        sorted.sort_by(|a, b| {
            b.fee_rate
                .partial_cmp(&a.fee_rate)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| b.fee.cmp(&a.fee))
        });

        let mut selected = Vec::new();
        let mut current_bytes = 0;
        let mut total_fees = 0;

        for entry in sorted {
            if current_bytes + entry.size <= max_bytes {
                selected.push(entry.transaction.clone());
                current_bytes += entry.size;
                total_fees += entry.fee;
            }
        }

        (selected, total_fees)
    }

    /// Removes confirmed transactions from the mempool after a block is mined.
    pub fn remove_confirmed(&mut self, txs: &[Transaction]) {
        for tx in txs {
            let txid = tx.txid();
            if self.entries.remove(&txid).is_some() {
                for input in &tx.inputs {
                    self.spent_outpoints.remove(&input.previous_output);
                }
            }
        }
    }

    /// Clears the entire mempool.
    pub fn clear(&mut self) {
        self.entries.clear();
        self.spent_outpoints.clear();
    }
}
