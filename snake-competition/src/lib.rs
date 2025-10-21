#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, symbol_short, token, Address, Env, Symbol, Vec, Map};

// ===== Storage Keys =====
const ADMIN: Symbol = symbol_short!("admin");
const TOKEN: Symbol = symbol_short!("token");
const COMPETITION: Symbol = symbol_short!("comp");
const LEADERBOARD: Symbol = symbol_short!("leader");
const PAID_PLAYERS: Symbol = symbol_short!("paid"); // New: to track players who have paid for a game

// ===== Competition Status =====
const STATUS_ACTIVE: u32 = 1;
// const STATUS_ENDED: u32 = 2;
const STATUS_CLAIMED: u32 = 3;

// ===== Data Structures =====
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Competition {
    pub entry_fee: i128,
    pub session_id: u32,
    pub deadline: u64,
    pub status: u32,
    pub prize_pool: i128,
    pub total_players: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlayerScore {
    pub player: Address,
    pub total_games: u32,
    pub total_score: u64,
    pub rank: u32,
}

// ===== Contract Definition =====
#[contract]
pub struct SnakeGameCompetition;

#[contractimpl]
impl SnakeGameCompetition {
    /// 🔧 Initialize contract (only once)
    pub fn initialize(env: Env, admin: Address, token_address: Address) {
        admin.require_auth();

        if env.storage().instance().has(&ADMIN) {
            panic!("Already initialized");
        }

        env.storage().instance().set(&ADMIN, &admin);
        env.storage().instance().set(&TOKEN, &token_address);
    }

    /// 🏁 Admin creates a new competition session
    pub fn create_competition(env: Env, admin: Address, session_id: u32, deadline: u64, entry_fee: i128) {
        admin.require_auth();
        let stored_admin: Address = env.storage().instance().get(&ADMIN).unwrap();
        if admin != stored_admin {
            panic!("Only admin can create competition");
        }

        // Check active competition
        if let Some(c) = env.storage().instance().get::<Symbol, Competition>(&COMPETITION) {
            if c.status == STATUS_ACTIVE {
                panic!("Competition already active");
            }
        }

        let now = env.ledger().timestamp();
        if deadline <= now {
            panic!("Deadline must be in the future");
        }

        if entry_fee <= 0 {
            panic!("Entry fee must be positive");
        }

        let comp = Competition {
            entry_fee,
            session_id,
            deadline,
            status: STATUS_ACTIVE,
            prize_pool: 0,
            total_players: 0,
        };

        env.storage().instance().set(&COMPETITION, &comp);
        env.storage().instance().set(&LEADERBOARD, &Vec::<PlayerScore>::new(&env));
        env.storage().instance().set(&PAID_PLAYERS, &Map::<Address, bool>::new(&env)); // Reset paid players
    }

    /// 💰 Player pays entry fee before playing (one per game)
    pub fn pay_entry_fee(env: Env, player: Address) {
        player.require_auth();

        let mut comp: Competition = env
            .storage()
            .instance()
            .get(&COMPETITION)
            .expect("No active competition");

        if comp.status != STATUS_ACTIVE {
            panic!("Competition not active");
        }

        let now = env.ledger().timestamp();
        if now >= comp.deadline {
            panic!("Competition has ended");
        }

        let mut paid_players: Map<Address, bool> = env.storage().instance().get(&PAID_PLAYERS).unwrap_or(Map::new(&env));
        if paid_players.get(player.clone()).unwrap_or(false) {
            panic!("Player has already paid for a game; submit score first");
        }

        // Transfer entry fee from player
        let token_address: Address = env.storage().instance().get(&TOKEN).unwrap();
        let entry_fee = comp.entry_fee;
        let token_client = token::Client::new(&env, &token_address);
        let contract_addr = env.current_contract_address();

        token_client.transfer(&player, &contract_addr, &entry_fee);
        comp.prize_pool += entry_fee;

        // Mark player as paid
        paid_players.set(player, true);
        env.storage().instance().set(&PAID_PLAYERS, &paid_players);
        env.storage().instance().set(&COMPETITION, &comp);
    }

    /// 🎮 Player submits score after playing (no payment here)
    pub fn submit_score(env: Env, player: Address, score: u64) {
        player.require_auth();

        let mut comp: Competition = env
            .storage()
            .instance()
            .get(&COMPETITION)
            .expect("No active competition");

        if comp.status != STATUS_ACTIVE {
            panic!("Competition not active");
        }

        let now = env.ledger().timestamp();
        if now >= comp.deadline {
            panic!("Competition has ended");
        }

        let mut paid_players: Map<Address, bool> = env.storage().instance().get(&PAID_PLAYERS).unwrap_or(Map::new(&env));
        if !paid_players.get(player.clone()).unwrap_or(false) {
            panic!("Player must pay entry fee before submitting score");
        }

        // Remove paid status
        paid_players.set(player.clone(), false);
        env.storage().instance().set(&PAID_PLAYERS, &paid_players);

        // Update leaderboard
        let leaderboard: Vec<PlayerScore> =
            env.storage().instance().get(&LEADERBOARD).unwrap_or(Vec::new(&env));

        let mut found = false;
        let mut updated = Vec::new(&env);

        for i in 0..leaderboard.len() {
            let mut p = leaderboard.get(i).unwrap();
            if p.player == player {
                found = true;
                p.total_games += 1;
                p.total_score += score;
            }
            updated.push_back(p);
        }

        if !found {
            comp.total_players += 1;
            updated.push_back(PlayerScore {
                player: player.clone(),
                total_games: 1,
                total_score: score,
                rank: 0,
            });
        }

        // Sort descending by score
        for i in 0..updated.len() {
            for j in 0..(updated.len() - i - 1) {
                let curr = updated.get(j).unwrap();
                let next = updated.get(j + 1).unwrap();
                if curr.total_score < next.total_score {
                    updated.set(j, next.clone());
                    updated.set(j + 1, curr);
                }
            }
        }

        // Assign rank
        let mut final_lb = Vec::new(&env);
        for i in 0..updated.len() {
            let mut ps = updated.get(i).unwrap();
            ps.rank = i + 1;
            final_lb.push_back(ps);
        }

        env.storage().instance().set(&LEADERBOARD, &final_lb);
        env.storage().instance().set(&COMPETITION, &comp);
    }

    /// 🏆 Admin ends competition and distributes prize
    pub fn end_competition(env: Env, admin: Address) {
        admin.require_auth();
        let stored_admin: Address = env.storage().instance().get(&ADMIN).unwrap();
        if admin != stored_admin {
            panic!("Only admin can end competition");
        }

        let mut comp: Competition = env.storage().instance().get(&COMPETITION).unwrap();
        if comp.status != STATUS_ACTIVE {
            panic!("Competition not active");
        }

        // let now = env.ledger().timestamp();
        // if now < comp.deadline {
        //     panic!("Deadline not reached");
        // }

        let lb: Vec<PlayerScore> = env.storage().instance().get(&LEADERBOARD).unwrap_or(Vec::new(&env));
        let prize_pool = comp.prize_pool;

        if prize_pool > 0 && lb.len() > 0 {
            let token_addr: Address = env.storage().instance().get(&TOKEN).unwrap();
            let token_client = token::Client::new(&env, &token_addr);
            let contract_addr = env.current_contract_address();
            let admin_addr = stored_admin;

            // Admin takes 10%
            let admin_fee = (prize_pool * 10) / 100;
            token_client.transfer(&contract_addr, &admin_addr, &admin_fee);

            // Remaining prize pool after admin fee
            let remaining_pool = prize_pool - admin_fee;

            // Rank 1: 50% of remaining
            if lb.len() >= 1 {
                let p = lb.get(0).unwrap();
                let amt = (remaining_pool * 50) / 100;
                token_client.transfer(&contract_addr, &p.player, &amt);
            }

            // Rank 2: 30% of remaining
            if lb.len() >= 2 {
                let p = lb.get(1).unwrap();
                let amt = (remaining_pool * 30) / 100;
                token_client.transfer(&contract_addr, &p.player, &amt);
            }

            // Rank 3: 20% of remaining
            if lb.len() >= 3 {
                let p = lb.get(2).unwrap();
                let amt = (remaining_pool * 20) / 100;
                token_client.transfer(&contract_addr, &p.player, &amt);
            }
        }

        comp.status = STATUS_CLAIMED;
        env.storage().instance().set(&COMPETITION, &comp);
    }

    // ===== View Functions =====
    pub fn get_competition(env: Env) -> Option<Competition> {
        env.storage().instance().get(&COMPETITION)
    }

    pub fn get_leaderboard(env: Env) -> Vec<PlayerScore> {
        env.storage().instance().get(&LEADERBOARD).unwrap_or(Vec::new(&env))
    }

    pub fn get_player_stats(env: Env, player: Address) -> Option<PlayerScore> {
        let lb: Vec<PlayerScore> = env.storage().instance().get(&LEADERBOARD).unwrap_or(Vec::new(&env));
        for i in 0..lb.len() {
            let p = lb.get(i).unwrap();
            if p.player == player {
                return Some(p);
            }
        }
        None
    }

    pub fn get_entry_fee(env: Env) -> i128 {
        if let Some(comp) = env.storage().instance().get::<Symbol, Competition>(&COMPETITION) {
            comp.entry_fee
        } else {
            0
        }
    }

    pub fn get_admin(env: Env) -> Address {
        env.storage().instance().get(&ADMIN).unwrap()
    }

    pub fn has_paid(env: Env, player: Address) -> bool {
        let paid_players: Map<Address, bool> = env.storage().instance().get(&PAID_PLAYERS).unwrap_or(Map::new(&env));
        paid_players.get(player).unwrap_or(false)
    }
}
