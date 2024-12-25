/// [WIP] Example: Mint and transfer trade cap

use std::collections::HashMap;
use std::sync::Arc;
use sui_sdk::SUI_COIN_TYPE;
use sui_types::programmable_transaction_builder::ProgrammableTransactionBuilder;
use deepbook::{DeepBookClient, DeepBookConfig};
use deepbook::utils::constants::{BalanceManager, BalanceManagerMap};
use crate::utils::{execute_transaction_block, get_all_coins};

mod utils;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Step 1: Initialize Sui client for writing
    let (sui, sender, receiver) = utils::setup_for_write().await?;

    // Step 2: Define environment
    let env = "testnet";
    let coins = get_all_coins(&sui, sender, SUI_COIN_TYPE).await?;
    println!("Gas coins: {:?}", coins);

    // Step 3: Initialize balance managers
    let mut balance_managers: BalanceManagerMap = HashMap::new();
    balance_managers.insert(
        "MANAGER_1".to_string(),
        BalanceManager {
            address: "0x0cb45faadd6c3769bd825dfd3538e34d6c658a0b55a8caa52e03c46b07aef8b9".to_string(),
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
    let db_client = DeepBookClient::new(
        Arc::new(sui.clone()),
        Arc::new(db_config),
    );

    // Step 5: Add mint_and_transfer_trade_cap transaction to PTB with deepbook-sdk
    let mut ptb = ProgrammableTransactionBuilder::new();
    match db_client.mint_and_transfer_trade_cap(
        &mut ptb,
        "MANAGER_1",
        receiver,
    ).await {
        Ok(_) => {
            println!("Add mint_and_transfer_trade_cap to the transaction block");
        },
        Err(e) => {
            println!("Error minting and transferring trade cap: {:?}", e);
        }
    }

    // Step 6: Execute the transaction block
    if let Err(e) = execute_transaction_block(&sui, ptb, sender).await {
        println!("Error executing transaction block for 'mint_and_transfer_trade_cap'");
        for source in e.chain() {
            println!("Caused by: {}", source);
        }
    }

    Ok(())
}