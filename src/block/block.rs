use crate::crypto::sha256::dsha256;
use crate::merkle::tree::compute_merkle_root;
use crate::primitives::hash::Hash;
use crate::transaction::coinbase::create_coinbase_tx;
use crate::transaction::tx::Transaction;
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

/// Block Header containing cryptographic summary and consensus metadata.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct BlockHeader {
    pub version: u32,
    pub prev_block_hash: Hash,
    pub merkle_root: Hash,
    pub timestamp: u64,
    pub bits: u32,
    pub nonce: u64,
}

impl BlockHeader {
    pub fn new(
        prev_block_hash: Hash,
        merkle_root: Hash,
        bits: u32,
        timestamp: u64,
    ) -> Self {
        Self {
            version: 1,
            prev_block_hash,
            merkle_root,
            timestamp,
            bits,
            nonce: 0,
        }
    }

    /// Computes the Double-SHA256 block hash over the header.
    pub fn hash(&self) -> Hash {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&self.version.to_le_bytes());
        bytes.extend_from_slice(self.prev_block_hash.as_bytes());
        bytes.extend_from_slice(self.merkle_root.as_bytes());
        bytes.extend_from_slice(&self.timestamp.to_le_bytes());
        bytes.extend_from_slice(&self.bits.to_le_bytes());
        bytes.extend_from_slice(&self.nonce.to_le_bytes());
        dsha256(&bytes)
    }
}

/// A complete block in the blockchain containing the header and ordered transactions.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Block {
    pub header: BlockHeader,
    pub transactions: Vec<Transaction>,
}

impl Block {
    pub fn new(
        prev_block_hash: Hash,
        transactions: Vec<Transaction>,
        difficulty_target: u32,
    ) -> Self {
        let txids: Vec<Hash> = transactions.iter().map(|tx| tx.txid()).collect();
        let merkle_root = compute_merkle_root(&txids);
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let header = BlockHeader::new(prev_block_hash, merkle_root, difficulty_target, timestamp);

        Self {
            header,
            transactions,
        }
    }

    pub fn hash(&self) -> Hash {
        self.header.hash()
    }

    /// Verifies that the block's transactions match the header's merkle root.
    pub fn verify_merkle_root(&self) -> bool {
        let txids: Vec<Hash> = self.transactions.iter().map(|tx| tx.txid()).collect();
        let calculated = compute_merkle_root(&txids);
        calculated == self.header.merkle_root
    }

    /// Checks if the block hash satisfies the difficulty target (leading zero bytes).
    pub fn verify_pow(&self, target_leading_zeros: usize) -> bool {
        let hash = self.hash();
        let bytes = hash.as_bytes();
        for &b in &bytes[..target_leading_zeros] {
            if b != 0 {
                return false;
            }
        }
        true
    }

    /// Performs Proof of Work mining by incrementing nonce until target leading zero bytes are achieved.
    pub fn mine(&mut self, target_leading_zeros: usize) -> (Hash, u64) {
        let mut attempts = 0u64;
        loop {
            if self.verify_pow(target_leading_zeros) {
                return (self.hash(), attempts);
            }
            self.header.nonce = self.header.nonce.wrapping_add(1);
            attempts += 1;
        }
    }

    /// Creates the Genesis block (Block 0) of the blockchain.
    pub fn genesis(miner_address: &str, miner_pkh: [u8; 20]) -> Self {
        let coinbase_tx = create_coinbase_tx(miner_address, miner_pkh, 0, 0, 0);
        let mut block = Self::new(Hash::ZERO, vec![coinbase_tx], 1);
        // Mine genesis block with 1 zero byte
        block.mine(1);
        block
    }
}
