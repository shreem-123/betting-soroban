

#![no_std]

use shared::rand::*;

use soroban_sdk::storage::Persistent;
use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, panic_with_error, token, Address, Env,
    Map, Symbol, Vec,
};

#[derive(Clone, Copy)]
#[contracttype]
enum DataKey {
    Admin = 1,
    Candidates = 2,
    MaxWinnerCount = 3,
    TicketPrice = 4,
    Token = 5,
    AlreadyInitialized = 6,
    AlreadyPlayed = 7,
}


#[contracterror]
#[derive(Clone, Debug, Copy, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
 
    AlreadyInitialized = 1,
  
    InsufficientFunds = 2,
  
    AlreadyPlayed = 3,
  
    MinParticipantsNotSatisfied = 4,
   
    InvalidMaxWinners = 5,
    
    MinimumTicketPrice = 6,
    
    NotInitialized = 7,
}

#[contract]
pub struct BettingContract;

#[contractimpl]
impl BettingContract {
    pub fn init_bet(
        env: Env,
        admin: Address,
        token: Address,
        max_winners_count: u32,
        ticket_price: i128,
    ) {
        admin.require_auth();
        let storage = env.storage().persistent();

        if max_winners_count == 0 {
            panic_with_error!(&env, Error::InvalidMaxWinners);
        }

        if ticket_price <= 1 {
            panic_with_error!(&env, Error::MinimumTicketPrice);
        }

        if storage
            .get::<_, bool>(&DataKey::AlreadyInitialized)
            .is_some()
        {
            panic_with_error!(&env, Error::AlreadyInitialized);
        }

        storage.set(&DataKey::Admin, &admin);
        storage.set(&DataKey::Token, &token);
        storage.set(&DataKey::MaxWinnerCount, &max_winners_count);
        storage.set(&DataKey::TicketPrice, &ticket_price);
        storage.set(&DataKey::Candidates, &Vec::<Address>::new(&env));
        storage.set(&DataKey::AlreadyPlayed, &false);
        storage.set(&DataKey::AlreadyInitialized, &true);
    }
    pub fn register(_: Env, by: Address) {
        by.require_auth();
    }

    pub fn buy_ticket(env: Env, by: Address) -> Result<u32, Error> {
        by.require_auth();

        let storage = env.storage().persistent();

        must_be_initialized_and_not_already_played(&storage)?;

        let price = storage.get::<_, i128>(&DataKey::TicketPrice).unwrap();
        let token = storage.get::<_, Address>(&DataKey::Token).unwrap();
        let token_client = token::Client::new(&env, &token);

        if token_client.balance(&by) <= price {
            return Err(Error::InsufficientFunds);
        }

        token_client.transfer(&by, &env.current_contract_address(), &price);

        let mut candidates = storage
            .get::<_, Vec<Address>>(&DataKey::Candidates)
            .unwrap();
        candidates.push_back(by);
        storage.set(&DataKey::Candidates, &candidates);
        Ok(candidates.len())
    }
    pub fn play_bet(env: Env, random_seed: u64) -> Result<(), Error> {
        let storage = env.storage().persistent();

        must_be_initialized_and_not_already_played(&storage)?;

        let admin = storage.get::<_, Address>(&DataKey::Admin).unwrap();
        admin.require_auth();

        let token: Address = storage.get::<_, Address>(&DataKey::Token).unwrap();

        let token_client = token::Client::new(&env, &token);

        let candidates = storage
            .get::<_, Vec<Address>>(&DataKey::Candidates)
            .unwrap();

        if candidates.is_empty() {
            return Err(Error::MinParticipantsNotSatisfied);
        }

        let max_winners_count = storage.get::<_, u32>(&DataKey::MaxWinnerCount).unwrap();
        let players = candidates.len();

        
        let winners_idx = calculate_winners::<RandomNumberGenerator>(
            &env,
            max_winners_count,
            players,
            random_seed.checked_add(env.ledger().timestamp()).unwrap(), 
        );

        // Pay the winners
        let balance = token_client.balance(&env.current_contract_address());
        let payout = balance / i128::from(max_winners_count);

        for winner in winners_idx {
            let candidate = candidates.get(winner).unwrap();
            token_client.transfer(&env.current_contract_address(), &candidate, &payout);
            let topics = (Symbol::new(&env, "winner"), candidate);
            env.events().publish(topics, payout);
        }
        storage.set(&DataKey::AlreadyPlayed, &true);
        Ok(())
    }
}


fn must_be_initialized_and_not_already_played(storage: &Persistent) -> Result<(), Error> {
    if storage
        .get::<_, bool>(&DataKey::AlreadyInitialized).is_none()
    {
        return Err(Error::NotInitialized);
    }
    if storage
        .get::<_, bool>(&DataKey::AlreadyPlayed).unwrap() 
    {
        return Err(Error::AlreadyPlayed);
    }
    Ok(())
}


fn calculate_winners<T: RandomNumberGeneratorTrait>(
    env: &Env,
    max_winners_count: u32,
    candidates_len: u32,
    random_seed: u64,
) -> Vec<u32> {
    let mut winners = Map::new(env);
    let new_seed = random_seed;
    let mut random_generator = T::new(env, new_seed);

    for _ in 0..max_winners_count {
        let winner = random_generator.number(env, candidates_len) - 1;
        if winners.contains_key(winner) {
            continue;
        }
        winners.set(winner, true);
    }
    winners.keys()
}

