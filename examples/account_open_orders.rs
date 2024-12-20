/// Example: Fetch open orders for a balance manager in a specific pool

mod utils;

use deepbook::client::DeepBookClient;
use deepbook::utils::constants::{BalanceManager, BalanceManagerMap};
use std::collections::HashMap;
use sui_sdk::types::programmable_transaction_builder::ProgrammableTransactionBuilder;
use sui_sdk::types::transaction::CallArg;
use tokio;
use deepbook::DeepBookConfig;
use crate::utils::{setup_for_read, setup_for_write};

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
            // address: "0x6149bfe6808f0d6a9db1c766552b7ae1df477f5885493436214ed4228e842393".to_string(),
            address: "0x0cb45faadd6c3769bd825dfd3538e34d6c658a0b55a8caa52e03c46b07aef8b9".to_string(), // my testnet balance manager
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

    // Step 5: Define pools and fetch open orders
    let pools = vec!["SUI_DBUSDC", "DEEP_SUI"];
    let manager = "MANAGER_1";

    for pool in pools {
        match db_client.account_open_orders(pool, manager).await {
            Ok(orders) => println!("Pool: {}, Orders: {:?}", pool, orders),
            Err(e) => println!("Error fetching orders for pool {}: {}", pool, e),
        }
    }

    Ok(())
}