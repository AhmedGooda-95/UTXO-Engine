use crate::transaction::tx::Transaction;
use crate::utxo::set::UtxoSet;
use crate::validation::error::ValidationError;

/// Calculates the transaction fee (sum of inputs - sum of outputs).
/// Coinbase transactions have a fee of 0.
pub fn calculate_and_verify_fee(
    tx: &Transaction,
    utxo_set: &UtxoSet,
) -> Result<u64, ValidationError> {
    if tx.is_coinbase() {
        return Ok(0);
    }

    let mut input_total: u64 = 0;
    for input in &tx.inputs {
        let entry = utxo_set
            .get(&input.previous_output)
            .ok_or(ValidationError::UtxoNotFound(input.previous_output))?;

        input_total = input_total
            .checked_add(entry.output.value)
            .ok_or(ValidationError::OutputValueOverflow)?;
    }

    let mut output_total: u64 = 0;
    for output in &tx.outputs {
        output_total = output_total
            .checked_add(output.value)
            .ok_or(ValidationError::OutputValueOverflow)?;
    }

    if input_total < output_total {
        return Err(ValidationError::InsufficientFunds {
            input_total,
            output_total,
        });
    }

    let fee = input_total - output_total;
    Ok(fee)
}
