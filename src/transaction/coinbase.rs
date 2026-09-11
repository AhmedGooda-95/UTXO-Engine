use crate::primitives::txin::TxIn;
use crate::primitives::txout::TxOut;
use crate::transaction::tx::Transaction;

/// Default block subsidy in satoshis (e.g. 50 BTC = 5,000,000,000 satoshis).
pub const INITIAL_SUBSIDY: u64 = 5_000_000_000;

/// Halving interval in number of blocks (e.g. 210,000 blocks).
pub const HALVING_INTERVAL: u64 = 210_000;

/// Calculates the block subsidy at a given block height.
pub fn calculate_block_subsidy(height: u64) -> u64 {
    let halvings = height / HALVING_INTERVAL;
    if halvings >= 64 {
        0
    } else {
        INITIAL_SUBSIDY >> halvings
    }
}

/// Creates a new Coinbase transaction rewarding the miner with block subsidy + accumulated fees.
pub fn create_coinbase_tx(
    miner_address: &str,
    miner_pkh: [u8; 20],
    block_height: u64,
    total_fees: u64,
    extra_nonce: u64,
) -> Transaction {
    let subsidy = calculate_block_subsidy(block_height);
    let total_reward = subsidy.saturating_add(total_fees);

    // Encode block height & extra nonce into coinbase arbitrary data
    let mut coinbase_data = Vec::new();
    coinbase_data.extend_from_slice(&block_height.to_le_bytes());
    coinbase_data.extend_from_slice(&extra_nonce.to_le_bytes());
    coinbase_data.extend_from_slice(b"/Rust-UTXO-Engine/");

    let input = TxIn::coinbase(&coinbase_data);
    let output = TxOut::new(total_reward, miner_address.to_string(), miner_pkh);

    Transaction::new(vec![input], vec![output], 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_subsidy_halving() {
        assert_eq!(calculate_block_subsidy(0), INITIAL_SUBSIDY);
        assert_eq!(calculate_block_subsidy(209_999), INITIAL_SUBSIDY);
        assert_eq!(calculate_block_subsidy(210_000), INITIAL_SUBSIDY / 2);
        assert_eq!(calculate_block_subsidy(420_000), INITIAL_SUBSIDY / 4);
    }

    #[test]
    fn test_coinbase_structure() {
        let pkh = [0x55; 20];
        let cb = create_coinbase_tx("1MinerAddr", pkh, 100, 50_000, 12345);
        assert!(cb.is_coinbase());
        assert_eq!(cb.outputs.len(), 1);
        assert_eq!(cb.outputs[0].value, INITIAL_SUBSIDY + 50_000);
    }
}
