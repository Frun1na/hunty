//! Tests for the nft-reward contract.
//!
//! # Issue #848 — consistency test
//!
//! The primary requirement from issue #848 is a test asserting that the owner
//! index is internally consistent (count matches enumerable entries, exist-keys
//! match) after an interleaved mint / transfer / burn sequence.
//!
//! All verification goes through the contract's public client API rather than
//! reaching into `storage` directly, since Soroban SDK v22 forbids storage
//! access outside of a contract execution context.

#[cfg(test)]
mod nft_reward_tests {
    use soroban_sdk::{testutils::Address as _, Address, Env, String};

    use crate::{NftRewardContract, NftRewardContractClient};

    // ── helpers ───────────────────────────────────────────────────────────────

    /// Assert that the owner index for `owner` is internally consistent,
    /// using only the contract's public API:
    ///
    /// 1. `balance_of` equals the length of `get_player_nfts`.
    /// 2. No duplicate ids appear in `get_player_nfts`.
    fn assert_owner_index_consistent(client: &NftRewardContractClient, owner: &Address) {
        let count = client.balance_of(owner);
        let nfts = client.get_player_nfts(owner);

        assert_eq!(
            nfts.len(),
            count,
            "balance_of={count} but get_player_nfts returned {} ids",
            nfts.len()
        );

        // no duplicates
        for i in 0..nfts.len() {
            for j in (i + 1)..nfts.len() {
                assert_ne!(
                    nfts.get(i).unwrap(),
                    nfts.get(j).unwrap(),
                    "duplicate id in get_player_nfts at positions {i} and {j}"
                );
            }
        }
    }

    /// Assert that `nft_id` is NOT present in `owner`'s NFT list.
    fn assert_nft_absent(client: &NftRewardContractClient, owner: &Address, nft_id: u64) {
        let nfts = client.get_player_nfts(owner);
        for i in 0..nfts.len() {
            assert_ne!(
                nfts.get(i).unwrap(),
                nft_id,
                "id {nft_id} still present in get_player_nfts at index {i} after removal"
            );
        }
    }

    /// Assert that `nft_id` IS present in `owner`'s NFT list.
    fn assert_nft_present(client: &NftRewardContractClient, owner: &Address, nft_id: u64) {
        let nfts = client.get_player_nfts(owner);
        let found = (0..nfts.len()).any(|i| nfts.get(i).unwrap() == nft_id);
        assert!(
            found,
            "id {nft_id} expected in get_player_nfts but not found"
        );
    }

    fn setup(env: &Env) -> (Address, NftRewardContractClient<'_>) {
        let contract_id = env.register(NftRewardContract, ());
        let client = NftRewardContractClient::new(env, &contract_id);
        let minter = Address::generate(env);
        (minter, client)
    }

    fn test_uri(env: &Env, n: u32) -> String {
        let uris = [
            "ipfs://QmTest1",
            "ipfs://QmTest2",
            "ipfs://QmTest3",
            "ipfs://QmTest4",
            "ipfs://QmTest5",
            "ipfs://QmTest6",
            "ipfs://QmTest7",
            "ipfs://QmTest8",
            "ipfs://QmTest9",
        ];
        let idx = ((n.saturating_sub(1)) % 9) as usize;
        String::from_str(env, uris[idx])
    }

    // ── individual operation tests ────────────────────────────────────────────

    #[test]
    fn test_mint_updates_owner_index() {
        let env = Env::default();
        env.mock_all_auths();
        let (minter, client) = setup(&env);
        let player = Address::generate(&env);

        let id = client.mint(&minter, &player, &test_uri(&env, 1));

        assert_eq!(client.balance_of(&player), 1);
        assert_owner_index_consistent(&client, &player);
        assert_eq!(client.get_owner(&id), Some(player));
    }

    #[test]
    fn test_burn_removes_from_owner_index() {
        let env = Env::default();
        env.mock_all_auths();
        let (minter, client) = setup(&env);
        let player = Address::generate(&env);

        let id = client.mint(&minter, &player, &test_uri(&env, 1));
        assert_eq!(client.balance_of(&player), 1);

        client.burn(&player, &id);

        assert_eq!(client.balance_of(&player), 0);
        assert_owner_index_consistent(&client, &player);
        assert_nft_absent(&client, &player, id);
        assert_eq!(client.get_owner(&id), None);
    }

    #[test]
    fn test_transfer_updates_both_indexes() {
        let env = Env::default();
        env.mock_all_auths();
        let (minter, client) = setup(&env);
        let alice = Address::generate(&env);
        let bob = Address::generate(&env);

        let id = client.mint(&minter, &alice, &test_uri(&env, 1));

        client.transfer(&alice, &bob, &id);

        assert_eq!(client.balance_of(&alice), 0);
        assert_eq!(client.balance_of(&bob), 1);
        assert_owner_index_consistent(&client, &alice);
        assert_owner_index_consistent(&client, &bob);
        assert_nft_absent(&client, &alice, id);
        assert_nft_present(&client, &bob, id);
        assert_eq!(client.get_owner(&id), Some(bob));
    }

    // ── issue #848 core test: mint → transfer → burn consistency ─────────────

    /// Interleaved mint / transfer / burn sequence.
    ///
    /// This is the acceptance criterion from issue #848: the owner index must be
    /// internally consistent at every stage — `balance_of` must match the length
    /// of `get_player_nfts`, no duplicates, and burned / transferred ids must
    /// not appear in the former owner's list.
    #[test]
    fn test_mint_transfer_burn_owner_index_consistency() {
        let env = Env::default();
        env.mock_all_auths();
        let (minter, client) = setup(&env);
        let alice = Address::generate(&env);
        let bob = Address::generate(&env);

        // ── 1. Mint three NFTs to alice ───────────────────────────────────────
        let id1 = client.mint(&minter, &alice, &test_uri(&env, 1));
        let id2 = client.mint(&minter, &alice, &test_uri(&env, 2));
        let id3 = client.mint(&minter, &alice, &test_uri(&env, 3));

        assert_eq!(client.balance_of(&alice), 3);
        assert_owner_index_consistent(&client, &alice);

        // ── 2. Transfer id2 from alice to bob ─────────────────────────────────
        client.transfer(&alice, &bob, &id2);

        assert_eq!(client.balance_of(&alice), 2);
        assert_eq!(client.balance_of(&bob), 1);
        assert_owner_index_consistent(&client, &alice);
        assert_owner_index_consistent(&client, &bob);
        assert_nft_absent(&client, &alice, id2);
        assert_nft_present(&client, &alice, id1);
        assert_nft_present(&client, &alice, id3);

        // ── 3. Mint a fourth NFT directly to bob ─────────────────────────────
        let id4 = client.mint(&minter, &bob, &test_uri(&env, 4));

        assert_eq!(client.balance_of(&bob), 2);
        assert_owner_index_consistent(&client, &bob);

        // ── 4. Burn id3 from alice (middle-of-list removal) ──────────────────
        client.burn(&alice, &id3);

        assert_eq!(client.balance_of(&alice), 1);
        assert_owner_index_consistent(&client, &alice);
        assert_nft_absent(&client, &alice, id3);
        assert_nft_present(&client, &alice, id1);

        // ── 5. Burn id1 from alice (last remaining) ──────────────────────────
        client.burn(&alice, &id1);

        assert_eq!(client.balance_of(&alice), 0);
        assert_owner_index_consistent(&client, &alice);
        assert_nft_absent(&client, &alice, id1);

        // ── 6. Bob transfers id4 back to alice, then alice burns it ──────────
        client.transfer(&bob, &alice, &id4);

        assert_eq!(client.balance_of(&bob), 1); // still has id2
        assert_eq!(client.balance_of(&alice), 1);
        assert_owner_index_consistent(&client, &bob);
        assert_owner_index_consistent(&client, &alice);

        client.burn(&alice, &id4);

        assert_eq!(client.balance_of(&alice), 0);
        assert_owner_index_consistent(&client, &alice);
        assert_nft_absent(&client, &alice, id4);

        // ── 7. Bob burns his remaining NFT (id2) ─────────────────────────────
        client.burn(&bob, &id2);

        assert_eq!(client.balance_of(&bob), 0);
        assert_owner_index_consistent(&client, &bob);
        assert_nft_absent(&client, &bob, id2);

        // ── 8. All tokens are gone; total supply is still 4 ──────────────────
        assert_eq!(client.total_supply(), 4);
        assert_eq!(client.get_owner(&id1), None);
        assert_eq!(client.get_owner(&id2), None);
        assert_eq!(client.get_owner(&id3), None);
        assert_eq!(client.get_owner(&id4), None);
    }

    // ── error-path tests ──────────────────────────────────────────────────────

    #[test]
    #[should_panic]
    fn test_burn_wrong_owner_panics() {
        let env = Env::default();
        env.mock_all_auths();
        let (minter, client) = setup(&env);
        let alice = Address::generate(&env);
        let eve = Address::generate(&env);

        let id = client.mint(&minter, &alice, &test_uri(&env, 1));
        client.burn(&eve, &id);
    }

    #[test]
    #[should_panic]
    fn test_burn_nonexistent_panics() {
        let env = Env::default();
        env.mock_all_auths();
        let (minter, client) = setup(&env);
        let alice = Address::generate(&env);

        let id = client.mint(&minter, &alice, &test_uri(&env, 1));
        client.burn(&alice, &id);
        // second burn must fail — token no longer exists
        client.burn(&alice, &id);
    }

    #[test]
    #[should_panic]
    fn test_transfer_wrong_owner_panics() {
        let env = Env::default();
        env.mock_all_auths();
        let (minter, client) = setup(&env);
        let alice = Address::generate(&env);
        let bob = Address::generate(&env);
        let eve = Address::generate(&env);

        let id = client.mint(&minter, &alice, &test_uri(&env, 1));
        // eve claims to be 'from' but is not the owner
        client.transfer(&eve, &bob, &id);
    }

    // ── swap-and-pop edge cases ───────────────────────────────────────────────

    /// Burn the *first* NFT when more are present — exercises moving the last
    /// element into slot 0.
    #[test]
    fn test_burn_first_nft_of_many() {
        let env = Env::default();
        env.mock_all_auths();
        let (minter, client) = setup(&env);
        let player = Address::generate(&env);

        let id1 = client.mint(&minter, &player, &test_uri(&env, 1));
        let id2 = client.mint(&minter, &player, &test_uri(&env, 2));
        let id3 = client.mint(&minter, &player, &test_uri(&env, 3));

        client.burn(&player, &id1);

        assert_eq!(client.balance_of(&player), 2);
        assert_owner_index_consistent(&client, &player);
        assert_nft_absent(&client, &player, id1);
        assert_nft_present(&client, &player, id2);
        assert_nft_present(&client, &player, id3);
    }

    /// Burn the *last* NFT in the list — no swap needed, just pop.
    #[test]
    fn test_burn_last_nft_of_many() {
        let env = Env::default();
        env.mock_all_auths();
        let (minter, client) = setup(&env);
        let player = Address::generate(&env);

        let id1 = client.mint(&minter, &player, &test_uri(&env, 1));
        let id2 = client.mint(&minter, &player, &test_uri(&env, 2));
        let id3 = client.mint(&minter, &player, &test_uri(&env, 3));

        client.burn(&player, &id3);

        assert_eq!(client.balance_of(&player), 2);
        assert_owner_index_consistent(&client, &player);
        assert_nft_absent(&client, &player, id3);
        assert_nft_present(&client, &player, id1);
        assert_nft_present(&client, &player, id2);
    }

    // ── issue #1405 coverage: explicit error results ──────────────────────────

    /// Unauthorized transfer must return `NftError::NotOwner` (not just panic),
    /// and must leave both owner indexes untouched.
    #[test]
    fn test_transfer_wrong_owner_returns_not_owner() {
        use crate::NftError;

        let env = Env::default();
        env.mock_all_auths();
        let (minter, client) = setup(&env);
        let alice = Address::generate(&env);
        let bob = Address::generate(&env);
        let eve = Address::generate(&env);

        let id = client.mint(&minter, &alice, &test_uri(&env, 1));

        let res = client.try_transfer(&eve, &bob, &id);
        assert_eq!(res, Err(Ok(NftError::NotOwner)));

        // nothing moved
        assert_eq!(client.get_owner(&id), Some(alice.clone()));
        assert_eq!(client.balance_of(&alice), 1);
        assert_eq!(client.balance_of(&bob), 0);
        assert_owner_index_consistent(&client, &alice);
    }

    /// Burning a non-owned token must return `NftError::NotOwner` and must not
    /// alter the actual owner's index.
    #[test]
    fn test_burn_wrong_owner_returns_not_owner() {
        use crate::NftError;

        let env = Env::default();
        env.mock_all_auths();
        let (minter, client) = setup(&env);
        let alice = Address::generate(&env);
        let eve = Address::generate(&env);

        let id = client.mint(&minter, &alice, &test_uri(&env, 1));

        let res = client.try_burn(&eve, &id);
        assert_eq!(res, Err(Ok(NftError::NotOwner)));

        // token untouched
        assert_eq!(client.get_owner(&id), Some(alice.clone()));
        assert_eq!(client.balance_of(&alice), 1);
        assert_owner_index_consistent(&client, &alice);
    }

    /// Transferring a token that was already burned returns `TokenNotFound`.
    #[test]
    fn test_transfer_burned_token_returns_token_not_found() {
        use crate::NftError;

        let env = Env::default();
        env.mock_all_auths();
        let (minter, client) = setup(&env);
        let alice = Address::generate(&env);
        let bob = Address::generate(&env);

        let id = client.mint(&minter, &alice, &test_uri(&env, 1));
        client.burn(&alice, &id);

        let res = client.try_transfer(&alice, &bob, &id);
        assert_eq!(res, Err(Ok(NftError::TokenNotFound)));
    }

    /// Burning an already-burned token returns `TokenNotFound`.
    #[test]
    fn test_burn_burned_token_returns_token_not_found() {
        use crate::NftError;

        let env = Env::default();
        env.mock_all_auths();
        let (minter, client) = setup(&env);
        let alice = Address::generate(&env);

        let id = client.mint(&minter, &alice, &test_uri(&env, 1));
        client.burn(&alice, &id);

        let res = client.try_burn(&alice, &id);
        assert_eq!(res, Err(Ok(NftError::TokenNotFound)));
    }

    // ── issue #1405: total supply after burns ─────────────────────────────────

    /// `total_supply` counts every minted token, including burned ones.
    #[test]
    fn test_total_supply_after_burns() {
        let env = Env::default();
        env.mock_all_auths();
        let (minter, client) = setup(&env);
        let alice = Address::generate(&env);
        let bob = Address::generate(&env);

        // mint tokens interleaved across two owners
        let id1 = client.mint(&minter, &alice, &test_uri(&env, 1));
        let id2 = client.mint(&minter, &bob, &test_uri(&env, 2));
        let id3 = client.mint(&minter, &alice, &test_uri(&env, 3));
        let id4 = client.mint(&minter, &bob, &test_uri(&env, 4));

        assert_eq!(client.total_supply(), 4);

        // burn 3 of the 4 tokens: total supply must NOT decrease
        client.burn(&alice, &id1);
        client.burn(&bob, &id2);
        client.burn(&alice, &id3);

        assert_eq!(client.total_supply(), 4);
        assert_eq!(client.get_owner(&id1), None);
        assert_eq!(client.get_owner(&id2), None);
        assert_eq!(client.get_owner(&id3), None);
        assert_eq!(client.get_owner(&id4), Some(bob.clone()));

        // burning the last token still doesn't touch the supply counter
        client.burn(&bob, &id4);

        assert_eq!(client.total_supply(), 4);
        assert_eq!(client.balance_of(&alice), 0);
        assert_eq!(client.balance_of(&bob), 0);
        assert_owner_index_consistent(&client, &alice);
        assert_owner_index_consistent(&client, &bob);
    }

    // ── issue #1405: many-token interleaved index consistency ────────────────

    /// Mint 12 tokens across 3 owners, transfer and burn them in an interleaved
    /// pattern, then assert the owner index is consistent at every stage.
    #[test]
    fn test_many_tokens_interleaved_burn_consistency() {
        let env = Env::default();
        env.mock_all_auths();
        let (minter, client) = setup(&env);
        let alice = Address::generate(&env);
        let bob = Address::generate(&env);
        let carol = Address::generate(&env);

        // mint 12 tokens: 5 alice, 4 bob, 3 carol
        let mut alice_ids: alloc::vec::Vec<u64> = alloc::vec::Vec::new();
        let mut bob_ids: alloc::vec::Vec<u64> = alloc::vec::Vec::new();
        let mut carol_ids: alloc::vec::Vec<u64> = alloc::vec::Vec::new();

        for i in 1..=5 {
            alice_ids.push(client.mint(&minter, &alice, &test_uri(&env, i)));
        }
        for i in 1..=4 {
            bob_ids.push(client.mint(&minter, &bob, &test_uri(&env, i)));
        }
        for i in 1..=3 {
            carol_ids.push(client.mint(&minter, &carol, &test_uri(&env, i)));
        }

        assert_eq!(client.balance_of(&alice), 5);
        assert_eq!(client.balance_of(&bob), 4);
        assert_eq!(client.balance_of(&carol), 3);
        assert_owner_index_consistent(&client, &alice);
        assert_owner_index_consistent(&client, &bob);
        assert_owner_index_consistent(&client, &carol);

        // interleave: burn alice[2], transfer bob[1] -> alice, burn carol[1],
        // transfer alice[4] -> bob, burn bob[3]
        client.burn(&alice, &alice_ids[2]);            // alice: 5 -> 4
        client.transfer(&bob, &alice, &bob_ids[1]);    // bob: 4 -> 3, alice: 4 -> 5
        client.burn(&carol, &carol_ids[1]);            // carol: 3 -> 2
        client.transfer(&alice, &bob, &alice_ids[4]);  // alice: 5 -> 4, bob: 3 -> 4
        client.burn(&bob, &bob_ids[3]);                // bob: 4 -> 3

        assert_eq!(client.balance_of(&alice), 4);
        assert_eq!(client.balance_of(&bob), 3);
        assert_eq!(client.balance_of(&carol), 2);
        assert_owner_index_consistent(&client, &alice);
        assert_owner_index_consistent(&client, &bob);
        assert_owner_index_consistent(&client, &carol);

        // burned / moved tokens must not appear in the wrong index
        assert_eq!(client.get_owner(&alice_ids[2]), None); // burned
        assert_nft_present(&client, &alice, bob_ids[1]);
        assert_nft_absent(&client, &bob, bob_ids[1]);
        assert_nft_present(&client, &bob, alice_ids[4]);
        assert_nft_absent(&client, &alice, alice_ids[4]);

        // total supply is unchanged by all this churn
        assert_eq!(client.total_supply(), 12);
    }

    // ── issue #1405: property-style randomized sequences ─────────────────────

    /// A deterministic pseudo-random sequence of mint / transfer / burn
    /// operations.  After every operation the owner index must remain
    /// internally consistent, and every live token's `get_owner` must match the
    /// reference state maintained on the test side.
    ///
    /// Uses a small LCG so the sequence is reproducible.
    #[test]
    fn test_property_random_sequences_preserve_index_invariants() {
        let env = Env::default();
        env.mock_all_auths();
        let (minter, client) = setup(&env);

        // participants
        let owners = [
            Address::generate(&env),
            Address::generate(&env),
            Address::generate(&env),
            Address::generate(&env),
        ];

        // reference state: token id -> owner index (None = burned)
        // use a Vec since ids are dense and sequential
        let mut token_owner: alloc::vec::Vec<Option<usize>> = alloc::vec::Vec::new();

        // LCG with fixed seed for reproducibility (NUMERICAL_RECIPES constants)
        let mut state: u64 = 0x2545F4914F6CDD1D;

        fn next_rand(state: &mut u64) -> u64 {
            *state = state
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            (*state >> 33) as u64
        }

        let mut mint_count = 0u64;

        for _ in 0..300 {
            let op = next_rand(&mut state) % 3;
            match op {
                // mint to a random owner
                0 => {
                    let owner = owners[(next_rand(&mut state) % 4) as usize].clone();
                    mint_count += 1;
                    let _id = client.mint(&minter, &owner, &test_uri(&env, (mint_count % 9) as u32));
                    token_owner.push(Some((owners.iter().position(|o| *o == owner)).unwrap()));
                    assert_eq!(client.total_supply(), mint_count);
                }
                // transfer a random live token to a random owner
                1 => {
                    if token_owner.is_empty() {
                        continue;
                    }
                    let id = (next_rand(&mut state) % token_owner.len() as u64) as usize;
                    let Some(from_idx) = token_owner[id] else {
                        continue; // token already burned
                    };
                    let to = owners[(next_rand(&mut state) % 4) as usize].clone();
                    let from = owners[from_idx].clone();
                    let to_idx = (owners.iter().position(|o| *o == to)).unwrap();
                    if from_idx == to_idx {
                        continue; // no-op, skip
                    }

                    let nft_id = (id + 1) as u64;
                    client.transfer(&from, &to, &nft_id);

                    token_owner[id] = Some(to_idx);
                }
                // burn a random live token
                _ => {
                    if token_owner.is_empty() {
                        continue;
                    }
                    let id = (next_rand(&mut state) % token_owner.len() as u64) as usize;
                    let Some(owner_idx) = token_owner[id] else {
                        continue; // already burned
                    };
                    let owner = owners[owner_idx].clone();

                    let nft_id = (id + 1) as u64;
                    client.burn(&owner, &nft_id);
                    token_owner[id] = None;
                }
            }

            // invariant check against the reference model
            for (i, owner_idx) in token_owner.iter().enumerate() {
                let nft_id = (i + 1) as u64;
                match owner_idx {
                    Some(idx) => {
                        assert_eq!(
                            client.get_owner(&nft_id),
                            Some(owners[*idx].clone()),
                            "live token {nft_id} ownership mismatch"
                        );
                    }
                    None => {
                        assert_eq!(
                            client.get_owner(&nft_id),
                            None,
                            "burned token {nft_id} should have no owner"
                        );
                    }
                }
            }

            for owner in &owners {
                assert_owner_index_consistent(&client, owner);
            }

            // per-owner reference balance check
            let mut ref_balance = [0u32; 4];
            for owner_idx in token_owner.iter().flatten() {
                ref_balance[*owner_idx] += 1;
            }
            for (k, owner) in owners.iter().enumerate() {
                assert_eq!(
                    client.balance_of(owner),
                    ref_balance[k],
                    "balance_of mismatch for owner {k}"
                );
            }

            assert_eq!(client.total_supply(), mint_count);
        }
    }

    // ── issue #1401: events published on mint / transfer / burn ──────────────

    #[test]
    fn test_mint_event_published() {
        use soroban_sdk::{
            testutils::Events, IntoVal, Symbol as SorobanSymbol, TryFromVal, Vec as SorobanVec,
        };

        let env = Env::default();
        env.mock_all_auths();
        let (minter, client) = setup(&env);
        let player = Address::generate(&env);

        let id = client.mint(&minter, &player, &test_uri(&env, 1));

        let events = env.events().all();
        assert_eq!(events.len(), 1);

        let event = events.get(0).unwrap();
        assert_eq!(
            event.1,
            SorobanVec::from_array(
                &env,
                [
                    SorobanSymbol::new(&env, "mint").into_val(&env),
                    player.into_val(&env),
                ]
            )
        );
        let data: u64 = u64::try_from_val(&env, &event.2).unwrap();
        assert_eq!(data, id);
    }

    #[test]
    fn test_transfer_event_published() {
        use soroban_sdk::{
            testutils::Events, IntoVal, Symbol as SorobanSymbol, TryFromVal, Vec as SorobanVec,
        };

        let env = Env::default();
        env.mock_all_auths();
        let (minter, client) = setup(&env);
        let alice = Address::generate(&env);
        let bob = Address::generate(&env);

        let id = client.mint(&minter, &alice, &test_uri(&env, 1));

        // After the mint invocation the event buffer holds the mint event.
        let mint_events = env.events().all();
        assert_eq!(mint_events.len(), 1);
        let mint_event = mint_events.get(0).unwrap();
        assert_eq!(
            mint_event.1,
            SorobanVec::from_array(
                &env,
                [
                    SorobanSymbol::new(&env, "mint").into_val(&env),
                    alice.into_val(&env),
                ]
            )
        );
        let mint_data: u64 = u64::try_from_val(&env, &mint_event.2).unwrap();
        assert_eq!(mint_data, id);

        client.transfer(&alice, &bob, &id);

        // Each top-level invocation resets the recorded events, so the buffer
        // now holds only the transfer event.
        let events = env.events().all();
        assert_eq!(events.len(), 1); // transfer

        let transfer_event = events.get(0).unwrap();
        assert_eq!(
            transfer_event.1,
            SorobanVec::from_array(
                &env,
                [
                    SorobanSymbol::new(&env, "transfer").into_val(&env),
                    alice.into_val(&env),
                    bob.into_val(&env),
                ]
            )
        );
        let transfer_data: u64 = u64::try_from_val(&env, &transfer_event.2).unwrap();
        assert_eq!(transfer_data, id);
    }

    #[test]
    fn test_burn_event_published() {
        use soroban_sdk::{
            testutils::Events, IntoVal, Symbol as SorobanSymbol, TryFromVal, Vec as SorobanVec,
        };

        let env = Env::default();
        env.mock_all_auths();
        let (minter, client) = setup(&env);
        let player = Address::generate(&env);

        let id = client.mint(&minter, &player, &test_uri(&env, 1));

        client.burn(&player, &id);

        // Each top-level invocation resets the recorded events, so the buffer
        // holds only the burn event.
        let events = env.events().all();
        assert_eq!(events.len(), 1);

        let burn_event = events.get(0).unwrap();
        assert_eq!(
            burn_event.1,
            SorobanVec::from_array(
                &env,
                [
                    SorobanSymbol::new(&env, "burn").into_val(&env),
                    player.into_val(&env),
                ]
            )
        );
        let burn_data: u64 = u64::try_from_val(&env, &burn_event.2).unwrap();
        assert_eq!(burn_data, id);
    }

}
