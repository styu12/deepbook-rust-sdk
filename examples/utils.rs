
use std::{str::FromStr, time::Duration};
use futures::{future, stream::StreamExt};
use serde_json::json;
use anyhow::bail;
use reqwest::Client;
use sui_sdk::{SuiClient, SuiClientBuilder, sui_client_config::{SuiClientConfig, SuiEnv}, wallet_context::WalletContext, types::{
    base_types::{ObjectID, SuiAddress},
    crypto::SignatureScheme::ED25519,
    digests::TransactionDigest,
    programmable_transaction_builder::ProgrammableTransactionBuilder,
    quorum_driver_types::ExecuteTransactionRequestType,
    transaction::{Argument, Command, Transaction, TransactionData},
}, rpc_types::{SuiTransactionBlockResponseOptions, Coin, SuiObjectDataOptions}, rpc_types};

#[derive(serde::Deserialize)]
struct FaucetResponse {
    task: String,
    error: Option<String>,
}

pub const SUI_FAUCET: &str = "https://faucet.testnet.sui.io/v1/gas"; // testnet faucet
pub const SUI_STATUS: &str = "https://faucet.testnet.sui.io/v1/status"; // testnet status
// TODO: change the address to the one you want to use for testing
const SENDER_ADDRESS: &str = "0x63caf24ab6dfc41c44fd67e7e117e2f0a4ef0f636fed9ddddde2f1bd230bae8e";
const RECIPIENT_ADDRESS: &str = "0xf4225f3f311cd5aa6ca53df437cb8cbd9d34b1bc979ff89e9356587692e21ebf";

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
) -> anyhow::Result<Vec<rpc_types::Coin>> {
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