use crate::primitives::outpoint::OutPoint;
use crate::utxo::entry::UtxoEntry;
use serde::{Deserialize, Serialize};

/// Represents the state modifications made by applying transactions.
/// This acts as an Undo Log / Differential that allows completely atomic
/// rollbacks (e.g. during chain reorganizations).
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct UtxoDiff {
    /// OutPoints and their original entries that were consumed/removed.
    pub spent_utxos: Vec<(OutPoint, UtxoEntry)>,
    /// OutPoints that were added as new unspent outputs.
    pub created_outpoints: Vec<OutPoint>,
}

impl UtxoDiff {
    pub fn new() -> Self {
        Self::default()
    }

    /// Merges another differential into this one (useful for whole blocks).
    pub fn merge(&mut self, mut other: UtxoDiff) {
        self.spent_utxos.append(&mut other.spent_utxos);
        self.created_outpoints.append(&mut other.created_outpoints);
    }
}
