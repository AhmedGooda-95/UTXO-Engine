use crate::primitives::outpoint::OutPoint;
use crate::transaction::tx::Transaction;
use crate::utxo::set::UtxoSet;
use crate::validation::error::ValidationError;
use std::collections::HashSet;

/// Verifies that a transaction does not attempt to spend the same OutPoint multiple times within itself.
pub fn check_no_duplicate_inputs(tx: &Transaction) -> Result<(), ValidationError> {
    if tx.is_coinbase() {
        return Ok(());
    }

    let mut seen: HashSet<OutPoint> = HashSet::with_capacity(tx.inputs.len());
    for input in &tx.inputs {
        if !seen.insert(input.previous_output) {
            return Err(ValidationError::DuplicateInput(input.previous_output));
        }
    }
    Ok(())
}

/// Verifies that all referenced inputs currently exist in the UTXO set (preventing double-spending of already spent coins).
pub fn check_inputs_unspent(tx: &Transaction, utxo_set: &UtxoSet) -> Result<(), ValidationError> {
    if tx.is_coinbase() {
        return Ok(());
    }

    for input in &tx.inputs {
        if !utxo_set.contains(&input.previous_output) {
            return Err(ValidationError::UtxoNotFound(input.previous_output));
        }
    }
    Ok(())
}
