use soroban_sdk::{symbol_short, testutils::Address as _, vec, Address, Env};

use crate::{Escrow, EscrowClient};

#[test]
fn test_hello() {
    let env = Env::default();
    let contract_id = env.register(Escrow, ());
    let client = EscrowClient::new(&env, &contract_id);

    let result = client.hello(&symbol_short!("World"));
    assert_eq!(result, symbol_short!("World"));
}

#[test]
fn test_initialize() {
    let env = Env::default();
    let contract_id = env.register(Escrow, ());
    let client = EscrowClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let arbitrator = Address::generate(&env);

    // Set the admin as the invoking address for authentication
    env.mock_auths(&[
        (&contract_id, &admin, &symbol_short!("initialize")),
    ]);

    client.initialize(&admin, &arbitrator);
}

#[test]
fn test_create_contract() {
    let env = Env::default();
    let contract_id = env.register(Escrow, ());
    let client = EscrowClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let arbitrator = Address::generate(&env);
    let client_addr = Address::generate(&env);
    let freelancer_addr = Address::generate(&env);
    let milestones = vec![&env, 200_0000000_i128, 400_0000000_i128, 600_0000000_i128];

    // Set up authentication
    env.mock_auths(&[
        (&contract_id, &admin, &symbol_short!("initialize")),
        (&contract_id, &client_addr, &symbol_short!("create_contract")),
    ]);

    // Initialize first
    client.initialize(&admin, &arbitrator);
    
    // Then create contract
    let id = client.create_contract(&client_addr, &freelancer_addr, &milestones);
    assert_eq!(id, 1);
}

#[test]
fn test_deposit_funds() {
    let env = Env::default();
    let contract_id = env.register(Escrow, ());
    let client = EscrowClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let arbitrator = Address::generate(&env);
    let client_addr = Address::generate(&env);
    let freelancer_addr = Address::generate(&env);
    let milestones = vec![&env, 1_000_0000000_i128];

    // Set up authentication
    env.mock_auths(&[
        (&contract_id, &admin, &symbol_short!("initialize")),
        (&contract_id, &client_addr, &symbol_short!("create_contract")),
        (&contract_id, &client_addr, &symbol_short!("deposit_funds")),
    ]);

    // Initialize and create contract
    client.initialize(&admin, &arbitrator);
    let escrow_id = client.create_contract(&client_addr, &freelancer_addr, &milestones);
    
    // Deposit funds
    let result = client.deposit_funds(&escrow_id, &1_000_0000000);
    assert!(result);
}

#[test]
fn test_release_milestone() {
    let env = Env::default();
    let contract_id = env.register(Escrow, ());
    let client = EscrowClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let arbitrator = Address::generate(&env);
    let client_addr = Address::generate(&env);
    let freelancer_addr = Address::generate(&env);
    let milestones = vec![&env, 1_000_0000000_i128];

    // Set up authentication
    env.mock_auths(&[
        (&contract_id, &admin, &symbol_short!("initialize")),
        (&contract_id, &client_addr, &symbol_short!("create_contract")),
        (&contract_id, &client_addr, &symbol_short!("deposit_funds")),
        (&contract_id, &client_addr, &symbol_short!("release_milestone")),
    ]);

    // Initialize, create, and fund contract
    client.initialize(&admin, &arbitrator);
    let escrow_id = client.create_contract(&client_addr, &freelancer_addr, &milestones);
    client.deposit_funds(&escrow_id, &1_000_0000000);
    
    // Release milestone
    let result = client.release_milestone(&escrow_id, &0);
    assert!(result);
}

// Dispute resolution tests

#[test]
fn test_create_dispute() {
    let env = Env::default();
    let contract_id = env.register(Escrow, ());
    let client = EscrowClient::new(&env, &contract_id);

    // Initialize contract
    let admin = Address::generate(&env);
    let arbitrator = Address::generate(&env);
    let client_addr = Address::generate(&env);
    let freelancer_addr = Address::generate(&env);
    let milestones = vec![&env, 1000_0000000_i128];

    // Set up authentication
    env.mock_auths(&[
        (&contract_id, &admin, &symbol_short!("initialize")),
        (&contract_id, &client_addr, &symbol_short!("create_contract")),
        (&contract_id, &client_addr, &symbol_short!("deposit_funds")),
        (&contract_id, &client_addr, &symbol_short!("create_dispute")),
    ]);

    // Initialize and create contract
    client.initialize(&admin, &arbitrator);
    let escrow_id = client.create_contract(&client_addr, &freelancer_addr, &milestones);
    client.deposit_funds(&escrow_id, &1000_0000000);

    // Create dispute
    let reason = symbol_short!("quality");
    let evidence = vec![&env, symbol_short!("evidence1")];

    let dispute_id = client.create_dispute(&escrow_id, &reason, &evidence);
    assert_eq!(dispute_id, 1);
}

#[test]
fn test_resolve_dispute_full_refund() {
    let env = Env::default();
    let contract_id = env.register(Escrow, ());
    let client = EscrowClient::new(&env, &contract_id);

    // Initialize contract
    let admin = Address::generate(&env);
    let arbitrator = Address::generate(&env);
    let client_addr = Address::generate(&env);
    let freelancer_addr = Address::generate(&env);
    let milestones = vec![&env, 1000_0000000_i128];

    // Set up authentication
    env.mock_auths(&[
        (&contract_id, &admin, &symbol_short!("initialize")),
        (&contract_id, &client_addr, &symbol_short!("create_contract")),
        (&contract_id, &client_addr, &symbol_short!("deposit_funds")),
        (&contract_id, &client_addr, &symbol_short!("create_dispute")),
        (&contract_id, &arbitrator, &symbol_short!("resolve_dispute")),
    ]);

    // Initialize and create contract
    client.initialize(&admin, &arbitrator);
    let escrow_id = client.create_contract(&client_addr, &freelancer_addr, &milestones);
    client.deposit_funds(&escrow_id, &1000_0000000);

    // Create dispute
    let reason = symbol_short!("quality");
    let evidence = vec![&env, symbol_short!("evidence1")];
    let dispute_id = client.create_dispute(&escrow_id, &reason, &evidence);

    // Resolve dispute with full refund
    let result = client.resolve_dispute(&dispute_id, &soroban_sdk::symbol_short!("FullRefund"), &0, &0);
    assert!(result);
}

#[test]
fn test_resolve_dispute_partial_refund() {
    let env = Env::default();
    let contract_id = env.register(Escrow, ());
    let client = EscrowClient::new(&env, &contract_id);

    // Initialize contract
    let admin = Address::generate(&env);
    let arbitrator = Address::generate(&env);
    let client_addr = Address::generate(&env);
    let freelancer_addr = Address::generate(&env);
    let milestones = vec![&env, 1000_0000000_i128];

    // Set up authentication
    env.mock_auths(&[
        (&contract_id, &admin, &symbol_short!("initialize")),
        (&contract_id, &client_addr, &symbol_short!("create_contract")),
        (&contract_id, &client_addr, &symbol_short!("deposit_funds")),
        (&contract_id, &client_addr, &symbol_short!("create_dispute")),
        (&contract_id, &arbitrator, &symbol_short!("resolve_dispute")),
    ]);

    // Initialize and create contract
    client.initialize(&admin, &arbitrator);
    let escrow_id = client.create_contract(&client_addr, &freelancer_addr, &milestones);
    client.deposit_funds(&escrow_id, &1000_0000000);

    // Create dispute
    let reason = symbol_short!("delay");
    let evidence = vec![&env, symbol_short!("evidence1")];
    let dispute_id = client.create_dispute(&escrow_id, &reason, &evidence);

    // Resolve dispute with partial refund
    let result = client.resolve_dispute(&dispute_id, &soroban_sdk::symbol_short!("PartialRefund"), &0, &0);
    assert!(result);
}

#[test]
fn test_resolve_dispute_full_payout() {
    let env = Env::default();
    let contract_id = env.register(Escrow, ());
    let client = EscrowClient::new(&env, &contract_id);

    // Initialize contract
    let admin = Address::generate(&env);
    let arbitrator = Address::generate(&env);
    let client_addr = Address::generate(&env);
    let freelancer_addr = Address::generate(&env);
    let milestones = vec![&env, 1000_0000000_i128];

    // Set up authentication
    env.mock_auths(&[
        (&contract_id, &admin, &symbol_short!("initialize")),
        (&contract_id, &client_addr, &symbol_short!("create_contract")),
        (&contract_id, &client_addr, &symbol_short!("deposit_funds")),
        (&contract_id, &client_addr, &symbol_short!("create_dispute")),
        (&contract_id, &arbitrator, &symbol_short!("resolve_dispute")),
    ]);

    // Initialize and create contract
    client.initialize(&admin, &arbitrator);
    let escrow_id = client.create_contract(&client_addr, &freelancer_addr, &milestones);
    client.deposit_funds(&escrow_id, &1000_0000000);

    // Create dispute
    let reason = symbol_short!("completed");
    let evidence = vec![&env, symbol_short!("evidence1")];
    let dispute_id = client.create_dispute(&escrow_id, &reason, &evidence);

    // Resolve dispute with full payout to freelancer
    let result = client.resolve_dispute(&dispute_id, &soroban_sdk::symbol_short!("FullPayout"), &0, &0);
    assert!(result);
}

#[test]
fn test_resolve_dispute_custom_split() {
    let env = Env::default();
    let contract_id = env.register(Escrow, ());
    let client = EscrowClient::new(&env, &contract_id);

    // Initialize contract
    let admin = Address::generate(&env);
    let arbitrator = Address::generate(&env);
    let client_addr = Address::generate(&env);
    let freelancer_addr = Address::generate(&env);
    let milestones = vec![&env, 1000_0000000_i128];

    // Set up authentication
    env.mock_auths(&[
        (&contract_id, &admin, &symbol_short!("initialize")),
        (&contract_id, &client_addr, &symbol_short!("create_contract")),
        (&contract_id, &client_addr, &symbol_short!("deposit_funds")),
        (&contract_id, &client_addr, &symbol_short!("create_dispute")),
        (&contract_id, &arbitrator, &symbol_short!("resolve_dispute")),
    ]);

    // Initialize and create contract
    client.initialize(&admin, &arbitrator);
    let escrow_id = client.create_contract(&client_addr, &freelancer_addr, &milestones);
    client.deposit_funds(&escrow_id, &1000_0000000);

    // Create dispute
    let reason = symbol_short!("partial");
    let evidence = vec![&env, symbol_short!("evidence1")];
    let dispute_id = client.create_dispute(&escrow_id, &reason, &evidence);

    // Resolve dispute with custom split (60% to client, 40% to freelancer)
    let result = client.resolve_dispute(
        &dispute_id,
        &soroban_sdk::symbol_short!("Split"),
        &600_0000000,
        &400_0000000,
    );
    assert!(result);
}

#[test]
#[should_panic(expected = "split amounts must equal total contract amount")]
fn test_resolve_dispute_invalid_split() {
    let env = Env::default();
    let contract_id = env.register(Escrow, ());
    let client = EscrowClient::new(&env, &contract_id);

    // Initialize contract
    let admin = Address::generate(&env);
    let arbitrator = Address::generate(&env);
    let client_addr = Address::generate(&env);
    let freelancer_addr = Address::generate(&env);
    let milestones = vec![&env, 1000_0000000_i128];

    // Set up authentication
    env.mock_auths(&[
        (&contract_id, &admin, &symbol_short!("initialize")),
        (&contract_id, &client_addr, &symbol_short!("create_contract")),
        (&contract_id, &client_addr, &symbol_short!("deposit_funds")),
        (&contract_id, &client_addr, &symbol_short!("create_dispute")),
        (&contract_id, &arbitrator, &symbol_short!("resolve_dispute")),
    ]);

    // Initialize and create contract
    client.initialize(&admin, &arbitrator);
    let escrow_id = client.create_contract(&client_addr, &freelancer_addr, &milestones);
    client.deposit_funds(&escrow_id, &1000_0000000);

    // Create dispute
    let reason = symbol_short!("partial");
    let evidence = vec![&env, symbol_short!("evidence1")];
    let dispute_id = client.create_dispute(&escrow_id, &reason, &evidence);

    // Try to resolve with invalid split (doesn't equal total)
    client.resolve_dispute(
        &dispute_id,
        &soroban_sdk::symbol_short!("Split"),
        &600_0000000,
        &300_0000000,
    );
}

#[test]
#[should_panic(expected = "only client or freelancer can create dispute")]
fn test_create_dispute_unauthorized() {
    let env = Env::default();
    let contract_id = env.register(Escrow, ());
    let client = EscrowClient::new(&env, &contract_id);

    // Initialize contract
    let admin = Address::generate(&env);
    let arbitrator = Address::generate(&env);
    let client_addr = Address::generate(&env);
    let freelancer_addr = Address::generate(&env);
    let milestones = vec![&env, 1000_0000000_i128];

    // Set up authentication
    env.mock_auths(&[
        (&contract_id, &admin, &symbol_short!("initialize")),
        (&contract_id, &client_addr, &symbol_short!("create_contract")),
        (&contract_id, &client_addr, &symbol_short!("deposit_funds")),
    ]);

    // Initialize and create contract
    client.initialize(&admin, &arbitrator);
    let escrow_id = client.create_contract(&client_addr, &freelancer_addr, &milestones);
    client.deposit_funds(&escrow_id, &1000_0000000);

    // Try to create dispute from unauthorized address (this will fail due to auth)
    let reason = symbol_short!("quality");
    let evidence = vec![&env, symbol_short!("evidence1")];

    // This should panic due to authorization failure
    client.create_dispute(&escrow_id, &reason, &evidence);
}

#[test]
fn test_update_admin() {
    let env = Env::default();
    let contract_id = env.register(Escrow, ());
    let client = EscrowClient::new(&env, &contract_id);

    // Initialize contract
    let admin = Address::generate(&env);
    let arbitrator = Address::generate(&env);

    // Set up authentication
    env.mock_auths(&[
        (&contract_id, &admin, &symbol_short!("initialize")),
        (&contract_id, &admin, &symbol_short!("update_admin")),
    ]);

    client.initialize(&admin, &arbitrator);

    // Update admin
    let new_admin = Address::generate(&env);
    client.update_admin(&new_admin);
}

#[test]
fn test_update_arbitrator() {
    let env = Env::default();
    let contract_id = env.register(Escrow, ());
    let client = EscrowClient::new(&env, &contract_id);

    // Initialize contract
    let admin = Address::generate(&env);
    let arbitrator = Address::generate(&env);

    // Set up authentication
    env.mock_auths(&[
        (&contract_id, &admin, &symbol_short!("initialize")),
        (&contract_id, &admin, &symbol_short!("update_arbitrator")),
    ]);

    client.initialize(&admin, &arbitrator);

    // Update arbitrator
    let new_arbitrator = Address::generate(&env);
    client.update_arbitrator(&new_arbitrator);
}

#[test]
#[should_panic(expected = "already initialized")]
fn test_double_initialize() {
    let env = Env::default();
    let contract_id = env.register(Escrow, ());
    let client = EscrowClient::new(&env, &contract_id);

    // Initialize contract
    let admin = Address::generate(&env);
    let arbitrator = Address::generate(&env);

    // Set up authentication
    env.mock_auths(&[
        (&contract_id, &admin, &symbol_short!("initialize")),
    ]);

    client.initialize(&admin, &arbitrator);

    // Try to initialize again
    let admin2 = Address::generate(&env);
    let arbitrator2 = Address::generate(&env);
    client.initialize(&admin2, &arbitrator2);
}
