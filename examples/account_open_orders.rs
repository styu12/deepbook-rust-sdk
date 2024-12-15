// examples/account_open_orders.rs
// Example: Fetch open orders for a balance manager in a specific pool

mod utils;

use deepbook::client::DeepBookClient;
use deepbook::utils::constants::{BalanceManager, BalanceManagerMap};
use std::collections::HashMap;
use sui_sdk::types::programmable_transaction_builder::ProgrammableTransactionBuilder;
use sui_sdk::types::transaction::CallArg;
use tokio;
use deepbook::DeepBookConfig;
use crate::utils::setup_for_write;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Step 1: Initialize Sui client

    // (1) get the Sui client, the sender and recipient that we will use
    // for the transaction, and find the coin we use as gas
    let (sui, sender, _recipient) = setup_for_write().await?;

    // we need to find the coin we will use as gas
    let coins = sui
        .coin_read_api()
        .get_coins(sender, None, None, None)
        .await?;
    let coin = coins.data.into_iter().next().unwrap();

    // (2) create a programmable transaction builder to add commands and create a PTB
    let mut ptb = ProgrammableTransactionBuilder::new();

    // Create an Argument::Input for Pure 6 value of type u64
    let input_value = 10u64;
    let input_argument = CallArg::Pure(bcs::to_bytes(&input_value).unwrap());

    // Add this input to the builder
    ptb.input(input_argument)?;

    // Step 2: Define environment
    let env = "testnet";

    // Step 3: Initialize balance managers
    let mut balance_managers: BalanceManagerMap = HashMap::new();
    balance_managers.insert(
        "MANAGER_1".to_string(),
        BalanceManager {
            address: "0x344c2734b1d211bd15212bfb7847c66a3b18803f3f5ab00f5ff6f87b6fe6d27d".to_string(),
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