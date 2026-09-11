use utxo_engine::crypto::ripemd160::address_to_pubkey_hash;
use utxo_engine::crypto::secp256k1::{verify_signature, KeyPair};
use utxo_engine::crypto::sha256::{dsha256, sha256};
use utxo_engine::merkle::tree::{compute_merkle_root, generate_merkle_proof};
use utxo_engine::primitives::hash::Hash;
use utxo_engine::primitives::outpoint::OutPoint;
use utxo_engine::primitives::txin::TxIn;
use utxo_engine::primitives::txout::TxOut;
use utxo_engine::transaction::tx::Transaction;
use utxo_engine::utxo::entry::UtxoEntry;
use utxo_engine::utxo::set::UtxoSet;
use utxo_engine::validation::error::ValidationError;
use utxo_engine::validation::validate_transaction;

#[test]
fn test_hash_primitives() {
    let empty_sha = sha256(b"");
    assert_eq!(
        empty_sha.to_hex(),
        "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
    );

    let parsed = Hash::from_hex(&empty_sha.to_hex()).unwrap();
    assert_eq!(empty_sha, parsed);
}

#[test]
fn test_secp256k1_signing_and_verification() {
    let alice = KeyPair::generate();
    let message_hash = dsha256(b"Authorize UTXO Transfer");

    let sig = alice.sign_digest(&message_hash).expect("Signing failed");
    let pubkey = alice.public_key_bytes();

    let valid = verify_signature(&pubkey, &message_hash, &sig).expect("Verification error");
    assert!(valid, "Valid signature must pass");

    let bob = KeyPair::generate();
    let bob_pubkey = bob.public_key_bytes();
    let invalid = verify_signature(&bob_pubkey, &message_hash, &sig).expect("Verification error");
    assert!(!invalid, "Signature by Alice must not verify under Bob's pubkey");
}

#[test]
fn test_address_base58check() {
    let kp = KeyPair::generate();
    let addr = kp.address();
    assert!(addr.starts_with('1'), "P2PKH address should start with 1");

    let (decoded_pkh, ver) = address_to_pubkey_hash(&addr).expect("Decode failed");
    assert_eq!(ver, 0x00);
    assert_eq!(decoded_pkh, kp.pubkey_hash());
}

#[test]
fn test_merkle_tree_proofs() {
    let leaves: Vec<Hash> = (0..5).map(|i| dsha256(&[i as u8])).collect();
    let root = compute_merkle_root(&leaves);

    for (idx, leaf) in leaves.iter().enumerate() {
        let proof = generate_merkle_proof(&leaves, idx).expect("Proof generation failed");
        assert_eq!(proof.target_leaf, *leaf);
        assert!(proof.verify(&root), "Proof for leaf {idx} must be valid");
    }
}

#[test]
fn test_validation_detects_duplicate_inputs() {
    let op = OutPoint::new(Hash([1u8; 32]), 0);
    let in1 = TxIn::new(op, 0);
    let in2 = TxIn::new(op, 0);
    let out = TxOut::new(100, "1Addr".to_string(), [0u8; 20]);

    let tx = Transaction::new(vec![in1, in2], vec![out], 0);
    let utxo_set = UtxoSet::new();

    let res = validate_transaction(&tx, &utxo_set, 1);
    assert_eq!(res, Err(ValidationError::DuplicateInput(op)));
}

#[test]
fn test_validation_detects_missing_utxo() {
    let _alice = KeyPair::generate();
    let op = OutPoint::new(Hash([99u8; 32]), 0);
    let tx_in = TxIn::new(op, 0);
    let tx_out = TxOut::new(500, "1Bob".to_string(), [0u8; 20]);

    let tx = Transaction::new(vec![tx_in], vec![tx_out], 0);
    let utxo_set = UtxoSet::new();

    let res = validate_transaction(&tx, &utxo_set, 1);
    assert_eq!(res, Err(ValidationError::UtxoNotFound(op)));
}

#[test]
fn test_validation_insufficient_funds() {
    let alice = KeyPair::generate();
    let op = OutPoint::new(Hash([7u8; 32]), 0);
    let mut utxo_set = UtxoSet::new();
    utxo_set.insert(
        op,
        UtxoEntry::new(
            TxOut::new(1000, alice.address(), alice.pubkey_hash()),
            1,
            false,
        ),
    );

    let in1 = TxIn::new(op, 0);
    // Asking for 1500 when input is 1000
    let out1 = TxOut::new(1500, "1Bob".to_string(), [0u8; 20]);
    let tx = Transaction::new(vec![in1], vec![out1], 0);

    let res = validate_transaction(&tx, &utxo_set, 2);
    assert_eq!(
        res,
        Err(ValidationError::InsufficientFunds {
            input_total: 1000,
            output_total: 1500
        })
    );
}
