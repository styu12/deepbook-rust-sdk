
use std::{str::FromStr, time::Duration};
use futures::{future, stream::StreamExt};
use serde_json::json;
use anyhow::{bail, Result};
use reqwest::Client;
use shared_crypto::intent::Intent;
use sui_config::{sui_config_dir, SUI_KEYSTORE_FILENAME};
use sui_keys::keystore::{AccountKeystore, FileBasedKeystore};
use sui_sdk::{SuiClient, SuiClientBuilder, types::{
    base_types::{ObjectID, SuiAddress},
}, rpc_types::{Coin, SuiObjectDataOptions}, SUI_COIN_TYPE};
use sui_sdk::rpc_types::SuiTransactionBlockResponseOptions;
use sui_types::programmable_transaction_builder::ProgrammableTransactionBuilder;
use sui_types::quorum_driver_types::ExecuteTransactionRequestType;
use sui_types::transaction::{Transaction, TransactionData};

#[derive(serde::Deserialize)]
struct FaucetResponse {
    task: String,
    error: Option<String>,
}

pub const SUI_FAUCET: &str = "https://faucet.testnet.sui.io/v1/gas"; // testnet faucet
pub const SUI_STATUS: &str = "https://faucet.testnet.sui.io/v1/status"; // testnet status
// TODO: Replace with the Sui Address you want to use for testing.
const SENDER_ADDRESS: &str = "";
const RECIPIENT_ADDRESS: &str = "";

/// Return a sui client to interact with the APIs,
/// the active address of the local wallet, and another address that can be used as a recipient.
///
/// By default, this function will set up a wallet locally if there isn't any, or reuse the
/// existing one and its active address. This function should be used when two addresses are needed,
/// e.g., transferring objects from one address to another.
pub async fn setup_for_write() -> Result<(SuiClient, SuiAddress, SuiAddress), anyhow::Error> {
    let (client, active_address) = setup_for_read().await?;
    // make sure we have some SUI (5_000_000 MIST) on this address
    let coin = fetch_coin(&client, &active_address).await?;
    if coin.is_none() {
        request_tokens_from_faucet(active_address, &client).await?;
    }

    let recipient_address = SuiAddress::from_str(RECIPIENT_ADDRESS).unwrap();

    Ok((client, active_address, recipient_address))
}

/// Return a sui client to interact with the APIs and an active address from the local wallet.
///
/// This function sets up a wallet in case there is no wallet locally,
/// and ensures that the active address of the wallet has SUI on it.
/// If there is no SUI owned by the active address, then it will request
/// SUI from the faucet.
pub async fn setup_for_read() -> Result<(SuiClient, SuiAddress), anyhow::Error> {
    let client = SuiClientBuilder::default().build_testnet().await?;
    println!("Sui testnet version is: {}", client.api_version());

    let active_address = SuiAddress::from_str(SENDER_ADDRESS).unwrap();
    println!("Active address is: {active_address}");

    Ok((client, active_address))
}

/// Return the coin owned by the address that has at least 5_000_000 MIST, otherwise returns None
pub async fn fetch_coin(
    sui: &SuiClient,
    sender: &SuiAddress,
) -> Result<Option<Coin>, anyhow::Error> {
    let coin_type = "0x2::sui::SUI".to_string();
    let coins_stream = sui
        .coin_read_api()
        .get_coins_stream(*sender, Some(coin_type));

    let mut coins = coins_stream
        .skip_while(|c| future::ready(c.balance < 5_000_000))
        .boxed();
    let coin = coins.next().await;
    Ok(coin)
}

/// Request tokens from the Faucet for the given address
#[allow(unused_assignments)]
pub async fn request_tokens_from_faucet(
    address: SuiAddress,
    sui_client: &SuiClient,
) -> Result<(), anyhow::Error> {
    let address_str = address.to_string();
    let json_body = json![{
        "FixedAmountRequest": {
            "recipient": &address_str
        }
    }];

    // make the request to the faucet JSON RPC API for coin
    let client = Client::new();
    let resp = client
        .post(SUI_FAUCET)
        .header("Content-Type", "application/json")
        .json(&json_body)
        .send()
        .await?;
    println!(
        "Faucet request for address {address_str} has status: {}",
        resp.status()
    );
    println!("Waiting for the faucet to complete the gas request...");
    let faucet_resp: FaucetResponse = resp.json().await?;

    let task_id = if let Some(err) = faucet_resp.error {
        bail!("Faucet request was unsuccessful. Error is {err:?}")
    } else {
        faucet_resp.task
    };

    println!("Faucet request task id: {task_id}");

    if let Err(e) = check_faucet_request_status(address, task_id, &client, sui_client).await {
        bail!("Faucet request failed: {e}")
    }

    Ok(())
}

pub async fn check_faucet_request_status(
    address: SuiAddress,
    task_id: String,
    client: &Client,
    sui_client: &SuiClient,
) -> Result<(), anyhow::Error> {
    let json_body = json![{
        "GetBatchSendStatusRequest": {
            "task_id": &task_id
        }
    }];

    let mut coin_id = "".to_string();

    // wait for the faucet to finish the batch of token requests
    loop {
        let resp = client
            .get(SUI_STATUS)
            .header("Content-Type", "application/json")
            .json(&json_body)
            .send()
            .await?;
        let text = resp.text().await?;
        if text.contains("SUCCEEDED") {
            let resp_json: serde_json::Value = serde_json::from_str(&text).unwrap();

            coin_id = <&str>::clone(
                &resp_json
                    .pointer("/status/transferred_gas_objects/sent/0/id")
                    .unwrap()
                    .as_str()
                    .unwrap(),
            )
                .to_string();

            println!("Faucet request succeeded. Coin ID: {coin_id}");

            break;
        } else {
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    }

    // wait until the fullnode has the coin object, and check if it has the same owner
    loop {
        let owner = sui_client
            .read_api()
            .get_object_with_options(
                ObjectID::from_str(&coin_id)?,
                SuiObjectDataOptions::new().with_owner(),
            )
            .await?;

        if owner.owner().is_some() {
            let owner_address = owner.owner().unwrap().get_owner_address()?;
            if owner_address == address {
                break;
            }
        } else {
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    }

    Ok(())
}


// get user coins, only for verified coins,
// if not verified, please use client.read_api().get_owned_objects
pub async fn get_all_coins(
    client: &SuiClient,
    address: SuiAddress,
    coin_type: &str,
) -> Result<Vec<Coin>> {
    let mut cursor = None;
    let mut coins = vec![];

    loop {
        let coins_res = client
            .coin_read_api()
            .get_coins(
                address,
                Some(coin_type.to_string()), cursor, None) // default limit is 50
            .await?;

        coins.extend(coins_res.data);
        if coins_res.has_next_page {
            cursor = coins_res.next_cursor;
            continue;
        }

        return Ok(coins);
    }
}

/// Execute a transaction block with the given programmable transaction builder and sender address.
/// Transaction will be signed based on your local Sui Keystore Configuration. (located at ~/.sui/sui_config/sui.keystore)
pub async fn execute_transaction_block(
    client: &SuiClient,
    ptb: ProgrammableTransactionBuilder,
    sender: SuiAddress,
) -> Result<()> {
    println!("Building the transaction...");
    let pt = ptb.finish();

    // get coins for gas fee
    let coins = get_all_coins(client, sender, SUI_COIN_TYPE).await
        .map_err(|e| anyhow::anyhow!("Failed to get coins for gas fee: {e}"))?;

    // build the transaction data
    let gas_price = client.read_api().get_reference_gas_price().await?;
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

    // sign transaction
    let keystore = FileBasedKeystore::new(&sui_config_dir()?.join(SUI_KEYSTORE_FILENAME))?;
    let signature = keystore.sign_secure(&sender, &tx_data, Intent::sui_transaction())?;

    // execute the transaction
    println!("Executing the transaction...");
    let transaction_response = client
        .quorum_driver_api()
        .execute_transaction_block(
            Transaction::from_data(tx_data, vec![signature]),
            SuiTransactionBlockResponseOptions::full_content(),
            Some(ExecuteTransactionRequestType::WaitForLocalExecution),
        )
        .await?;

    // print transaction results
    println!("------------------------------------");
    println!("Transaction Results");
    println!("[hash]\n {:?}\n", transaction_response.digest.to_string());
    println!("[effect]\n {:?}\n", transaction_response.effects);
    println!("[object changes]:\n {:?}\n", transaction_response.object_changes);

    Ok(())
}

fn main() {
    println!("This is an example utility file.");
}
