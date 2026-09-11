use crate::primitives::outpoint::OutPoint;
use crate::transaction::tx::Transaction;
use crate::utxo::entry::UtxoEntry;
use crate::utxo::error::UtxoError;
use crate::utxo::revert::UtxoDiff;
use crate::utxo::set::UtxoSet;

/// Applies a validated transaction to the UTXO set:
/// 1. Consumes (removes) inputs from the UTXO set (unless coinbase).
/// 2. Creates and stores new UTXO entries for every output.
/// 3. Records state alterations in `UtxoDiff` to enable atomic rollback.
pub fn apply_transaction(
    utxo_set: &mut UtxoSet,
    tx: &Transaction,
    block_height: u64,
) -> Result<UtxoDiff, UtxoError> {
    let txid = tx.txid();
    let is_coinbase = tx.is_coinbase();
    let mut diff = UtxoDiff::new();

    // 1. Consume inputs (Coinbase transactions do not consume existing UTXOs)
    if !is_coinbase {
        for input in &tx.inputs {
            let spent_entry = utxo_set
                .remove(&input.previous_output)
                .ok_or(UtxoError::UtxoNotFound(input.previous_output))?;

            diff.spent_utxos.push((input.previous_output, spent_entry));
        }
    }

    // 2. Create outputs
    for (vout, output) in tx.outputs.iter().enumerate() {
        let outpoint = OutPoint::new(txid, vout as u32);
        let entry = UtxoEntry::new(output.clone(), block_height, is_coinbase);

        utxo_set.insert(outpoint, entry);
        diff.created_outpoints.push(outpoint);
    }

    Ok(diff)
}
