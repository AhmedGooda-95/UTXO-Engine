use crate::crypto::ripemd160::hash160;
use crate::crypto::secp256k1::verify_signature;
use crate::transaction::tx::Transaction;
use crate::utxo::set::UtxoSet;
use crate::validation::error::ValidationError;

/// Verifies cryptographic ECDSA signatures for all inputs in a transaction.
pub fn validate_signatures(tx: &Transaction, utxo_set: &UtxoSet) -> Result<(), ValidationError> {
    if tx.is_coinbase() {
        return Ok(());
    }

    for (index, input) in tx.inputs.iter().enumerate() {
        let utxo = utxo_set
            .get(&input.previous_output)
            .ok_or(ValidationError::UtxoNotFound(input.previous_output))?;

        // 1. Check that the provided public key hashes to the locking script's pubkey_hash
        let derived_pkh = hash160(&input.public_key);
        if derived_pkh != utxo.output.pubkey_hash {
            return Err(ValidationError::PubKeyHashMismatch { input_index: index });
        }

        // 2. Compute the exact sighash digest for this input
        let sighash = tx
            .signature_hash(index, &utxo.output.pubkey_hash)
            .map_err(|_| ValidationError::InvalidSignature {
                input_index: index,
                outpoint: input.previous_output,
            })?;

        // 3. Verify the ECDSA Secp256k1 signature
        let is_valid = verify_signature(&input.public_key, &sighash, &input.signature)
            .map_err(|_| ValidationError::InvalidSignature {
                input_index: index,
                outpoint: input.previous_output,
            })?;

        if !is_valid {
            return Err(ValidationError::InvalidSignature {
                input_index: index,
                outpoint: input.previous_output,
            });
        }
    }

    Ok(())
}
