pub mod double_spend;
pub mod error;
pub mod fee;
pub mod input;
pub mod signature;

pub use double_spend::{check_inputs_unspent, check_no_duplicate_inputs};
pub use error::ValidationError;
pub use fee::calculate_and_verify_fee;
pub use input::{validate_coinbase_maturity, validate_structure, COINBASE_MATURITY};
pub use signature::validate_signatures;

use crate::transaction::tx::Transaction;
use crate::utxo::set::UtxoSet;

/// Runs all validation checks on a standard (non-coinbase) transaction:
/// 1. Structural validity (non-empty inputs/outputs, non-zero values).
/// 2. Internal double-spend prevention (unique inputs).
/// 3. External double-spend check (all inputs exist in UTXO set).
/// 4. Coinbase maturity checks on spent outputs.
/// 5. Value and fee verification (input_total >= output_total).
/// 6. Cryptographic signature and public key hash verification.
///
/// Returns the calculated transaction fee on success.
pub fn validate_transaction(
    tx: &Transaction,
    utxo_set: &UtxoSet,
    current_height: u64,
) -> Result<u64, ValidationError> {
    // 1. Structure
    validate_structure(tx)?;

    // 2. Duplicate inputs inside tx
    check_no_duplicate_inputs(tx)?;

    // 3. UTXO existence
    check_inputs_unspent(tx, utxo_set)?;

    // 4. Coinbase maturity
    validate_coinbase_maturity(tx, utxo_set, current_height)?;

    // 5. Fee and solvency check
    let fee = calculate_and_verify_fee(tx, utxo_set)?;

    // 6. Cryptographic signatures
    validate_signatures(tx, utxo_set)?;

    Ok(fee)
}
