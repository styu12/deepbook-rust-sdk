/// Example: Place a limit order

use std::collections::HashMap;
use std::sync::Arc;
use sui_types::programmable_transaction_builder::ProgrammableTransactionBuilder;
use deepbook::{DeepBookClient, DeepBookConfig};
use deepbook::utils::constants::{BalanceManager, BalanceManagerMap};
use crate::utils::{execute_transaction_block};

mod utils;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Step 1: Initialize Sui client for writing
    let (sui, sender, _receiver) = utils::setup_for_write().await?;

    // Step 2: Define environment
    let env = "testnet";

    // Step 3: Initialize balance managers
    let mut balance_managers: BalanceManagerMap = HashMap::new();
    balance_managers.insert(
        "MANAGER_1".to_string(),
        BalanceManager {
            address: "0x0cb45faadd6c3769bd825dfd3538e34d6c658a0b55a8caa52e03c46b07aef8b9".to_string(),
            // use trade_cap if you're not the owner of the balance manager
            trade_cap: None,
        },
    );

    // Step 4: Initialize DeepBookClient with DeepBookConfig
    let db_config = DeepBookConfig::new(
        env,
        sender.to_string(),
        None,
        Some(balance_managers),
        None,
        None,
    );
    let db_client = DeepBookClient::new(Arc::new(sui.clone()), Arc::new(db_config));

    // Step 5: Add place_limit_order transaction to PTB with deepbook-sdk
    let mut ptb = ProgrammableTransactionBuilder::new();
    match db_client.deep_book.place_limit_order(
        &mut ptb,
        "DEEP_SUI", // pool_id
        "MANAGER_1", // balance_manager_key
        "123456789", // client_order_id
        0.02, // price
        10.0, // amount
        true, // is_bid
        None,
        None,
        None,
        None,
    ).await {
        Ok(_) => println!("add place_limit_order transaction to PTB"),
        Err(e) => {
            println!("Error placing limit order: {}", e);
            for source in e.chain() {
                println!("Caused by: {}", source);
            }
        },
    }

    // Step 6: Execute the transaction block
    if let Err(e) = execute_transaction_block(&sui, ptb, sender).await {
        println!("Error executing transaction block for 'place_limit_order'");
        for source in e.chain() {
            println!("Caused by: {}", source);
        }
    }

    Ok(())
}