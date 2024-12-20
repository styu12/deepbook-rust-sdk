/// Example: Fetch balance of a balance manager

use std::collections::HashMap;
use deepbook::{DeepBookClient, DeepBookConfig};
use deepbook::utils::constants::{BalanceManager, BalanceManagerMap};
use crate::utils::setup_for_read;

mod utils;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Step 1: Initialize Sui client
    let (sui, sender) = setup_for_read().await?;

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
    let db_client = DeepBookClient::new(
        sui,
        &db_config,
    );

    match db_client.check_manager_balance("MANAGER_1", "SUI").await {
        Ok(balance) => println!("Balance: {:?}", balance),
        Err(e) => println!("Error fetching balance: {}", e),
    }

    Ok(())
}