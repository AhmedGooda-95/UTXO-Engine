use crate::primitives::outpoint::OutPoint;
use thiserror::Error;

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum UtxoError {
    #[error("UTXO not found: {0}")]
    UtxoNotFound(OutPoint),

    #[error("Output index {0} already exists in UTXO set")]
    OutputAlreadyExists(OutPoint),

    #[error("Cannot spend coinbase output before maturity (requires {required} confirmations, currently {current})")]
    CoinbaseNotMature { required: u64, current: u64 },

    #[error("Reversion error: expected created output {0} was not in UTXO set")]
    RevertMissingCreated(OutPoint),
}
