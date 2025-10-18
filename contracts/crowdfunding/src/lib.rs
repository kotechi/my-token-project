#![no_std]
use soroban_sdk::{contract, contractimpl, symbol_short, token, Address, Env, Map, Symbol};

const CAMPAIGN_GOAL: Symbol = symbol_short!("goal");
const CAMPAIGN_DEADLINE: Symbol = symbol_short!("deadline");
const TOTAL_RAISED: Symbol = symbol_short!("raised");
const DONATIONS: Symbol = symbol_short!("donations");
const CAMPAIGN_OWNER: Symbol = symbol_short!("owner");
const XLM_TOKEN_ADDRESS: Symbol = symbol_short!("xlm_addr");
const IS_ALREADY_INIT: Symbol = symbol_short!("is_init");

#[contract]
pub struct CrowdfundingContract;

#[contractimpl]
impl CrowdfundingContract {

    pub fn initialize(
        env: Env,
        owner: Address,
        goal: i128,
        deadline: u64,
        xlm_token: Address,
    ) {
        owner.require_auth();

        // Check if already initialized
        let is_init: bool = env.storage().instance().get(&IS_ALREADY_INIT).unwrap_or(false);
        if is_init {
            panic!("Contract already initialized");
        }

        env.storage().instance().set(&CAMPAIGN_OWNER, &owner);
        env.storage().instance().set(&CAMPAIGN_GOAL, &goal);
        env.storage().instance().set(&CAMPAIGN_DEADLINE, &deadline);
        env.storage().instance().set(&TOTAL_RAISED, &0i128);
        env.storage().instance().set(&XLM_TOKEN_ADDRESS, &xlm_token);
        env.storage().instance().set(&IS_ALREADY_INIT, &true);

        let donations: Map<Address, i128> = Map::new(&env);
        env.storage().instance().set(&DONATIONS, &donations);
    }

    pub fn donate(env: Env, donor: Address, amount: i128) {
        donor.require_auth();

        // Check initialization
        Self::require_initialized(&env);

        let deadline: u64 = env.storage().instance().get(&CAMPAIGN_DEADLINE).unwrap();
        if env.ledger().timestamp() > deadline {
            panic!("Campaign has ended");
        }

        if amount <= 0 {
            panic!("Donation amount must be positive");
        }

        let xlm_token_address: Address = env.storage().instance().get(&XLM_TOKEN_ADDRESS).unwrap();
        let xlm_token = token::Client::new(&env, &xlm_token_address);
        let contract_address = env.current_contract_address();

        xlm_token.transfer(&donor, &contract_address, &amount);

        let mut total: i128 = env.storage().instance().get(&TOTAL_RAISED).unwrap();
        total += amount;
        env.storage().instance().set(&TOTAL_RAISED, &total);

        let mut donations: Map<Address, i128> = env.storage().instance().get(&DONATIONS).unwrap();
        let current_donation = donations.get(donor.clone()).unwrap_or(0);
        donations.set(donor, current_donation + amount);
        env.storage().instance().set(&DONATIONS, &donations);
    }

    pub fn get_total_raised(env: Env) -> i128 {
        env.storage().instance().get(&TOTAL_RAISED).unwrap_or(0)
    }

    pub fn get_donation(env: Env, donor: Address) -> i128 {
        let donations: Map<Address, i128> = env.storage().instance()
            .get(&DONATIONS)
            .unwrap_or(Map::new(&env));
        donations.get(donor).unwrap_or(0)
    }

    pub fn get_is_already_init(env: Env) -> bool {
        env.storage().instance().get(&IS_ALREADY_INIT).unwrap_or(false)
    }

    // ✅ FIXED: Return default 0 if not initialized
    pub fn get_goal(env: Env) -> i128 {
        env.storage().instance().get(&CAMPAIGN_GOAL).unwrap_or(0)
    }

    // ✅ FIXED: Return default 0 if not initialized
    pub fn get_deadline(env: Env) -> u64 {
        env.storage().instance().get(&CAMPAIGN_DEADLINE).unwrap_or(0)
    }

    pub fn is_goal_reached(env: Env) -> bool {
        let total_raised: i128 = env.storage().instance().get(&TOTAL_RAISED).unwrap_or(0);
        let goal: i128 = env.storage().instance().get(&CAMPAIGN_GOAL).unwrap_or(0);
        if goal == 0 {
            return false;
        }
        total_raised >= goal
    }

    pub fn is_ended(env: Env) -> bool {
        let deadline: u64 = env.storage().instance().get(&CAMPAIGN_DEADLINE).unwrap_or(0);
        if deadline == 0 {
            return false;
        }
        env.ledger().timestamp() > deadline
    }

    pub fn get_progress_percentage(env: Env) -> i128 {
        let total_raised: i128 = env.storage().instance().get(&TOTAL_RAISED).unwrap_or(0);
        let goal: i128 = env.storage().instance().get(&CAMPAIGN_GOAL).unwrap_or(0);
        if goal == 0 {
            return 0;
        }
        (total_raised * 100) / goal
    }

    pub fn refund(env: Env, donor: Address) -> i128 {
        donor.require_auth();
        
        Self::require_initialized(&env);

        let deadline: u64 = env.storage().instance().get(&CAMPAIGN_DEADLINE).unwrap();
        let goal: i128 = env.storage().instance().get(&CAMPAIGN_GOAL).unwrap();
        let total_raised: i128 = env.storage().instance().get(&TOTAL_RAISED).unwrap_or(0);

        if env.ledger().timestamp() <= deadline {
            panic!("Campaign belum berakhir");
        }
        if total_raised >= goal {
            panic!("Goal sudah tercapai, tidak bisa refund");
        }

        let mut donations: Map<Address, i128> = env.storage().instance().get(&DONATIONS).unwrap();
        let donated_amount = donations.get(donor.clone()).unwrap_or(0);

        if donated_amount <= 0 {
            panic!("No donations found for this address");
        }

        let xlm_token_address: Address = env.storage().instance().get(&XLM_TOKEN_ADDRESS).unwrap();
        let xlm_token = token::Client::new(&env, &xlm_token_address);
        let contract_address = env.current_contract_address();

        xlm_token.transfer(&contract_address, &donor, &donated_amount);

        let mut total: i128 = env.storage().instance().get(&TOTAL_RAISED).unwrap();
        total -= donated_amount;
        env.storage().instance().set(&TOTAL_RAISED, &total);

        donations.set(donor, 0);
        env.storage().instance().set(&DONATIONS, &donations);
        
        donated_amount
    }

    // ✅ NEW: Helper function to check initialization
    fn require_initialized(env: &Env) {
        let is_init: bool = env.storage().instance().get(&IS_ALREADY_INIT).unwrap_or(false);
        if !is_init {
            panic!("Contract not initialized");
        }
    }
}

#[cfg(test)]
mod test;