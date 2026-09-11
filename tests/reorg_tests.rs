use utxo_engine::blockchain::Blockchain;
use utxo_engine::crypto::secp256k1::KeyPair;

#[test]
fn test_atomic_chain_reorganization_and_rollback() {
    let alice = KeyPair::generate();
    let bob = KeyPair::generate();

    let mut chain = Blockchain::new(&alice.address(), alice.pubkey_hash());

    // Mine 10 blocks for maturity
    for _ in 0..10 {
        chain
            .mine_pending_transactions(&alice.address(), alice.pubkey_hash())
            .unwrap();
    }

    let alice_balance_before = chain.utxo_set.get_balance(&alice.address());
    let bob_balance_before = chain.utxo_set.get_balance(&bob.address());
    assert_eq!(bob_balance_before, 0);

    // Alice sends 2,500,000 to Bob
    chain
        .create_and_send_transaction(
            &alice,
            &bob.address(),
            bob.pubkey_hash(),
            2_500_000,
            10_000,
        )
        .expect("Send failed");

    // Mine block 11 containing the transaction
    let block11 = chain
        .mine_pending_transactions(&alice.address(), alice.pubkey_hash())
        .expect("Mine failed");

    assert_eq!(chain.height(), 11);
    let bob_balance_after = chain.utxo_set.get_balance(&bob.address());
    assert_eq!(bob_balance_after, 2_500_000);

    // Now simulate a chain reorganization: revert block 11!
    let reverted_block = chain.revert_last_block().expect("Revert failed");
    assert_eq!(reverted_block.hash(), block11.hash());
    assert_eq!(chain.height(), 10);

    // Check that UTXO state is rolled back cleanly:
    // 1. Bob's balance must be back to 0
    assert_eq!(chain.utxo_set.get_balance(&bob.address()), 0);

    // 2. Alice's balance must be fully restored!
    assert_eq!(
        chain.utxo_set.get_balance(&alice.address()),
        alice_balance_before
    );
}
