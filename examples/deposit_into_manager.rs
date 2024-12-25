/// Example: Deposit into a balance manager

use std::collections::HashMap;
use std::sync::Arc;
use sui_sdk::types::programmable_transaction_builder::ProgrammableTransactionBuilder;
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

    // Step 5: Add deposit_into_manager transaction to PTB with deepbook-sdk
    let mut ptb = ProgrammableTransactionBuilder::new();
    match db_client.balance_manager.deposit_into_manager(
        &mut ptb,
        "MANAGER_1",
        "SUI",
        0.1,
    ).await {
        Ok(_) => println!("add deposit transaction to PTB (0.1 SUI for MANAGER_1)"),
        Err(e) => {
            println!("Error depositing into MANAGER_1");
            for source in e.chain() {
                println!("Caused by: {}", source);
            }
        },
    }

    // Step 6: Execute the transaction block
    if let Err(e) = execute_transaction_block(&sui, ptb, sender).await {
        println!("Error executing transaction block for 'deposit_into_manager'");
        for source in e.chain() {
            println!("Caused by: {}", source);
        }
    }

    Ok(())
}