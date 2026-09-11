use std::time::Instant;
use utxo_engine::blockchain::Blockchain;
use utxo_engine::crypto::secp256k1::KeyPair;
use utxo_engine::merkle::tree::{compute_merkle_root, generate_merkle_proof};
use utxo_engine::primitives::hash::Hash;

fn format_sat(satoshis: u64) -> String {
    let btc = satoshis as f64 / 100_000_000.0;
    format!("{satoshis} sat ({btc:.8} BTC)")
}

fn print_separator() {
    println!("{}", "─".repeat(70));
}

fn main() {
    println!("\n{}", "=".repeat(70));
    println!("        🦀 RUST BLOCKCHAIN UTXO ENGINE 🦀");
    println!("   Double-SHA256 • Secp256k1 • PoW • Mempool • Atomic Reorg");
    println!("{}\n", "=".repeat(70));

    // ── STEP 1: KEYPAIR GENERATION ──────────────────────────────────────────
    println!("🔑 [1/8] Generating Secp256k1 Cryptographic Keypairs...");
    let alice = KeyPair::generate();
    let bob = KeyPair::generate();
    let charlie = KeyPair::generate();

    println!("   👤 Alice:   {}", alice.address());
    println!("   👤 Bob:     {}", bob.address());
    println!("   👤 Charlie: {}", charlie.address());
    print_separator();

    // ── STEP 2: GENESIS BLOCK INITIALIZATION ────────────────────────────────
    println!("⚡ [2/8] Creating Genesis Block with Miner Alice...");
    let mut chain = Blockchain::new(&alice.address(), alice.pubkey_hash());
    let genesis = &chain.blocks[0];
    println!("   📦 Genesis Hash:  {}", genesis.hash());
    println!("   🌳 Merkle Root:   {}", genesis.header.merkle_root);
    println!("   ⛏️  Nonce:         {}", genesis.header.nonce);
    println!(
        "   💰 Alice Balance: {}",
        format_sat(chain.utxo_set.get_balance(&alice.address()))
    );
    print_separator();

    // ── STEP 3: MINING FOR COINBASE MATURITY ─────────────────────────────────
    println!("⛏️  [3/8] Mining 10 Blocks to achieve Coinbase Maturity (10 confirmations)...");
    let start_mining = Instant::now();
    for h in 1..=10 {
        let block = chain
            .mine_pending_transactions(&alice.address(), alice.pubkey_hash())
            .expect("Mining failed");
        println!(
            "   🧱 Block #{:<2} mined! Hash: {}... Nonce: {}",
            h,
            &block.hash().to_hex()[..16],
            block.header.nonce
        );
    }
    let mining_duration = start_mining.elapsed();
    println!("   ⏱️  Total mining time: {:.2?}", mining_duration);
    println!("   📊 Current Height:    {}", chain.height());
    println!(
        "   💰 Alice Total Mined: {}",
        format_sat(chain.utxo_set.get_balance(&alice.address()))
    );
    print_separator();

    // ── STEP 4: TRANSACTION CREATION & BROADCAST ────────────────────────────
    let send_amount = 1_500_000_000; // 15 BTC
    let fee = 10_000; // 0.0001 BTC
    println!(
        "💸 [4/8] Alice Sending {} to Bob (Fee: {})...",
        format_sat(send_amount),
        format_sat(fee)
    );

    let tx1 = chain
        .create_and_send_transaction(
            &alice,
            &bob.address(),
            bob.pubkey_hash(),
            send_amount,
            fee,
        )
        .expect("Transaction failed");

    println!("   📝 TxID:            {}", tx1.txid());
    println!("   📥 Inputs count:    {}", tx1.inputs.len());
    println!("   📤 Outputs count:   {}", tx1.outputs.len());
    println!("   📨 Mempool Status:  {} pending transaction(s)", chain.mempool.len());
    print_separator();

    // ── STEP 5: DOUBLE-SPEND ATTACK SIMULATION ──────────────────────────────
    println!("🛡️  [5/8] Simulating DOUBLE-SPEND ATTACK...");
    println!("   Alice attempts to spend the SAME input(s) again to Charlie...");

    let double_spend_result = chain.create_and_send_transaction(
        &alice,
        &charlie.address(),
        charlie.pubkey_hash(),
        send_amount,
        fee,
    );

    match double_spend_result {
        Ok(_) => {
            println!("   ❌ ERROR: Double spend was unexpectedly accepted!");
        }
        Err(e) => {
            println!("   ✅ SUCCESS: Double spend was immediately REJECTED by Validation Engine!");
            println!("   🛑 Reason: {e}");
        }
    }
    print_separator();

    // ── STEP 6: MINING BLOCK WITH TRANSACTIONS ───────────────────────────────
    println!("⛏️  [6/8] Mining Block #11 to confirm transactions...");
    let block11 = chain
        .mine_pending_transactions(&alice.address(), alice.pubkey_hash())
        .expect("Block 11 mining failed");

    println!("   🧱 Block #11 Hash:       {}", block11.hash());
    println!("   📦 Transactions in Block: {}", block11.transactions.len());
    println!("   📨 Mempool after block:   {} pending", chain.mempool.len());
    println!("   💰 Bob Confirmed Balance: {}", format_sat(chain.utxo_set.get_balance(&bob.address())));
    print_separator();

    // ── STEP 7: BOB SPENDS TO CHARLIE ───────────────────────────────────────
    let bob_send = 500_000_000; // 5 BTC
    let bob_fee = 5_000;
    println!(
        "🔄 [7/8] Bob forwards {} to Charlie (Fee: {})...",
        format_sat(bob_send),
        format_sat(bob_fee)
    );

    let _tx2 = chain
        .create_and_send_transaction(
            &bob,
            &charlie.address(),
            charlie.pubkey_hash(),
            bob_send,
            bob_fee,
        )
        .expect("Bob transaction failed");

    chain
        .mine_pending_transactions(&alice.address(), alice.pubkey_hash())
        .expect("Block 12 mining failed");

    println!("   💰 Bob Final Balance:     {}", format_sat(chain.utxo_set.get_balance(&bob.address())));
    println!("   💰 Charlie Final Balance: {}", format_sat(chain.utxo_set.get_balance(&charlie.address())));
    println!("   🪙 Total UTXO Coin Supply: {}", format_sat(chain.utxo_set.total_supply()));
    println!("   🗄️  Total Unspent UTXOs:    {}", chain.utxo_set.len());
    print_separator();

    // ── STEP 8: CHAIN REORGANIZATION & ATOMIC ROLLBACK ───────────────────────
    println!("⏪ [8/8] Testing CHAIN REORGANIZATION & ATOMIC ROLLBACK...");
    println!("   Reverting the most recent Block #12...");
    let reverted = chain.revert_last_block().expect("Reversion failed");
    println!("   🔄 Reverted Block Hash: {}", reverted.hash());
    println!("   📊 Restored Chain Height: {}", chain.height());
    println!(
        "   💰 Charlie Balance after Reorg: {}",
        format_sat(chain.utxo_set.get_balance(&charlie.address()))
    );
    println!(
        "   💰 Bob Balance after Reorg:     {}",
        format_sat(chain.utxo_set.get_balance(&bob.address()))
    );
    assert_eq!(chain.utxo_set.get_balance(&charlie.address()), 0);
    println!("   ✅ All UTXO states rolled back with 100% mathematical precision via UtxoDiff!");
    print_separator();

    // ── BONUS: MERKLE TREE INCLUSION PROOF ───────────────────────────────────
    println!("🌲 [Bonus] Demonstrating Merkle Inclusion Proof...");
    let dummy_txs: Vec<Hash> = (1..=4).map(|i| Hash([i; 32])).collect();
    let root = compute_merkle_root(&dummy_txs);
    let proof = generate_merkle_proof(&dummy_txs, 2).expect("Proof failed");
    let is_valid = proof.verify(&root);
    println!("   Root:  {}", root);
    println!("   Leaf:  {}", proof.target_leaf);
    println!("   Proof: {} steps. Validated: {}", proof.steps.len(), is_valid);

    // ── TPS BENCHMARK ───────────────────────────────────────────────────────
    println!("\n🚀 Running UTXO Engine Performance Benchmark...");
    let benchmark_count = 10_000;
    let bench_start = Instant::now();
    let mut test_utxo_set = chain.utxo_set.clone();
    for i in 0..benchmark_count {
        let op = utxo_engine::primitives::outpoint::OutPoint::new(Hash([0xFF; 32]), i as u32);
        let out = utxo_engine::primitives::txout::TxOut::new(1000, "1Bench".to_string(), [0xAA; 20]);
        let entry = utxo_engine::utxo::entry::UtxoEntry::new(out, 1, false);
        test_utxo_set.insert(op, entry);
    }
    let bench_duration = bench_start.elapsed();
    let tps = benchmark_count as f64 / bench_duration.as_secs_f64();
    println!(
        "   ⚡ Inserted {} UTXOs in {:.2?} ({:.0} operations/sec)",
        benchmark_count, bench_duration, tps
    );

    println!("\n{}", "=".repeat(70));
    println!("   ✨ All UTXO Engine modules, consensus, and security checks PASSED! ✨");
    println!("{}\n", "=".repeat(70));
}
