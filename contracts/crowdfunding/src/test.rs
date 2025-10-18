#![cfg(test)]

use super::*;
use soroban_sdk::{testutils::Address as _, testutils::Ledger, Address, Env};

const XLM_CONTRACT_TESTNET: &str = "CDLZFC3SYJYDZT7K67VZ75HPJVIEUVNIXF47ZG2FB2RMQQVU2HHGCYSC";

// Test 1: Initialize campaign successfully
#[test]
fn test_initialize_campaign() {
    // Setup test environment
    let env = Env::default();
    let contract_id = env.register(CrowdfundingContract, ());
    let client = CrowdfundingContractClient::new(&env, &contract_id);

    // Create mock addresses
    let owner = Address::generate(&env);
    let goal = 900_000_000i128; // 90 XLM goal (90 * 10^7 stroops)
    let deadline = env.ledger().timestamp() + 86400; // 24 jam dari sekarang
    let xlm_token_address =
        Address::from_string(&soroban_sdk::String::from_str(&env, XLM_CONTRACT_TESTNET));

    // Mock authorization untuk owner
    env.mock_all_auths();

    // Verify campaign initialized dengan benar
    client.initialize(&owner, &goal, &deadline, &xlm_token_address);

    // Verify campaign initialized dengan benar
    assert_eq!(client.get_total_raised(), 0);
}

// Test 2: Make a donation
#[test]
fn test_get_donation_no_donation() {
    let env = Env::default();
    let contract_id = env.register(CrowdfundingContract, ());
    let client = CrowdfundingContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let non_donor = Address::generate(&env);
    let goal = 900_000_000i128;
    let deadline = env.ledger().timestamp() + 86400;
    let xlm_token_address =
        Address::from_string(&soroban_sdk::String::from_str(&env, XLM_CONTRACT_TESTNET));

    env.mock_all_auths();

    // Initialize campaign
    client.initialize(&owner, &goal, &deadline, &xlm_token_address);

    // Check donation amount for address that never donated
    assert_eq!(client.get_donation(&non_donor), 0);
}

// Test 3: Cannot donate negative or zero amount
#[test]
#[should_panic(expected = "Donation amount must be positive")]
fn test_donate_zero_amount() {
    let env = Env::default();
    let contract_id = env.register(CrowdfundingContract, ());
    let client = CrowdfundingContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let donor = Address::generate(&env);
    let goal = 900_000_000i128;
    let deadline = env.ledger().timestamp() + 86400;
    let xlm_token_address =
        Address::from_string(&soroban_sdk::String::from_str(&env, XLM_CONTRACT_TESTNET));

    env.mock_all_auths();

    client.initialize(&owner, &goal, &deadline, &xlm_token_address);

    // Try to donate 0 - should panic
    client.donate(&donor, &0);
}

// Test 4: Cannot donate negative amount
#[test]
#[should_panic(expected = "Donation amount must be positive")]
fn test_donate_negative_amount() {
    let env = Env::default();
    let contract_id = env.register(CrowdfundingContract, ());
    let client = CrowdfundingContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let donor = Address::generate(&env);
    let goal = 900_000_000i128;
    let deadline = env.ledger().timestamp() + 86400;
    let xlm_token_address =
        Address::from_string(&soroban_sdk::String::from_str(&env, XLM_CONTRACT_TESTNET));

    env.mock_all_auths();

    client.initialize(&owner, &goal, &deadline, &xlm_token_address);

    // Try to donate negative amount - should panic
    client.donate(&donor, &-100_000_000);
}

// Test 5: Campaign deadline validation
#[test]
#[should_panic(expected = "Campaign has ended")]
fn test_donate_after_deadline() {
    let env = Env::default();
    let contract_id = env.register(CrowdfundingContract, ());
    let client = CrowdfundingContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let donor = Address::generate(&env);
    let goal = 900_000_000i128;
    let deadline = env.ledger().timestamp() + 100;
    let xlm_token_address =
        Address::from_string(&soroban_sdk::String::from_str(&env, XLM_CONTRACT_TESTNET));

    env.mock_all_auths();

    client.initialize(&owner, &goal, &deadline, &xlm_token_address);

    // Fast forward time past deadline
    env.ledger().with_mut(|li| {
        li.timestamp = deadline + 1;
    });

    // This should panic - will fail at XLM transfer but deadline check comes first
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
    let contract_id = env.register(CrowdfundingContract, ());
    let client = CrowdfundingContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let goal = 900_000_000i128;
    let deadline = env.ledger().timestamp() + 86400;
    let xlm_token_address =
        Address::from_string(&soroban_sdk::String::from_str(&env, XLM_CONTRACT_TESTNET));

    env.mock_all_auths();

    // Before initialization
    assert_eq!(client.get_is_already_init(), false);

    // Initialize the contract
    client.initialize(&owner, &goal, &deadline, &xlm_token_address);

    // After initialization, should return true
    assert_eq!(client.get_is_already_init(), true);
}

// Test 8: Initialization flag persists after other operations
#[test]
fn test_is_already_init_persists() {
    let env = Env::default();
    let contract_id = env.register(CrowdfundingContract, ());
    let client = CrowdfundingContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let donor = Address::generate(&env);
    let goal = 900_000_000i128;
    let deadline = env.ledger().timestamp() + 86400;
    let xlm_token_address =
        Address::from_string(&soroban_sdk::String::from_str(&env, XLM_CONTRACT_TESTNET));

    env.mock_all_auths();

    // Initialize the contract
    client.initialize(&owner, &goal, &deadline, &xlm_token_address);

    // Verify it's initialized
    assert_eq!(client.get_is_already_init(), true);

    // Check after querying other functions
    let _ = client.get_total_raised();
    let _ = client.get_donation(&donor);

    // Should still be true
    assert_eq!(client.get_is_already_init(), true);
}

#[test]
fn test_get_goal() {
    let env = Env::default();
    let contract_id = env.register(CrowdfundingContract, ());
    let client = CrowdfundingContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let goal = 100_000_000i128; // 10 XLM
    let deadline = env.ledger().timestamp() + 86400;

    env.mock_all_auths();

    client.initialize(&owner, &goal, &deadline, &Address::generate(&env));

    // Test get_goal returns correct value
    assert_eq!(client.get_goal(), goal);
}

#[test]
fn test_get_deadline() {
    let env = Env::default();
    let contract_id = env.register(CrowdfundingContract, ());
    let client = CrowdfundingContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let goal = 100_000_000i128;
    let deadline = env.ledger().timestamp() + 86400; // 24 hours

    env.mock_all_auths();

    client.initialize(&owner, &goal, &deadline, &Address::generate(&env));

    // Test get_deadline returns correct value
    assert_eq!(client.get_deadline(), deadline);
}

// #[test]
// fn test_is_goal_reached() {
//     let env = Env::default();
//     let contract_id = env.register(CrowdfundingContract, ());
//     let client = CrowdfundingContractClient::new(&env, &contract_id);

//     let owner = Address::generate(&env);
//     let donor = Address::generate(&env);
//     let goal = 50_000_000i128; // 5 XLM
//     let deadline = env.ledger().timestamp() + 86400;

//     env.mock_all_auths();

//     client.initialize(&owner, &goal, &deadline, &Address::generate(&env));

//     // Before donation - should be false
//     assert_eq!(client.is_goal_reached(), false);

//     // Donate less than goal
//     client.donate(&donor, &30_000_000); // 3 XLM
//     assert_eq!(client.is_goal_reached(), false);

//     // Donate to reach goal
//     client.donate(&donor, &20_000_000); // 2 XLM more
//     assert_eq!(client.is_goal_reached(), true);

//     // Donate more than goal
//     client.donate(&donor, &10_000_000); // 1 XLM more
//     assert_eq!(client.is_goal_reached(), true); // Still true
// }

#[test]
fn test_is_ended() {
    let env = Env::default();
    let contract_id = env.register(CrowdfundingContract, ());
    let client = CrowdfundingContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let goal = 100_000_000i128;
    let deadline = env.ledger().timestamp() + 1000; // 1000 seconds

    env.mock_all_auths();

    client.initialize(&owner, &goal, &deadline, &Address::generate(&env));

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

// #[test]
// fn test_get_progress_percentage() {
//     let env = Env::default();
//     let contract_id = env.register(CrowdfundingContract, ());
//     let client = CrowdfundingContractClient::new(&env, &contract_id);

//     let owner = Address::generate(&env);
//     let donor = Address::generate(&env);
//     let goal = 100_000_000i128; // 10 XLM
//     let deadline = env.ledger().timestamp() + 86400;

//     env.mock_all_auths();

//     client.initialize(&owner, &goal, &deadline, &Address::generate(&env));

//     // 0% progress
//     assert_eq!(client.get_progress_percentage(), 0);

//     // Donate 25% of goal
//     client.donate(&donor, &25_000_000); // 2.5 XLM
//     assert_eq!(client.get_progress_percentage(), 25);

//     // Donate to reach 50%
//     client.donate(&donor, &25_000_000); // 2.5 XLM more
//     assert_eq!(client.get_progress_percentage(), 50);

//     // Donate to reach 100%
//     client.donate(&donor, &50_000_000); // 5 XLM more
//     assert_eq!(client.get_progress_percentage(), 100);

//     // Donate more than goal - 120%
//     client.donate(&donor, &20_000_000); // 2 XLM more
//     assert_eq!(client.get_progress_percentage(), 120);
// }

// #[test]
// fn test_refund_success() {
//     let env = Env::default();
//     let contract_id = env.register(CrowdfundingContract, ());
//     let client = CrowdfundingContractClient::new(&env, &contract_id);

//     let owner = Address::generate(&env);
//     let donor = Address::generate(&env);
//     let goal = 100_000_000i128; // 10 XLM
//     let deadline = env.ledger().timestamp() + 100;

//     env.mock_all_auths();

//     client.initialize(&owner, &goal, &deadline, &Address::generate(&env));

//     // Make donation
//     let donation_amount = 30_000_000i128; // 3 XLM
//     client.donate(&donor, &donation_amount);

//     // Verify donation recorded
//     assert_eq!(client.get_donation(&donor), donation_amount);
//     assert_eq!(client.get_total_raised(), donation_amount);

//     // Fast forward past deadline (goal not reached)
//     env.ledger().with_mut(|li| {
//         li.timestamp = deadline + 1;
//     });

//     // Refund
//     let refunded = client.refund(&donor);

//     // Verify refund
//     assert_eq!(refunded, donation_amount);
//     assert_eq!(client.get_donation(&donor), 0);
//     assert_eq!(client.get_total_raised(), 0);
// }

// #[test]
// #[should_panic(expected = "Campaign belum berakhir")]
// fn test_refund_before_deadline() {
//     let env = Env::default();
//     env.mock_all_auths();

//     let contract_id = env.register(CrowdfundingContract, ());
//     let client = CrowdfundingContractClient::new(&env, &contract_id);

//     let owner = Address::generate(&env);
//     let donor = Address::generate(&env);
//     let goal = 100_000_000i128;
//     let deadline = env.ledger().timestamp() + 1000;

//     client.initialize(&owner, &goal, &deadline, &Address::generate(&env));
//     client.donate(&donor, &30_000_000);

//     // Try refund before deadline - should panic
//     client.refund(&donor);
// }

// #[test]
// #[should_panic(expected = "Goal sudah tercapai, tidak bisa refund")]
// fn test_refund_when_goal_reached() {
//     let env = Env::default();
//     env.mock_all_auths();

//     let contract_id = env.register(CrowdfundingContract, ());
//     let client = CrowdfundingContractClient::new(&env, &contract_id);

//     let owner = Address::generate(&env);
//     let donor = Address::generate(&env);
//     let goal = 50_000_000i128; // 5 XLM
//     let deadline = env.ledger().timestamp() + 100;

//     client.initialize(&owner, &goal, &deadline, &Address::generate(&env));

//     // Donate exactly goal amount
//     client.donate(&donor, &goal);

//     // Fast forward past deadline
//     env.ledger().with_mut(|li| {
//         li.timestamp = deadline + 1;
//     });

//     // Try refund when goal reached - should panic
//     client.refund(&donor);
// }
// 🎓 STUDENT EXERCISE:
// Add test untuk function yang akan kalian implement!
// Examples:
// - test_get_goal() return correct goal
// - test_is_goal_reached() when goal met
// - test_is_ended() after deadline passes
// - test_get_deadline() return correct timestamp