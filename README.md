======================================================================
        🦀 RUST BLOCKCHAIN UTXO ENGINE 🦀
   Double-SHA256 • Secp256k1 • PoW • Mempool • Atomic Reorg
======================================================================

🔑 [1/8] Generating Secp256k1 Cryptographic Keypairs...
   👤 Alice:   1LsuzsofK29kfKcSujd94Re3PogBg7mhjR
   👤 Bob:     14qvyfrnCUKTWkdhAecTVUeyEM4fHypVCc
   👤 Charlie: 1NVTVcVRFnc2rNjZFFRVVkPFsqDQQe4Rbw
──────────────────────────────────────────────────────────────────────
⚡ [2/8] Creating Genesis Block with Miner Alice...
   📦 Genesis Hash:  00afc01a19c37db888fa6af06452ee1c9143147bff59f4cb5614599ba684206b
   🌳 Merkle Root:   7056f7aecf18a615f03bbf435d455356626fa9adfebb1decb249f38a9e5dbb26
   ⛏️  Nonce:         518
   💰 Alice Balance: 5000000000 sat (50.00000000 BTC)
──────────────────────────────────────────────────────────────────────
⛏️  [3/8] Mining 10 Blocks to achieve Coinbase Maturity (10 confirmations)...
   🧱 Block #1  mined! Hash: 008a55d29a6da8ac... Nonce: 202
   🧱 Block #2  mined! Hash: 0030a932e7642d3e... Nonce: 143
   🧱 Block #3  mined! Hash: 00add0052370392a... Nonce: 137
   🧱 Block #4  mined! Hash: 00344189278a6bc7... Nonce: 510
   🧱 Block #5  mined! Hash: 00a649859b517577... Nonce: 919
   🧱 Block #6  mined! Hash: 00c9e29e55a95478... Nonce: 146
   🧱 Block #7  mined! Hash: 00cbb172c8a972e8... Nonce: 275
   🧱 Block #8  mined! Hash: 00c97603c9da01f8... Nonce: 186
   🧱 Block #9  mined! Hash: 000bbe6b037cd417... Nonce: 82
   🧱 Block #10 mined! Hash: 0053e9b354cf5f31... Nonce: 155
   ⏱️  Total mining time: 24.24ms
   📊 Current Height:    10
   💰 Alice Total Mined: 55000000000 sat (550.00000000 BTC)
──────────────────────────────────────────────────────────────────────
💸 [4/8] Alice Sending 1500000000 sat (15.00000000 BTC) to Bob (Fee: 10000 sat (0.00010000 BTC))...
   📝 TxID:            96eac8bacd3961f6589f2fe20a9e0b3a4c6d0de486747c7cedc68f3edd2fff73
   📥 Inputs count:    1
   📤 Outputs count:   2
   📨 Mempool Status:  1 pending transaction(s)
──────────────────────────────────────────────────────────────────────
🛡️  [5/8] Simulating DOUBLE-SPEND ATTACK...
   Alice attempts to spend the SAME input(s) again to Charlie...
   ✅ SUCCESS: Double spend was immediately REJECTED by Validation Engine!
   🛑 Reason: Mempool rejected transaction: Double spend detected within transaction: input 7056f7aecf18a615f03bbf435d455356626fa9adfebb1decb249f38a9e5dbb26:0 used multiple times
──────────────────────────────────────────────────────────────────────
⛏️  [6/8] Mining Block #11 to confirm transactions...
   🧱 Block #11 Hash:       0065c2d18749ed501dfb3a180ac6e8ddb83f6dce91cc920e1d75830719e1ef1a
   📦 Transactions in Block: 2
   📨 Mempool after block:   0 pending
   💰 Bob Confirmed Balance: 1500000000 sat (15.00000000 BTC)
──────────────────────────────────────────────────────────────────────
🔄 [7/8] Bob forwards 500000000 sat (5.00000000 BTC) to Charlie (Fee: 5000 sat (0.00005000 BTC))...
   💰 Bob Final Balance:     999995000 sat (9.99995000 BTC)
   💰 Charlie Final Balance: 500000000 sat (5.00000000 BTC)
   🪙 Total UTXO Coin Supply: 65000000000 sat (650.00000000 BTC)
   🗄️  Total Unspent UTXOs:    15
──────────────────────────────────────────────────────────────────────
⏪ [8/8] Testing CHAIN REORGANIZATION & ATOMIC ROLLBACK...
   Reverting the most recent Block #12...
   🔄 Reverted Block Hash: 00bb8e8a1cca8746175bc308ca66430943edbe9c341cadeae6d153f4d7dfa67d
   📊 Restored Chain Height: 11
   💰 Charlie Balance after Reorg: 0 sat (0.00000000 BTC)
   💰 Bob Balance after Reorg:     1500000000 sat (15.00000000 BTC)
   ✅ All UTXO states rolled back with 100% mathematical precision via UtxoDiff!
──────────────────────────────────────────────────────────────────────
🌲 [Bonus] Demonstrating Merkle Inclusion Proof...
   Root:  085aabaef98668701b87c9a1986bdf116726a9949802326b69895697d4e8c812
   Leaf:  0303030303030303030303030303030303030303030303030303030303030303
   Proof: 2 steps. Validated: true

🚀 Running UTXO Engine Performance Benchmark...
   ⚡ Inserted 10000 UTXOs in 8.24ms (1213946 operations/sec)

======================================================================
   ✨ All UTXO Engine modules, consensus, and security checks PASSED! ✨
======================================================================
