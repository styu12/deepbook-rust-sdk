/// Example: Create and share a balance manager

use std::sync::Arc;
use sui_types::programmable_transaction_builder::ProgrammableTransactionBuilder;
use deepbook::{DeepBookClient, DeepBookConfig};
use crate::utils::{execute_transaction_block};

mod utils;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Step 1: Initialize Sui client for writing
    let (sui, sender, receiver) = utils::setup_for_write().await?;

    // Step 2: Define environment
    let env = "testnet";

    // Step 3: Initialize DeepBookClient with DeepBookConfig
    let db_config = DeepBookConfig::new(
        env,
        sender.to_string(),
        None,
        None,
        None,
        None,
    );
    let db_client = DeepBookClient::new(Arc::new(sui.clone()), Arc::new(db_config));

    // Step 4: Add create_and_share_balance_manager transaction to PTB with deepbook-sdk
    let mut ptb = ProgrammableTransactionBuilder::new();
    match db_client.balance_manager.create_and_share_balance_manager(
        &mut ptb,
    ) {
        Ok(_) => println!("add create_and_share_balance_manager transaction to PTB"),
        Err(e) => {
            println!("Error creating and sharing balance manager");
            for source in e.chain() {
                println!("Caused by: {}", source);
            }
        },
    }

    // Step 5: Execute the transaction block
    if let Err(e) = execute_transaction_block(&sui, ptb, sender).await {
        println!("Error executing transaction block for 'create_and_share_balance_manager'");
        for source in e.chain() {
            println!("Caused by: {}", source);
        }
    }

    Ok(())
}