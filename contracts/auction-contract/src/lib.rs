#![no_std]
use soroban_sdk::{
    contract, contractimpl, contracttype, Address, Env, Symbol
};

#[derive(Clone)]
#[contracttype]
pub struct Auction {
    pub id: u64,
    pub seller: Address,
    pub item_name: Symbol,
    pub description: Symbol,
    pub min_bid: i128,
    pub highest_bid: i128,
    pub highest_bidder: Option<Address>,
    pub end_time: u64,
    pub active: bool,
}

#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    AuctionCounter,
    Auction(u64),
    UserBid(u64, Address),
}

#[contract]
pub struct AuctionContract;

#[contractimpl]
impl AuctionContract {
    // Initialize contract
    pub fn initialize(env: Env) {
        env.storage().instance().set(&DataKey::AuctionCounter, &0u64);
    }

    // Create new auction
    pub fn create_auction(
        env: Env,
        seller: Address,
        item_name: Symbol,
        description: Symbol,
        min_bid: i128,
        duration: u64,
    ) -> u64 {
        seller.require_auth();

        let mut counter: u64 = env
            .storage()
            .instance()
            .get(&DataKey::AuctionCounter)
            .unwrap_or(0);
        
        counter += 1;

        let auction = Auction {
            id: counter,
            seller: seller.clone(),
            item_name,
            description,
            min_bid,
            highest_bid: 0,
            highest_bidder: None,
            end_time: env.ledger().timestamp() + duration,
            active: true,
        };

        env.storage().persistent().set(&DataKey::Auction(counter), &auction);
        env.storage().instance().set(&DataKey::AuctionCounter, &counter);

        counter
    }

    // Place a bid
    pub fn place_bid(env: Env, auction_id: u64, bidder: Address, amount: i128) {
        bidder.require_auth();

        let mut auction: Auction = env
            .storage()
            .persistent()
            .get(&DataKey::Auction(auction_id))
            .expect("Auction not found");

        // Validate auction
        assert!(auction.active, "Auction is not active");
        assert!(
            env.ledger().timestamp() < auction.end_time,
            "Auction has ended"
        );
        assert!(amount > auction.highest_bid, "Bid too low");
        assert!(amount >= auction.min_bid, "Bid below minimum");

        // Update auction
        auction.highest_bid = amount;
        auction.highest_bidder = Some(bidder.clone());

        env.storage().persistent().set(&DataKey::Auction(auction_id), &auction);
        env.storage().persistent().set(&DataKey::UserBid(auction_id, bidder), &amount);
    }

    // End auction and finalize
    pub fn end_auction(env: Env, auction_id: u64) {
        let mut auction: Auction = env
            .storage()
            .persistent()
            .get(&DataKey::Auction(auction_id))
            .expect("Auction not found");

        assert!(auction.active, "Auction already ended");
        assert!(
            env.ledger().timestamp() >= auction.end_time,
            "Auction not yet ended"
        );

        auction.active = false;
        env.storage().persistent().set(&DataKey::Auction(auction_id), &auction);
    }

    // Get auction details
    pub fn get_auction(env: Env, auction_id: u64) -> Auction {
        env.storage()
            .persistent()
            .get(&DataKey::Auction(auction_id))
            .expect("Auction not found")
    }

    // Get total auction count
    pub fn get_auction_count(env: Env) -> u64 {
        env.storage()
            .instance()
            .get(&DataKey::AuctionCounter)
            .unwrap_or(0)
    }
}


#[cfg(test)]
mod test;