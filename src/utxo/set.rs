use crate::primitives::outpoint::OutPoint;
use crate::transaction::tx::Transaction;
use crate::utxo::apply::apply_transaction;
use crate::utxo::entry::UtxoEntry;
use crate::utxo::error::UtxoError;
use crate::utxo::revert::UtxoDiff;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// The complete state of unspent transaction outputs in the blockchain.
#[derive(Clone, Default, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct UtxoSet {
    entries: HashMap<OutPoint, UtxoEntry>,
}

impl UtxoSet {
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn contains(&self, outpoint: &OutPoint) -> bool {
        self.entries.contains_key(outpoint)
    }

    pub fn get(&self, outpoint: &OutPoint) -> Option<&UtxoEntry> {
        self.entries.get(outpoint)
    }

    pub fn insert(&mut self, outpoint: OutPoint, entry: UtxoEntry) -> Option<UtxoEntry> {
        self.entries.insert(outpoint, entry)
    }

    pub fn remove(&mut self, outpoint: &OutPoint) -> Option<UtxoEntry> {
        self.entries.remove(outpoint)
    }

    /// Applies a transaction directly, updating state and returning the undo diff.
    pub fn apply_transaction(
        &mut self,
        tx: &Transaction,
        block_height: u64,
    ) -> Result<UtxoDiff, UtxoError> {
        apply_transaction(self, tx, block_height)
    }

    /// Reverts a UtxoDiff atomically to undo transactions (for chain reorgs).
    pub fn revert_diff(&mut self, diff: &UtxoDiff) -> Result<(), UtxoError> {
        // 1. Remove all outputs created by the reverted transactions
        for outpoint in &diff.created_outpoints {
            if self.entries.remove(outpoint).is_none() {
                return Err(UtxoError::RevertMissingCreated(*outpoint));
            }
        }

        // 2. Restore all previously spent inputs back into unspent state
        for (outpoint, entry) in &diff.spent_utxos {
            self.entries.insert(*outpoint, entry.clone());
        }

        Ok(())
    }

    /// Calculates the spendable balance for a given address.
    pub fn get_balance(&self, address: &str) -> u64 {
        self.entries
            .values()
            .filter(|e| e.output.recipient_address == address)
            .map(|e| e.output.value)
            .sum()
    }

    /// Retrieves all UTXOs belonging to a given address.
    pub fn get_utxos_for_address(&self, address: &str) -> Vec<(OutPoint, UtxoEntry)> {
        self.entries
            .iter()
            .filter(|(_, e)| e.output.recipient_address == address)
            .map(|(op, e)| (*op, e.clone()))
            .collect()
    }

    /// Returns the total circulating coins currently in existence across all UTXOs.
    pub fn total_supply(&self) -> u64 {
        self.entries.values().map(|e| e.output.value).sum()
    }

    pub fn iter(&self) -> impl Iterator<Item = (&OutPoint, &UtxoEntry)> {
        self.entries.iter()
    }
}
