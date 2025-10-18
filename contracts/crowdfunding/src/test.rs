#![cfg(test)]

use super::*;
use soroban_sdk::{
    testutils::{Address as _, Ledger},
    token, Address, Env,
};

// Helper function to create a mock token contract for testing
fn create_token_contract<'a>(env: &Env, admin: &Address) -> token::StellarAssetClient<'a> {
    let token_address = env.register_stellar_asset_contract_v2(admin.clone());
    token::StellarAssetClient::new(env, &token_address.address())
}

// Test 1: Initialize campaign successfully
#[test]
fn test_initialize_campaign() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(CrowdfundingContract, ());
    let client = CrowdfundingContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let goal = 900_000_000i128; // 90 XLM goal
    let deadline = env.ledger().timestamp() + 86400; // 24 hours from now
    
    // Create mock token
    let token_admin = Address::generate(&env);
    let token = create_token_contract(&env, &token_admin);
    let token_address = token.address.clone();

    // Initialize campaign
    client.initialize(&owner, &goal, &deadline, &token_address);

    // Verify campaign initialized correctly
    assert_eq!(client.get_total_raised(), 0);
    assert_eq!(client.get_goal(), goal);
    assert_eq!(client.get_deadline(), deadline);
    assert_eq!(client.get_is_already_init(), true);
}

// Test 2: Get donation for address that never donated
#[test]
fn test_get_donation_no_donation() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(CrowdfundingContract, ());
    let client = CrowdfundingContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let non_donor = Address::generate(&env);
    let goal = 900_000_000i128;
    let deadline = env.ledger().timestamp() + 86400;
    
    let token_admin = Address::generate(&env);
    let token = create_token_contract(&env, &token_admin);

    // Initialize campaign
    client.initialize(&owner, &goal, &deadline, &token.address);

    // Check donation amount for address that never donated
    assert_eq!(client.get_donation(&non_donor), 0);
}

// Test 3: Cannot donate zero amount
#[test]
#[should_panic(expected = "Donation amount must be positive")]
fn test_donate_zero_amount() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(CrowdfundingContract, ());
    let client = CrowdfundingContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let donor = Address::generate(&env);
    let goal = 900_000_000i128;
    let deadline = env.ledger().timestamp() + 86400;
    
    let token_admin = Address::generate(&env);
    let token = create_token_contract(&env, &token_admin);

    client.initialize(&owner, &goal, &deadline, &token.address);

    // Try to donate 0 - should panic
    client.donate(&donor, &0);
}

// Test 4: Cannot donate negative amount
#[test]
#[should_panic(expected = "Donation amount must be positive")]
fn test_donate_negative_amount() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(CrowdfundingContract, ());
    let client = CrowdfundingContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let donor = Address::generate(&env);
    let goal = 900_000_000i128;
    let deadline = env.ledger().timestamp() + 86400;
    
    let token_admin = Address::generate(&env);
    let token = create_token_contract(&env, &token_admin);

    client.initialize(&owner, &goal, &deadline, &token.address);

    // Try to donate negative amount - should panic
    client.donate(&donor, &-100_000_000);
}

// Test 5: Campaign deadline validation
#[test]
#[should_panic(expected = "Campaign has ended")]
fn test_donate_after_deadline() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(CrowdfundingContract, ());
    let client = CrowdfundingContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let donor = Address::generate(&env);
    let goal = 900_000_000i128;
    let deadline = env.ledger().timestamp() + 100;
    
    let token_admin = Address::generate(&env);
    let token = create_token_contract(&env, &token_admin);

    client.initialize(&owner, &goal, &deadline, &token.address);

    // Fast forward time past deadline
    env.ledger().with_mut(|li| {
        li.timestamp = deadline + 1;
    });

    // This should panic
    client.donate(&donor, &100_000_000);
}

// Test 6: Check initialization status before initialization
#[test]
fn test_is_already_init_before_initialization() {
    let env = Env::default();
    let contract_id = env.register(CrowdfundingContract, ());
    let client = CrowdfundingContractClient::new(&env, &contract_id);

    // Before initialization, should return false
    assert_eq!(client.get_is_already_init(), false);
}

// Test 7: Check initialization status after initialization
#[test]
fn test_is_already_init_after_initialization() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(CrowdfundingContract, ());
    let client = CrowdfundingContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let goal = 900_000_000i128;
    let deadline = env.ledger().timestamp() + 86400;
    
    let token_admin = Address::generate(&env);
    let token = create_token_contract(&env, &token_admin);

    // Before initialization
    assert_eq!(client.get_is_already_init(), false);

    // Initialize the contract
    client.initialize(&owner, &goal, &deadline, &token.address);

    // After initialization, should return true
    assert_eq!(client.get_is_already_init(), true);
}

// Test 8: Initialization flag persists after other operations
#[test]
fn test_is_already_init_persists() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(CrowdfundingContract, ());
    let client = CrowdfundingContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let donor = Address::generate(&env);
    let goal = 900_000_000i128;
    let deadline = env.ledger().timestamp() + 86400;
    
    let token_admin = Address::generate(&env);
    let token = create_token_contract(&env, &token_admin);

    // Initialize the contract
    client.initialize(&owner, &goal, &deadline, &token.address);

    // Verify it's initialized
    assert_eq!(client.get_is_already_init(), true);

    // Check after querying other functions
    let _ = client.get_total_raised();
    let _ = client.get_donation(&donor);

    // Should still be true
    assert_eq!(client.get_is_already_init(), true);
}

// Test 9: Get goal returns correct value
#[test]
fn test_get_goal() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(CrowdfundingContract, ());
    let client = CrowdfundingContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let goal = 100_000_000i128; // 10 XLM
    let deadline = env.ledger().timestamp() + 86400;
    
    let token_admin = Address::generate(&env);
    let token = create_token_contract(&env, &token_admin);

    client.initialize(&owner, &goal, &deadline, &token.address);

    // Test get_goal returns correct value
    assert_eq!(client.get_goal(), goal);
}

// Test 10: Get deadline returns correct value
#[test]
fn test_get_deadline() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(CrowdfundingContract, ());
    let client = CrowdfundingContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let goal = 100_000_000i128;
    let deadline = env.ledger().timestamp() + 86400; // 24 hours

    let token_admin = Address::generate(&env);
    let token = create_token_contract(&env, &token_admin);

    client.initialize(&owner, &goal, &deadline, &token.address);

    // Test get_deadline returns correct value
    assert_eq!(client.get_deadline(), deadline);
}

// Test 11: Is goal reached - full lifecycle
#[test]
fn test_is_goal_reached() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(CrowdfundingContract, ());
    let client = CrowdfundingContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let donor = Address::generate(&env);
    let goal = 50_000_000i128; // 5 XLM
    let deadline = env.ledger().timestamp() + 86400;

    // Create and setup token
    let token_admin = Address::generate(&env);
    let token = create_token_contract(&env, &token_admin);
    
    // Mint tokens to donor for testing
    token.mint(&donor, &100_000_000); // 10 XLM

    client.initialize(&owner, &goal, &deadline, &token.address);

    // Before donation - should be false
    assert_eq!(client.is_goal_reached(), false);

    // Donate less than goal
    client.donate(&donor, &30_000_000); // 3 XLM
    assert_eq!(client.is_goal_reached(), false);

    // Donate to reach goal
    client.donate(&donor, &20_000_000); // 2 XLM more
    assert_eq!(client.is_goal_reached(), true);

    // Donate more than goal
    client.donate(&donor, &10_000_000); // 1 XLM more
    assert_eq!(client.is_goal_reached(), true); // Still true
}

// Test 12: Is ended - timeline check
#[test]
fn test_is_ended() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(CrowdfundingContract, ());
    let client = CrowdfundingContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let goal = 100_000_000i128;
    let deadline = env.ledger().timestamp() + 1000; // 1000 seconds

    let token_admin = Address::generate(&env);
    let token = create_token_contract(&env, &token_admin);

    client.initialize(&owner, &goal, &deadline, &token.address);

    // Before deadline
    assert_eq!(client.is_ended(), false);

    // Fast forward time to deadline
    env.ledger().with_mut(|li| {
        li.timestamp = deadline;
    });
    assert_eq!(client.is_ended(), false); // Exactly at deadline = not ended

    // Fast forward past deadline
    env.ledger().with_mut(|li| {
        li.timestamp = deadline + 1;
    });
    assert_eq!(client.is_ended(), true); // Now it's ended
}

// Test 13: Get progress percentage
#[test]
fn test_get_progress_percentage() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(CrowdfundingContract, ());
    let client = CrowdfundingContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let donor = Address::generate(&env);
    let goal = 100_000_000i128; // 10 XLM
    let deadline = env.ledger().timestamp() + 86400;

    // Create and setup token
    let token_admin = Address::generate(&env);
    let token = create_token_contract(&env, &token_admin);
    
    // Mint tokens to donor
    token.mint(&donor, &200_000_000); // 20 XLM

    client.initialize(&owner, &goal, &deadline, &token.address);

    // 0% progress
    assert_eq!(client.get_progress_percentage(), 0);

    // Donate 25% of goal
    client.donate(&donor, &25_000_000); // 2.5 XLM
    assert_eq!(client.get_progress_percentage(), 25);

    // Donate to reach 50%
    client.donate(&donor, &25_000_000); // 2.5 XLM more
    assert_eq!(client.get_progress_percentage(), 50);

    // Donate to reach 100%
    client.donate(&donor, &50_000_000); // 5 XLM more
    assert_eq!(client.get_progress_percentage(), 100);

    // Donate more than goal - 120%
    client.donate(&donor, &20_000_000); // 2 XLM more
    assert_eq!(client.get_progress_percentage(), 120);
}

// Test 14: Refund success scenario
#[test]
fn test_refund_success() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(CrowdfundingContract, ());
    let client = CrowdfundingContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let donor = Address::generate(&env);
    let goal = 100_000_000i128; // 10 XLM
    let deadline = env.ledger().timestamp() + 100;

    // Create and setup token
    let token_admin = Address::generate(&env);
    let token = create_token_contract(&env, &token_admin);
    
    // Mint tokens to donor
    token.mint(&donor, &50_000_000); // 5 XLM

    client.initialize(&owner, &goal, &deadline, &token.address);

    // Make donation
    let donation_amount = 30_000_000i128; // 3 XLM
    client.donate(&donor, &donation_amount);

    // Verify donation recorded
    assert_eq!(client.get_donation(&donor), donation_amount);
    assert_eq!(client.get_total_raised(), donation_amount);

    // Fast forward past deadline (goal not reached)
    env.ledger().with_mut(|li| {
        li.timestamp = deadline + 1;
    });

    // Refund
    let refunded = client.refund(&donor);

    // Verify refund
    assert_eq!(refunded, donation_amount);
    assert_eq!(client.get_donation(&donor), 0);
    assert_eq!(client.get_total_raised(), 0);
}

// Test 15: Cannot refund before deadline
#[test]
#[should_panic(expected = "Campaign belum berakhir")]
fn test_refund_before_deadline() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(CrowdfundingContract, ());
    let client = CrowdfundingContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let donor = Address::generate(&env);
    let goal = 100_000_000i128;
    let deadline = env.ledger().timestamp() + 1000;

    // Create and setup token
    let token_admin = Address::generate(&env);
    let token = create_token_contract(&env, &token_admin);
    
    // Mint tokens to donor
    token.mint(&donor, &50_000_000);

    client.initialize(&owner, &goal, &deadline, &token.address);
    client.donate(&donor, &30_000_000);

    // Try refund before deadline - should panic
    client.refund(&donor);
}

// Test 16: Cannot refund when goal reached
#[test]
#[should_panic(expected = "Goal sudah tercapai, tidak bisa refund")]
fn test_refund_when_goal_reached() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(CrowdfundingContract, ());
    let client = CrowdfundingContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let donor = Address::generate(&env);
    let goal = 50_000_000i128; // 5 XLM
    let deadline = env.ledger().timestamp() + 100;

    // Create and setup token
    let token_admin = Address::generate(&env);
    let token = create_token_contract(&env, &token_admin);
    
    // Mint tokens to donor
    token.mint(&donor, &goal);

    client.initialize(&owner, &goal, &deadline, &token.address);

    // Donate exactly goal amount
    client.donate(&donor, &goal);

    // Fast forward past deadline
    env.ledger().with_mut(|li| {
        li.timestamp = deadline + 1;
    });

    // Try refund when goal reached - should panic
    client.refund(&donor);
}

// Test 17: Cannot refund if no donations made
#[test]
#[should_panic(expected = "No donations found for this address")]
fn test_refund_no_donations() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(CrowdfundingContract, ());
    let client = CrowdfundingContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let non_donor = Address::generate(&env);
    let goal = 100_000_000i128;
    let deadline = env.ledger().timestamp() + 100;

    let token_admin = Address::generate(&env);
    let token = create_token_contract(&env, &token_admin);

    client.initialize(&owner, &goal, &deadline, &token.address);

    // Fast forward past deadline
    env.ledger().with_mut(|li| {
        li.timestamp = deadline + 1;
    });

    // Try to refund without making donation - should panic
    client.refund(&non_donor);
}