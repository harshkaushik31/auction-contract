// auction-contract/src/test.rs

#![cfg(test)]

use super::*;
use soroban_sdk::{
    symbol_short, 
    testutils::{Address as _, Ledger},  // Add Ledger trait here
    Address, 
    Env,
};

fn create_auction_contract(env: &Env) -> AuctionContractClient {
    let contract_id = env.register(
        AuctionContract,
        ()
    );
    AuctionContractClient::new(env, &contract_id)
}

#[test]
fn test_initialize() {
    let env = Env::default();
    let client = create_auction_contract(&env);

    client.initialize();
    
    let count = client.get_auction_count();
    assert_eq!(count, 0);
}

#[test]
fn test_create_auction() {
    let env = Env::default();
    env.mock_all_auths();
    
    let client = create_auction_contract(&env);
    client.initialize();

    let seller = Address::generate(&env);
    let item_name = symbol_short!("Laptop");
    let description = symbol_short!("GamingPC");
    let min_bid: i128 = 1000;
    let duration: u64 = 86400; // 1 day

    let auction_id = client.create_auction(
        &seller,
        &item_name,
        &description,
        &min_bid,
        &duration,
    );

    assert_eq!(auction_id, 1);
    
    let count = client.get_auction_count();
    assert_eq!(count, 1);
}

#[test]
fn test_get_auction() {
    let env = Env::default();
    env.mock_all_auths();
    
    let client = create_auction_contract(&env);
    client.initialize();

    let seller = Address::generate(&env);
    let item_name = symbol_short!("Laptop");
    let description = symbol_short!("GamingPC");
    let min_bid: i128 = 1000;
    let duration: u64 = 86400;

    let auction_id = client.create_auction(
        &seller,
        &item_name,
        &description,
        &min_bid,
        &duration,
    );

    let auction = client.get_auction(&auction_id);
    
    assert_eq!(auction.id, auction_id);
    assert_eq!(auction.seller, seller);
    assert_eq!(auction.item_name, item_name);
    assert_eq!(auction.description, description);
    assert_eq!(auction.min_bid, min_bid);
    assert_eq!(auction.highest_bid, 0);
    assert_eq!(auction.highest_bidder, None);
    assert_eq!(auction.active, true);
}

#[test]
fn test_place_bid() {
    let env = Env::default();
    env.mock_all_auths();
    
    let client = create_auction_contract(&env);
    client.initialize();

    let seller = Address::generate(&env);
    let bidder = Address::generate(&env);
    let item_name = symbol_short!("Laptop");
    let description = symbol_short!("GamingPC");
    let min_bid: i128 = 1000;
    let duration: u64 = 86400;

    let auction_id = client.create_auction(
        &seller,
        &item_name,
        &description,
        &min_bid,
        &duration,
    );

    let bid_amount: i128 = 1500;
    client.place_bid(&auction_id, &bidder, &bid_amount);

    let auction = client.get_auction(&auction_id);
    assert_eq!(auction.highest_bid, bid_amount);
    assert_eq!(auction.highest_bidder, Some(bidder));
}

#[test]
fn test_multiple_bids() {
    let env = Env::default();
    env.mock_all_auths();
    
    let client = create_auction_contract(&env);
    client.initialize();

    let seller = Address::generate(&env);
    let bidder1 = Address::generate(&env);
    let bidder2 = Address::generate(&env);
    let bidder3 = Address::generate(&env);
    
    let item_name = symbol_short!("Laptop");
    let description = symbol_short!("GamingPC");
    let min_bid: i128 = 1000;
    let duration: u64 = 86400;

    let auction_id = client.create_auction(
        &seller,
        &item_name,
        &description,
        &min_bid,
        &duration,
    );

    // First bid
    client.place_bid(&auction_id, &bidder1, &1500);
    let auction = client.get_auction(&auction_id);
    assert_eq!(auction.highest_bid, 1500);
    assert_eq!(auction.highest_bidder, Some(bidder1.clone()));

    // Second bid (higher)
    client.place_bid(&auction_id, &bidder2, &2000);
    let auction = client.get_auction(&auction_id);
    assert_eq!(auction.highest_bid, 2000);
    assert_eq!(auction.highest_bidder, Some(bidder2.clone()));

    // Third bid (even higher)
    client.place_bid(&auction_id, &bidder3, &2500);
    let auction = client.get_auction(&auction_id);
    assert_eq!(auction.highest_bid, 2500);
    assert_eq!(auction.highest_bidder, Some(bidder3));
}

#[test]
#[should_panic(expected = "Bid too low")]
fn test_bid_too_low() {
    let env = Env::default();
    env.mock_all_auths();
    
    let client = create_auction_contract(&env);
    client.initialize();

    let seller = Address::generate(&env);
    let bidder1 = Address::generate(&env);
    let bidder2 = Address::generate(&env);
    
    let item_name = symbol_short!("Laptop");
    let description = symbol_short!("GamingPC");
    let min_bid: i128 = 1000;
    let duration: u64 = 86400;

    let auction_id = client.create_auction(
        &seller,
        &item_name,
        &description,
        &min_bid,
        &duration,
    );

    // First bid
    client.place_bid(&auction_id, &bidder1, &1500);

    // Try to place lower bid - should panic
    client.place_bid(&auction_id, &bidder2, &1400);
}

#[test]
#[should_panic(expected = "Bid below minimum")]
fn test_bid_below_minimum() {
    let env = Env::default();
    env.mock_all_auths();
    
    let client = create_auction_contract(&env);
    client.initialize();

    let seller = Address::generate(&env);
    let bidder = Address::generate(&env);
    
    let item_name = symbol_short!("Laptop");
    let description = symbol_short!("GamingPC");
    let min_bid: i128 = 1000;
    let duration: u64 = 86400;

    let auction_id = client.create_auction(
        &seller,
        &item_name,
        &description,
        &min_bid,
        &duration,
    );

    // Try to place bid below minimum - should panic
    client.place_bid(&auction_id, &bidder, &500);
}

#[test]
fn test_end_auction() {
    let env = Env::default();
    env.mock_all_auths();
    
    let client = create_auction_contract(&env);
    client.initialize();

    let seller = Address::generate(&env);
    let bidder = Address::generate(&env);
    
    let item_name = symbol_short!("Laptop");
    let description = symbol_short!("GamingPC");
    let min_bid: i128 = 1000;
    let duration: u64 = 86400;

    let auction_id = client.create_auction(
        &seller,
        &item_name,
        &description,
        &min_bid,
        &duration,
    );

    // Place a bid
    client.place_bid(&auction_id, &bidder, &1500);

    // Fast forward time past auction end using Ledger trait
    env.ledger().with_mut(|li| {
        li.timestamp = li.timestamp + duration + 1;
    });

    // End the auction
    client.end_auction(&auction_id);

    let auction = client.get_auction(&auction_id);
    assert_eq!(auction.active, false);
}

#[test]
#[should_panic(expected = "Auction not yet ended")]
fn test_end_auction_too_early() {
    let env = Env::default();
    env.mock_all_auths();
    
    let client = create_auction_contract(&env);
    client.initialize();

    let seller = Address::generate(&env);
    let item_name = symbol_short!("Laptop");
    let description = symbol_short!("GamingPC");
    let min_bid: i128 = 1000;
    let duration: u64 = 86400;

    let auction_id = client.create_auction(
        &seller,
        &item_name,
        &description,
        &min_bid,
        &duration,
    );

    // Try to end auction before time is up - should panic
    client.end_auction(&auction_id);
}

#[test]
#[should_panic(expected = "Auction has ended")]
fn test_bid_on_ended_auction_by_time() {
    let env = Env::default();
    env.mock_all_auths();
    
    let client = create_auction_contract(&env);
    client.initialize();

    let seller = Address::generate(&env);
    let bidder = Address::generate(&env);
    
    let item_name = symbol_short!("Laptop");
    let description = symbol_short!("GamingPC");
    let min_bid: i128 = 1000;
    let duration: u64 = 86400;

    let auction_id = client.create_auction(
        &seller,
        &item_name,
        &description,
        &min_bid,
        &duration,
    );

    // Fast forward time past auction end
    env.ledger().with_mut(|li| {
        li.timestamp = li.timestamp + duration + 1;
    });

    // Try to place bid after time expired - should panic
    client.place_bid(&auction_id, &bidder, &1500);
}

#[test]
#[should_panic(expected = "Auction is not active")]
fn test_bid_on_ended_auction() {
    let env = Env::default();
    env.mock_all_auths();
    
    let client = create_auction_contract(&env);
    client.initialize();

    let seller = Address::generate(&env);
    let bidder = Address::generate(&env);
    
    let item_name = symbol_short!("Laptop");
    let description = symbol_short!("GamingPC");
    let min_bid: i128 = 1000;
    let duration: u64 = 86400;

    let auction_id = client.create_auction(
        &seller,
        &item_name,
        &description,
        &min_bid,
        &duration,
    );

    // Fast forward time and end auction
    env.ledger().with_mut(|li| {
        li.timestamp = li.timestamp + duration + 1;
    });
    client.end_auction(&auction_id);

    // Try to place bid on ended auction - should panic
    client.place_bid(&auction_id, &bidder, &1500);
}

#[test]
fn test_multiple_auctions() {
    let env = Env::default();
    env.mock_all_auths();
    
    let client = create_auction_contract(&env);
    client.initialize();

    let seller1 = Address::generate(&env);
    let seller2 = Address::generate(&env);
    let seller3 = Address::generate(&env);

    // Create first auction
    let auction_id1 = client.create_auction(
        &seller1,
        &symbol_short!("Laptop"),
        &symbol_short!("Gaming"),
        &1000,
        &86400,
    );

    // Create second auction
    let auction_id2 = client.create_auction(
        &seller2,
        &symbol_short!("Phone"),
        &symbol_short!("iPhone"),
        &500,
        &86400,
    );

    // Create third auction
    let auction_id3 = client.create_auction(
        &seller3,
        &symbol_short!("Watch"),
        &symbol_short!("Rolex"),
        &2000,
        &86400,
    );

    assert_eq!(auction_id1, 1);
    assert_eq!(auction_id2, 2);
    assert_eq!(auction_id3, 3);

    let count = client.get_auction_count();
    assert_eq!(count, 3);

    // Verify each auction
    let auction1 = client.get_auction(&auction_id1);
    assert_eq!(auction1.seller, seller1);
    assert_eq!(auction1.min_bid, 1000);

    let auction2 = client.get_auction(&auction_id2);
    assert_eq!(auction2.seller, seller2);
    assert_eq!(auction2.min_bid, 500);

    let auction3 = client.get_auction(&auction_id3);
    assert_eq!(auction3.seller, seller3);
    assert_eq!(auction3.min_bid, 2000);
}

#[test]
#[should_panic(expected = "Auction not found")]
fn test_get_nonexistent_auction() {
    let env = Env::default();
    env.mock_all_auths();
    
    let client = create_auction_contract(&env);
    client.initialize();

    // Try to get auction that doesn't exist - should panic
    client.get_auction(&999);
}

#[test]
#[should_panic(expected = "Auction already ended")]
fn test_end_auction_twice() {
    let env = Env::default();
    env.mock_all_auths();
    
    let client = create_auction_contract(&env);
    client.initialize();

    let seller = Address::generate(&env);
    let item_name = symbol_short!("Laptop");
    let description = symbol_short!("GamingPC");
    let min_bid: i128 = 1000;
    let duration: u64 = 86400;

    let auction_id = client.create_auction(
        &seller,
        &item_name,
        &description,
        &min_bid,
        &duration,
    );

    // Fast forward time past auction end
    env.ledger().with_mut(|li| {
        li.timestamp = li.timestamp + duration + 1;
    });

    // End the auction once
    client.end_auction(&auction_id);

    // Try to end again - should panic
    client.end_auction(&auction_id);
}

#[test]
fn test_auction_with_no_bids() {
    let env = Env::default();
    env.mock_all_auths();
    
    let client = create_auction_contract(&env);
    client.initialize();

    let seller = Address::generate(&env);
    let item_name = symbol_short!("Laptop");
    let description = symbol_short!("GamingPC");
    let min_bid: i128 = 1000;
    let duration: u64 = 86400;

    let auction_id = client.create_auction(
        &seller,
        &item_name,
        &description,
        &min_bid,
        &duration,
    );

    // Fast forward time past auction end
    env.ledger().with_mut(|li| {
        li.timestamp = li.timestamp + duration + 1;
    });

    // End the auction with no bids
    client.end_auction(&auction_id);

    let auction = client.get_auction(&auction_id);
    assert_eq!(auction.highest_bid, 0);
    assert_eq!(auction.highest_bidder, None);
    assert_eq!(auction.active, false);
}

#[test]
fn test_same_bidder_increases_bid() {
    let env = Env::default();
    env.mock_all_auths();
    
    let client = create_auction_contract(&env);
    client.initialize();

    let seller = Address::generate(&env);
    let bidder = Address::generate(&env);
    
    let item_name = symbol_short!("Laptop");
    let description = symbol_short!("GamingPC");
    let min_bid: i128 = 1000;
    let duration: u64 = 86400;

    let auction_id = client.create_auction(
        &seller,
        &item_name,
        &description,
        &min_bid,
        &duration,
    );

    // First bid
    client.place_bid(&auction_id, &bidder, &1500);
    
    // Same bidder increases bid
    client.place_bid(&auction_id, &bidder, &2000);

    let auction = client.get_auction(&auction_id);
    assert_eq!(auction.highest_bid, 2000);
    assert_eq!(auction.highest_bidder, Some(bidder));
}