#![cfg(test)]

mod test {
    use soroban_sdk::{Env, Address, symbol_short};
    use soroban_sdk::testutils::Address as _;
    use splitex_escrow::{splitexEscrow, splitexEscrowClient};

    // Register the contract and return a ready-to-use client.
    fn setup(env: &Env) -> splitexEscrowClient {
        env.mock_all_auths();
        let contract_id = env.register(splitexEscrow, ());
        splitexEscrowClient::new(env, &contract_id)
    }

    fn create_test_addresses(env: &Env) -> (Address, Address, Address, Address) {
        (
            Address::generate(env),
            Address::generate(env),
            Address::generate(env),
            Address::generate(env),
        )
    }

    fn create_token_address(env: &Env) -> Address {
        Address::generate(env)
    }

    #[test]
    fn test_contract_initialization() {
        let env = Env::default();
        let client = setup(&env);
        client.init();
        // If initialization fails, the call above panics.
        assert!(true, "Contract initialized successfully");
    }

    #[test]
    fn test_create_split_basic() {
        let env = Env::default();
        let client = setup(&env);
        client.init();

        let (creator, participant1, participant2, _) = create_test_addresses(&env);
        let token = create_token_address(&env);

        let participants = soroban_sdk::vec![&env, participant1.clone(), participant2.clone()];
        let shares = soroban_sdk::vec![&env, 50u32, 50u32];

        let split_id = client.create_split(
            &creator,
            &symbol_short!("dinner"),
            &1000i128,
            &token,
            &participants,
            &shares,
        );

        assert_eq!(split_id, 1, "First split should have ID 1");
    }

    #[test]
    fn test_create_split_three_way() {
        let env = Env::default();
        let client = setup(&env);
        client.init();

        let (creator, p1, p2, p3) = create_test_addresses(&env);
        let token = create_token_address(&env);

        let participants = soroban_sdk::vec![&env, p1.clone(), p2.clone(), p3.clone()];
        let shares = soroban_sdk::vec![&env, 33u32, 33u32, 34u32]; // Sums to 100

        let split_id = client.create_split(
            &creator,
            &symbol_short!("party"),
            &3000i128,
            &token,
            &participants,
            &shares,
        );

        assert_eq!(split_id, 1, "Split created successfully");

        let split = client.get_split(&split_id);
        assert_eq!(split.total_amount, 3000i128, "Correct total amount");
        assert_eq!(split.settled, false, "Split not settled initially");
    }

    #[test]
    fn test_deposit_share() {
        let env = Env::default();
        let client = setup(&env);
        client.init();

        let (creator, participant1, participant2, _) = create_test_addresses(&env);
        let token = create_token_address(&env);

        let participants = soroban_sdk::vec![&env, participant1.clone(), participant2.clone()];
        let shares = soroban_sdk::vec![&env, 50u32, 50u32];

        let split_id = client.create_split(
            &creator,
            &symbol_short!("lunch"),
            &2000i128,
            &token,
            &participants,
            &shares,
        );

        // Participant 1 deposits their share (50% of 2000 = 1000)
        client.deposit_share(&participant1, &split_id, &1000i128);

        let share = client.get_participant_share(&split_id, &participant1);
        assert_eq!(share.amount_paid, 1000i128, "Amount paid is correct");
        assert_eq!(share.is_settled, true, "Participant marked as settled");
    }

    #[test]
    fn test_get_participant_share() {
        let env = Env::default();
        let client = setup(&env);
        client.init();

        let (creator, participant1, participant2, _) = create_test_addresses(&env);
        let token = create_token_address(&env);

        let participants = soroban_sdk::vec![&env, participant1.clone(), participant2.clone()];
        let shares = soroban_sdk::vec![&env, 40u32, 60u32];

        let split_id = client.create_split(
            &creator,
            &symbol_short!("bills"),
            &1000i128,
            &token,
            &participants,
            &shares,
        );

        let share = client.get_participant_share(&split_id, &participant1);
        assert_eq!(share.share_percentage, 40u32, "Correct share percentage");
        assert_eq!(share.amount_to_pay, 400i128, "Correct amount to pay (40% of 1000)");
        assert_eq!(share.amount_paid, 0i128, "No payment yet");
    }

    #[test]
    fn test_settle_split() {
        let env = Env::default();
        let client = setup(&env);
        client.init();

        let (creator, participant1, participant2, _) = create_test_addresses(&env);
        let token = create_token_address(&env);

        let participants = soroban_sdk::vec![&env, participant1.clone(), participant2.clone()];
        let shares = soroban_sdk::vec![&env, 50u32, 50u32];

        let split_id = client.create_split(
            &creator,
            &symbol_short!("dinner"),
            &2000i128,
            &token,
            &participants,
            &shares,
        );

        // Both participants deposit
        client.deposit_share(&participant1, &split_id, &1000i128);
        client.deposit_share(&participant2, &split_id, &1000i128);

        // Settle the split
        client.settle_split(&split_id);

        assert!(client.is_split_settled(&split_id), "Split is marked as settled");

        let settlement = client.get_settlement(&split_id);
        assert_eq!(settlement.split_id, split_id, "Settlement split ID matches");
        assert_eq!(settlement.total_paid, 2000i128, "Total paid is correct");
        assert_eq!(settlement.participant_count, 2u32, "Correct participant count");
    }

    #[test]
    fn test_get_user_splits() {
        let env = Env::default();
        let client = setup(&env);
        client.init();

        let (creator, participant1, participant2, _) = create_test_addresses(&env);
        let token = create_token_address(&env);

        let participants = soroban_sdk::vec![&env, participant1.clone(), participant2.clone()];
        let shares = soroban_sdk::vec![&env, 50u32, 50u32];

        let _split_id_1 = client.create_split(
            &creator,
            &symbol_short!("lunch"),
            &1000i128,
            &token,
            &participants,
            &shares,
        );

        let _split_id_2 = client.create_split(
            &creator,
            &symbol_short!("dinner"),
            &2000i128,
            &token,
            &participants,
            &shares,
        );

        let user_splits = client.get_user_splits(&creator);
        assert_eq!(user_splits.len(), 2, "User has 2 splits");
    }

    #[test]
    fn test_multiple_splits_sequential_ids() {
        let env = Env::default();
        let client = setup(&env);
        client.init();

        let (creator, participant1, participant2, _) = create_test_addresses(&env);
        let token = create_token_address(&env);

        let participants = soroban_sdk::vec![&env, participant1.clone(), participant2.clone()];
        let shares = soroban_sdk::vec![&env, 50u32, 50u32];

        let split_id_1 = client.create_split(
            &creator,
            &symbol_short!("split1"),
            &1000i128,
            &token,
            &participants,
            &shares,
        );

        let split_id_2 = client.create_split(
            &creator,
            &symbol_short!("split2"),
            &2000i128,
            &token,
            &participants,
            &shares,
        );

        assert_eq!(split_id_1, 1, "First split ID is 1");
        assert_eq!(split_id_2, 2, "Second split ID is 2");
    }

    #[test]
    fn test_partial_deposit() {
        let env = Env::default();
        let client = setup(&env);
        client.init();

        let (creator, participant1, participant2, _) = create_test_addresses(&env);
        let token = create_token_address(&env);

        let participants = soroban_sdk::vec![&env, participant1.clone(), participant2.clone()];
        let shares = soroban_sdk::vec![&env, 50u32, 50u32];

        let split_id = client.create_split(
            &creator,
            &symbol_short!("expenses"),
            &2000i128,
            &token,
            &participants,
            &shares,
        );

        // Participant 1 deposits half of their share
        client.deposit_share(&participant1, &split_id, &500i128);

        let share = client.get_participant_share(&split_id, &participant1);
        assert_eq!(share.amount_paid, 500i128, "Partial payment recorded");
        assert_eq!(share.is_settled, false, "Not fully settled with partial payment");
    }

    #[test]
    #[should_panic]
    fn test_invalid_shares_sum() {
        let env = Env::default();
        let client = setup(&env);
        client.init();

        let (creator, participant1, participant2, _) = create_test_addresses(&env);
        let token = create_token_address(&env);

        let participants = soroban_sdk::vec![&env, participant1.clone(), participant2.clone()];
        let shares = soroban_sdk::vec![&env, 50u32, 40u32]; // Only sums to 90

        client.create_split(
            &creator,
            &symbol_short!("invalid"),
            &1000i128,
            &token,
            &participants,
            &shares,
        );
    }

    #[test]
    #[should_panic]
    fn test_empty_participants() {
        let env = Env::default();
        let client = setup(&env);
        client.init();

        let (creator, _, _, _) = create_test_addresses(&env);
        let token = create_token_address(&env);

        let participants = soroban_sdk::vec![&env];
        let shares = soroban_sdk::vec![&env];

        client.create_split(
            &creator,
            &symbol_short!("empty"),
            &1000i128,
            &token,
            &participants,
            &shares,
        );
    }

    #[test]
    #[should_panic]
    fn test_non_positive_amount() {
        let env = Env::default();
        let client = setup(&env);
        client.init();

        let (creator, participant1, participant2, _) = create_test_addresses(&env);
        let token = create_token_address(&env);

        let participants = soroban_sdk::vec![&env, participant1.clone(), participant2.clone()];
        let shares = soroban_sdk::vec![&env, 50u32, 50u32];

        client.create_split(
            &creator,
            &symbol_short!("zero"),
            &0i128, // Invalid
            &token,
            &participants,
            &shares,
        );
    }
}
