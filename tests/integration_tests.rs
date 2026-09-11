use utxo_engine::blockchain::Blockchain;
use utxo_engine::crypto::secp256k1::KeyPair;

#[test]
fn test_end_to_end_blockchain_flow() {
    let alice = KeyPair::generate();
    let bob = KeyPair::generate();
    let charlie = KeyPair::generate();

    // 1. Initialize chain with Alice as Genesis miner
    let mut chain = Blockchain::new(&alice.address(), alice.pubkey_hash());
    assert_eq!(chain.height(), 0);

    // 2. Mine 10 blocks so Alice's coinbase matures (maturity = 10 blocks)
    for _ in 0..10 {
        chain
            .mine_pending_transactions(&alice.address(), alice.pubkey_hash())
            .expect("Mining failed");
    }
    assert_eq!(chain.height(), 10);

    let alice_initial_balance = chain.utxo_set.get_balance(&alice.address());
    assert!(alice_initial_balance > 0, "Alice should have mined coins");

    // 3. Alice sends 1,000,000 satoshis to Bob with 5,000 fee
    let _tx_alice_to_bob = chain
        .create_and_send_transaction(
            &alice,
            &bob.address(),
            bob.pubkey_hash(),
            1_000_000,
            5_000,
        )
        .expect("Alice to Bob transaction failed");

    assert_eq!(chain.mempool.len(), 1);

    // 4. Test Double Spend detection:
    // Alice attempts to use the same input in another transaction before the block is mined
    let duplicate_tx = chain.create_and_send_transaction(
        &alice,
        &charlie.address(),
        charlie.pubkey_hash(),
        1_000_000,
        5_000,
    );
    assert!(
        duplicate_tx.is_err(),
        "Mempool must reject double-spending the same UTXO"
    );

    // 5. Mine block to confirm Alice -> Bob
    let block11 = chain
        .mine_pending_transactions(&alice.address(), alice.pubkey_hash())
        .expect("Mining block 11 failed");

    assert_eq!(chain.height(), 11);
    assert_eq!(chain.mempool.len(), 0);
    assert_eq!(block11.transactions.len(), 2); // Coinbase + Alice->Bob

    // 6. Verify Bob's confirmed balance
    let bob_balance = chain.utxo_set.get_balance(&bob.address());
    assert_eq!(bob_balance, 1_000_000);

    // 7. Bob sends 400,000 satoshis to Charlie with 2,000 fee
    let _tx_bob_to_charlie = chain
        .create_and_send_transaction(
            &bob,
            &charlie.address(),
            charlie.pubkey_hash(),
            400_000,
            2_000,
        )
        .expect("Bob to Charlie transaction failed");

    chain
        .mine_pending_transactions(&alice.address(), alice.pubkey_hash())
        .expect("Mining block 12 failed");

    // 8. Final balance checks
    let charlie_balance = chain.utxo_set.get_balance(&charlie.address());
    let bob_final_balance = chain.utxo_set.get_balance(&bob.address());

    assert_eq!(charlie_balance, 400_000);
    assert_eq!(bob_final_balance, 1_000_000 - 400_000 - 2_000); // 598,000
}
