/// Example: Create and share a balance manager

use shared_crypto::intent::Intent;
use sui_config::{sui_config_dir, SUI_KEYSTORE_FILENAME};
use sui_keys::keystore::{AccountKeystore, FileBasedKeystore};
use sui_sdk::rpc_types::SuiTransactionBlockResponseOptions;
use sui_sdk::SUI_COIN_TYPE;
use sui_types::programmable_transaction_builder::ProgrammableTransactionBuilder;
use sui_types::quorum_driver_types::ExecuteTransactionRequestType;
use sui_types::transaction::{Transaction, TransactionData};
use deepbook::{DeepBookClient, DeepBookConfig};
use crate::utils::get_all_coins;

mod utils;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Step 1: Initialize Sui client for writing
    let (sui, sender, receiver) = utils::setup_for_write().await?;

    // Step 2: Define environment
    let env = "testnet";
    let coins = get_all_coins(&sui, sender, SUI_COIN_TYPE).await?;

    // Step 4: Initialize DeepBookClient with DeepBookConfig
    let db_config = DeepBookConfig::new(
        env,
        sender.to_string(),
        None,
        None,
        None,
        None,
    );
    let db_client = DeepBookClient::new(
        sui.clone(),
        &db_config,
    );

    let mut ptb = ProgrammableTransactionBuilder::new();

    match db_client.create_and_share_balance_manager(&mut ptb).await {
        Ok(_) => println!("add create_and_share_balance_manager transaction to PTB"),
        Err(e) => println!("Error creating and sharing balance manager: {}", e),
    }

    println!("Building the transaction...");
    let pt = ptb.finish();

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