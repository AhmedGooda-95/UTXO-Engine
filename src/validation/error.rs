use crate::primitives::outpoint::OutPoint;
use thiserror::Error;

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum ValidationError {
    #[error("Transaction has no inputs")]
    EmptyInputs,

    #[error("Transaction has no outputs")]
    EmptyOutputs,

    #[error("Output value must be greater than zero")]
    OutputZeroValue,

    #[error("Output values exceed maximum supply / overflow")]
    OutputValueOverflow,

    #[error("Double spend detected within transaction: input {0} used multiple times")]
    DuplicateInput(OutPoint),

    #[error("Double spend / missing UTXO: referenced output {0} is not in the UTXO set")]
    UtxoNotFound(OutPoint),

    #[error("Coinbase output {outpoint} is not mature (requires {required} confirmations, currently {current})")]
    CoinbaseNotMature {
        outpoint: OutPoint,
        required: u64,
        current: u64,
    },

    #[error("Insufficient funds: input sum ({input_total}) is less than output sum ({output_total})")]
    InsufficientFunds {
        input_total: u64,
        output_total: u64,
    },

    #[error("Public key hash mismatch on input {input_index}: public key does not match locking script")]
    PubKeyHashMismatch { input_index: usize },

    #[error("Cryptographic signature invalid on input {input_index} for output {outpoint}")]
    InvalidSignature {
        input_index: usize,
        outpoint: OutPoint,
    },

    #[error("Coinbase reward exceeded: allowed {allowed} sat, claimed {actual} sat")]
    CoinbaseRewardExceeded { allowed: u64, actual: u64 },
}
