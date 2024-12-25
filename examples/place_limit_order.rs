/// Example: Place a limit order

use std::collections::HashMap;
use shared_crypto::intent::Intent;
use sui_config::{sui_config_dir, SUI_KEYSTORE_FILENAME};
use sui_keys::keystore::{AccountKeystore, FileBasedKeystore};
use sui_sdk::rpc_types::SuiTransactionBlockResponseOptions;
use sui_sdk::SUI_COIN_TYPE;
use sui_types::programmable_transaction_builder::ProgrammableTransactionBuilder;
use sui_types::quorum_driver_types::ExecuteTransactionRequestType;
use sui_types::transaction::{Transaction, TransactionData};
use deepbook::{DeepBookClient, DeepBookConfig};
use deepbook::utils::constants::{BalanceManager, BalanceManagerMap};
use crate::utils::get_all_coins;

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
    let db_client = DeepBookClient::new(
        sui.clone(),
        &db_config,
    );

    let mut ptb = ProgrammableTransactionBuilder::new();

    match db_client.place_limit_order(
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
        Err(e) => println!("Error placing limit order: {}", e),
    }

    let pt = ptb.finish();
    println!("pt: {:?}", pt);

    // tx_data
    let gas_price = sui.read_api().get_reference_gas_price().await?;
    let tx_data = TransactionData::new_programmable(
        sender,
        coins
            .iter()
            .map(|coin| coin.object_ref())
            .collect::<Vec<_>>(),
        pt,
        10_000_000, // gas_budget (0.01 SUI)
        gas_price,
    );

    // 4) sign transaction
    let keystore = FileBasedKeystore::new(&sui_config_dir()?.join(SUI_KEYSTORE_FILENAME))?;
    let signature = keystore.sign_secure(&sender, &tx_data, Intent::sui_transaction())?;

    // execute the transaction
    println!("Executing the transaction...");
    let transaction_response = sui
        .quorum_driver_api()
        .execute_transaction_block(
            Transaction::from_data(tx_data, vec![signature]),
            SuiTransactionBlockResponseOptions::full_content(),
            Some(ExecuteTransactionRequestType::WaitForLocalExecution),
        )
        .await?;

    println!("transaction hash:\n {:?}\n", transaction_response.digest.to_string());
    println!("transaction effect:\n {:?}\n", transaction_response.effects);
    println!("transcation object changes:\n {:?}\n", transaction_response.object_changes);

    Ok(())
}