use crate::transaction::tx::Transaction;
use crate::utxo::set::UtxoSet;
use crate::validation::error::ValidationError;

/// Number of confirmations required before a coinbase output can be spent.
pub const COINBASE_MATURITY: u64 = 10;

/// Validates structural rules of the transaction: non-empty lists and positive output values.
pub fn validate_structure(tx: &Transaction) -> Result<(), ValidationError> {
    if tx.inputs.is_empty() {
        return Err(ValidationError::EmptyInputs);
    }

    if tx.outputs.is_empty() {
        return Err(ValidationError::EmptyOutputs);
    }

    for output in &tx.outputs {
        if output.value == 0 {
            return Err(ValidationError::OutputZeroValue);
        }
    }

    Ok(())
}

/// Verifies that any spent Coinbase outputs have achieved the required maturity.
pub fn validate_coinbase_maturity(
    tx: &Transaction,
    utxo_set: &UtxoSet,
    current_height: u64,
) -> Result<(), ValidationError> {
    if tx.is_coinbase() {
        return Ok(());
    }

    for input in &tx.inputs {
        if let Some(entry) = utxo_set.get(&input.previous_output) {
            if entry.is_coinbase {
                let confirmations = current_height.saturating_sub(entry.block_height);
                if confirmations < COINBASE_MATURITY {
                    return Err(ValidationError::CoinbaseNotMature {
                        outpoint: input.previous_output,
                        required: COINBASE_MATURITY,
                        current: confirmations,
                    });
                }
            }
        }
    }

    Ok(())
}
