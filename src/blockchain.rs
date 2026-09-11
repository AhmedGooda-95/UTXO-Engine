use crate::block::Block;
use crate::crypto::secp256k1::KeyPair;
use crate::mempool::Mempool;
use crate::primitives::hash::Hash;
use crate::primitives::txin::TxIn;
use crate::primitives::txout::TxOut;
use crate::transaction::coinbase::{calculate_block_subsidy, create_coinbase_tx};
use crate::transaction::tx::Transaction;
use crate::utxo::revert::UtxoDiff;
use crate::utxo::set::UtxoSet;
use crate::validation::validate_transaction;

/// The complete Blockchain UTXO Engine orchestrating state, consensus, and transactions.
#[derive(Clone, Debug)]
pub struct Blockchain {
    pub blocks: Vec<Block>,
    pub block_diffs: Vec<UtxoDiff>,
    pub utxo_set: UtxoSet,
    pub mempool: Mempool,
    pub mining_difficulty_zeros: usize,
}

impl Blockchain {
    /// Initializes the blockchain with a Genesis Block.
    pub fn new(genesis_miner_address: &str, genesis_miner_pkh: [u8; 20]) -> Self {
        let genesis_block = Block::genesis(genesis_miner_address, genesis_miner_pkh);
        let mut utxo_set = UtxoSet::new();

        // Apply genesis transactions to initial UTXO set
        let mut genesis_diff = UtxoDiff::new();
        for tx in &genesis_block.transactions {
            let diff = utxo_set
                .apply_transaction(tx, 0)
                .expect("Genesis block application failed");
            genesis_diff.merge(diff);
        }

        Self {
            blocks: vec![genesis_block],
            block_diffs: vec![genesis_diff],
            utxo_set,
            mempool: Mempool::new(),
            mining_difficulty_zeros: 1, // 1 leading zero byte for fast testing/demo
        }
    }

    pub fn height(&self) -> u64 {
        (self.blocks.len() as u64).saturating_sub(1)
    }

    pub fn tip_hash(&self) -> Hash {
        self.blocks.last().map(|b| b.hash()).unwrap_or(Hash::ZERO)
    }

    /// Adds and validates a newly mined block to the blockchain.
    pub fn add_block(&mut self, block: Block) -> Result<(), String> {
        let current_height = self.height() + 1;

        // 1. Validate previous block hash linkage
        if block.header.prev_block_hash != self.tip_hash() {
            return Err(format!(
                "Invalid previous block hash: expected {}, got {}",
                self.tip_hash(),
                block.header.prev_block_hash
            ));
        }

        // 2. Validate Proof of Work
        if !block.verify_pow(self.mining_difficulty_zeros) {
            return Err("Block does not meet Proof-of-Work difficulty target".to_string());
        }

        // 3. Validate Merkle Root
        if !block.verify_merkle_root() {
            return Err("Block transactions do not match header Merkle Root".to_string());
        }

        // 4. Must have at least 1 transaction (the Coinbase)
        if block.transactions.is_empty() {
            return Err("Block cannot be empty (requires coinbase)".to_string());
        }

        let coinbase_tx = &block.transactions[0];
        if !coinbase_tx.is_coinbase() {
            return Err("First transaction in block must be Coinbase".to_string());
        }

        // 5. Validate non-coinbase transactions and accumulate fees
        let mut total_fees = 0u64;
        let mut block_diff = UtxoDiff::new();

        // Temporary copy of UTXO set to execute transactions atomically
        let mut temp_utxo_set = self.utxo_set.clone();

        for (idx, tx) in block.transactions.iter().enumerate().skip(1) {
            if tx.is_coinbase() {
                return Err(format!("Multiple coinbase transactions at index {idx}"));
            }

            let fee = validate_transaction(tx, &temp_utxo_set, current_height)
                .map_err(|e| format!("Transaction {idx} ({}) invalid: {e}", tx.txid()))?;

            total_fees = total_fees
                .checked_add(fee)
                .ok_or("Block fee accumulation overflow")?;

            let diff = temp_utxo_set
                .apply_transaction(tx, current_height)
                .map_err(|e| format!("Failed to apply tx {idx}: {e}"))?;

            block_diff.merge(diff);
        }

        // 6. Validate Coinbase reward (Subsidy + Fees)
        let allowed_subsidy = calculate_block_subsidy(current_height);
        let max_reward = allowed_subsidy
            .checked_add(total_fees)
            .ok_or("Max reward overflow")?;
        let actual_reward = coinbase_tx.total_output_value();

        if actual_reward > max_reward {
            return Err(format!(
                "Coinbase reward exceeded: allowed {max_reward} sat, miner took {actual_reward} sat"
            ));
        }

        // 7. Apply Coinbase to state
        let cb_diff = temp_utxo_set
            .apply_transaction(coinbase_tx, current_height)
            .map_err(|e| format!("Failed to apply coinbase tx: {e}"))?;
        block_diff.merge(cb_diff);

        // 8. Commit state changes
        self.utxo_set = temp_utxo_set;
        self.block_diffs.push(block_diff);
        self.mempool.remove_confirmed(&block.transactions);
        self.blocks.push(block);

        Ok(())
    }

    /// Reverts the most recent block, restoring the previous UTXO state atomically (chain reorganization).
    pub fn revert_last_block(&mut self) -> Result<Block, String> {
        if self.blocks.len() <= 1 {
            return Err("Cannot revert the Genesis block".to_string());
        }

        let block = self.blocks.pop().unwrap();
        let diff = self.block_diffs.pop().unwrap();

        self.utxo_set
            .revert_diff(&diff)
            .map_err(|e| format!("Revert failed: {e}"))?;

        Ok(block)
    }

    /// Mines a new block from the current mempool and adds it to the chain.
    pub fn mine_pending_transactions(
        &mut self,
        miner_address: &str,
        miner_pkh: [u8; 20],
    ) -> Result<Block, String> {
        let (selected_txs, total_fees) = self.mempool.get_block_template(1_000_000);
        let next_height = self.height() + 1;

        let coinbase_tx = create_coinbase_tx(
            miner_address,
            miner_pkh,
            next_height,
            total_fees,
            rand::random(),
        );

        let mut all_txs = Vec::with_capacity(1 + selected_txs.len());
        all_txs.push(coinbase_tx);
        all_txs.extend(selected_txs);

        let mut block = Block::new(self.tip_hash(), all_txs, self.mining_difficulty_zeros as u32);
        block.mine(self.mining_difficulty_zeros);

        self.add_block(block.clone())?;
        Ok(block)
    }

    /// Helper for creating, signing, and broadcasting a standard payment transaction.
    pub fn create_and_send_transaction(
        &mut self,
        sender_key: &KeyPair,
        recipient_address: &str,
        recipient_pkh: [u8; 20],
        amount: u64,
        fee: u64,
    ) -> Result<Transaction, String> {
        let sender_pkh = sender_key.pubkey_hash();
        let sender_address = sender_key.address();

        // 1. Coin selection: gather UTXOs belonging to sender
        let available_utxos = self.utxo_set.get_utxos_for_address(&sender_address);
        let total_needed = amount.checked_add(fee).ok_or("Amount + fee overflow")?;

        let mut selected_utxos = Vec::new();
        let mut accumulated_value = 0u64;

        for (outpoint, entry) in available_utxos {
            // Check coinbase maturity if applicable
            if entry.is_coinbase && self.height().saturating_sub(entry.block_height) < 10 {
                continue;
            }

            selected_utxos.push((outpoint, entry.clone()));
            accumulated_value += entry.output.value;
            if accumulated_value >= total_needed {
                break;
            }
        }

        if accumulated_value < total_needed {
            return Err(format!(
                "Insufficient funds: available {} sat, needed {} sat (amount: {}, fee: {})",
                accumulated_value, total_needed, amount, fee
            ));
        }

        // 2. Prepare outputs
        let mut outputs = vec![TxOut::new(amount, recipient_address.to_string(), recipient_pkh)];

        // Change output
        let change = accumulated_value - total_needed;
        if change > 0 {
            outputs.push(TxOut::new(change, sender_address.clone(), sender_pkh));
        }

        // 3. Prepare unsigned inputs
        let mut inputs = Vec::new();
        for (outpoint, _) in &selected_utxos {
            inputs.push(TxIn::new(*outpoint, TxIn::FINAL_SEQUENCE));
        }

        let mut tx = Transaction::new(inputs, outputs, 0);

        // 4. Sign each input with sender's key
        let public_key_bytes = sender_key.public_key_bytes();
        for (i, (_, entry)) in selected_utxos.iter().enumerate() {
            let sighash = tx
                .signature_hash(i, &entry.output.pubkey_hash)
                .map_err(|e| format!("Failed to generate sighash: {e}"))?;
            let signature = sender_key
                .sign_digest(&sighash)
                .map_err(|e| format!("Signing error: {e}"))?;

            tx.inputs[i].signature = signature;
            tx.inputs[i].public_key = public_key_bytes.clone();
        }

        // 5. Broadcast to mempool
        self.mempool
            .add_transaction(tx.clone(), &self.utxo_set, self.height() + 1)
            .map_err(|e| format!("Mempool rejected transaction: {e}"))?;

        Ok(tx)
    }
}
