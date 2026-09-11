use crate::crypto::sha256::hash256_combine;
use crate::primitives::hash::Hash;

/// Represents a single step in a Merkle Inclusion Proof.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MerkleProofStep {
    pub sibling: Hash,
    pub is_sibling_left: bool,
}

/// A Merkle Inclusion Proof proving a transaction hash belongs to a Merkle Root.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MerkleProof {
    pub target_leaf: Hash,
    pub steps: Vec<MerkleProofStep>,
}

impl MerkleProof {
    /// Verifies if this proof leads to the specified Merkle Root.
    pub fn verify(&self, expected_root: &Hash) -> bool {
        let mut current = self.target_leaf;
        for step in &self.steps {
            if step.is_sibling_left {
                current = hash256_combine(&step.sibling, &current);
            } else {
                current = hash256_combine(&current, &step.sibling);
            }
        }
        current == *expected_root
    }
}

/// Computes the Bitcoin-style Merkle Root from a slice of transaction hashes.
/// If empty, returns Hash::ZERO.
/// If a layer has an odd number of elements, the last element is duplicated.
pub fn compute_merkle_root(hashes: &[Hash]) -> Hash {
    if hashes.is_empty() {
        return Hash::ZERO;
    }
    if hashes.len() == 1 {
        return hashes[0];
    }

    let mut current_layer: Vec<Hash> = hashes.to_vec();

    while current_layer.len() > 1 {
        let mut next_layer = Vec::with_capacity((current_layer.len() + 1) / 2);

        for chunk in current_layer.chunks(2) {
            let left = chunk[0];
            let right = if chunk.len() == 2 { chunk[1] } else { chunk[0] };
            next_layer.push(hash256_combine(&left, &right));
        }

        current_layer = next_layer;
    }

    current_layer[0]
}

/// Generates a Merkle Proof for a given leaf index in the transaction list.
pub fn generate_merkle_proof(hashes: &[Hash], target_index: usize) -> Option<MerkleProof> {
    if hashes.is_empty() || target_index >= hashes.len() {
        return None;
    }

    let target_leaf = hashes[target_index];
    let mut steps = Vec::new();
    let mut current_layer: Vec<Hash> = hashes.to_vec();
    let mut current_idx = target_index;

    while current_layer.len() > 1 {
        let pair_idx = if current_idx % 2 == 0 {
            if current_idx + 1 < current_layer.len() {
                current_idx + 1
            } else {
                current_idx // duplicate last
            }
        } else {
            current_idx - 1
        };

        let is_sibling_left = pair_idx < current_idx;
        steps.push(MerkleProofStep {
            sibling: current_layer[pair_idx],
            is_sibling_left,
        });

        // Compute next layer
        let mut next_layer = Vec::with_capacity((current_layer.len() + 1) / 2);
        for chunk in current_layer.chunks(2) {
            let left = chunk[0];
            let right = if chunk.len() == 2 { chunk[1] } else { chunk[0] };
            next_layer.push(hash256_combine(&left, &right));
        }

        current_layer = next_layer;
        current_idx /= 2;
    }

    Some(MerkleProof {
        target_leaf,
        steps,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_merkle_root_single() {
        let h1 = Hash([1u8; 32]);
        assert_eq!(compute_merkle_root(&[h1]), h1);
    }

    #[test]
    fn test_merkle_proof_verification() {
        let h1 = Hash([1u8; 32]);
        let h2 = Hash([2u8; 32]);
        let h3 = Hash([3u8; 32]);
        let hashes = vec![h1, h2, h3];

        let root = compute_merkle_root(&hashes);

        for (i, h) in hashes.iter().enumerate() {
            let proof = generate_merkle_proof(&hashes, i).expect("Proof generation failed");
            assert_eq!(proof.target_leaf, *h);
            assert!(proof.verify(&root), "Proof for leaf {i} must verify against root");
        }
    }
}
