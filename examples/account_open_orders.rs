/// Example: Fetch open orders for a balance manager in a specific pool

mod utils;

use deepbook::client::DeepBookClient;
use deepbook::utils::constants::{BalanceManager, BalanceManagerMap};
use std::collections::HashMap;
use std::sync::Arc;
use tokio;
use deepbook::DeepBookConfig;
use crate::utils::{setup_for_read};

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
    let db_client = DeepBookClient::new(Arc::new(sui.clone()), Arc::new(db_config));

    // Step 5: Define the pool and manager which you want to fetch open orders for
    let pools = vec!["SUI_DBUSDC", "DEEP_SUI"];
    let manager = "MANAGER_1";

    // Step 6: Call account_open_orders with deepbook-sdk and check the response
    println!("------------------------------------");
    println!("Sui RPC Response");
    for pool in pools {
        match db_client.account_open_orders(pool, manager).await {
            Ok(orders) => {
                println!("[pool]\n {:?}", pool);
                println!("[orders]\n {:?}\n", orders);
            },
            Err(e) => println!("Error fetching orders for pool {}: {}", pool, e),
        }
    }
    println!("------------------------------------");

    Ok(())
}