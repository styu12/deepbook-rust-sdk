// Copyright (c) Jarry Han (styu12)
// SPDX-License-Identifier: Apache-2.0
//
// This Rust SDK is inspired by the Sui TypeScript SDK and developed independently by Jarry Han (styu12).

use std::str::FromStr;
use std::sync::Arc;
use crate::transactions::{
    balance_manager::BalanceManagerContract, deepbook::DeepBookContract,
    deepbook_admin::DeepBookAdminContract, flash_loan::FlashLoanContract,
    governance::GovernanceContract,
};
use crate::utils::config::{DeepBookConfig};
use anyhow::{anyhow, Context, Result};
use log::debug;
use serde_json::json;
use sui_sdk::rpc_types::{DevInspectResults, SuiObjectDataOptions};
use sui_sdk::types::programmable_transaction_builder::ProgrammableTransactionBuilder;
use sui_sdk::types::transaction::{ObjectArg, TransactionKind};
use sui_sdk::SuiClient;
use sui_sdk::types::base_types::SuiAddress;
use sui_sdk::types::collection_types::VecSet;
use sui_sdk::types::sui_serde::BigInt;
use sui_sdk::types::TypeTag;
use sui_types::base_types::{ObjectID};
use sui_types::{Identifier};
use sui_types::object::Owner;

/// Main client for managing DeepBook operations.
///
/// `DeepBookClient` provides methods to interact with the DeepBook protocol,
/// including managing balances, executing transactions, and interacting with
/// governance and other features.
pub struct DeepBookClient {
    /// Sui client instance used to interact with the Sui blockchain.
    client: Arc<SuiClient>,
    /// Configuration for the DeepBook environment.
    config: Arc<DeepBookConfig>,
    /// Contract for managing account balances.
    pub balance_manager: Arc<BalanceManagerContract>,
    /// Contract for interacting with the DeepBook market.
    pub deep_book: DeepBookContract,
    /// Contract for administrative tasks in DeepBook.
    pub deep_book_admin: DeepBookAdminContract,
    /// Contract for flash loan operations.
    pub flash_loans: FlashLoanContract,
    /// Contract for interacting with governance features.
    pub governance: GovernanceContract,
}

impl DeepBookClient {
    /// Creates a new `DeepBookClient` instance.
    ///
    /// # Arguments
    /// * `config` - A configuration object containing environment details.
    ///
    /// # Returns
    /// A fully initialized `DeepBookClient` instance.
    pub fn new(
        client: Arc<SuiClient>,
        config: Arc<DeepBookConfig>,
    ) -> Self {
        let balance_manager = Arc::new(BalanceManagerContract::new(client.clone(), config.clone()));
        let deep_book = DeepBookContract::new(client.clone(), config.clone(), balance_manager.clone());
        let deep_book_admin = DeepBookAdminContract::new(client.clone(), config.clone());
        let flash_loans = FlashLoanContract::new(client.clone(), config.clone());
        let governance = GovernanceContract::new(client.clone(), config.clone());

        debug!("DeepBook client initialized, config: {:?}", config);

        DeepBookClient {
            client,
            config,
            balance_manager,
            deep_book,
            deep_book_admin,
            flash_loans,
            governance,
        }
    }

    /// Get open orders for a balance manager in a pool.
    ///
    /// # Arguments
    /// * `pool_key` - The key of the pool.
    /// * `manager_key` - The key of the balance manager.
    ///
    /// # Returns
    /// A vector of open order IDs.
    pub async fn account_open_orders(
        &self,
        pool_key: &str,
        manager_key: &str,
    ) -> Result<Vec<u128>> {
        let mut ptb = ProgrammableTransactionBuilder::new();

        self.deep_book
            .account_open_orders(&mut ptb, pool_key, manager_key)
            .await.with_context(|| "Failed to add account_open_orders command to PTB")?;

        let response = self
            .dev_inspect_transaction_results(ptb)
            .await
            .with_context(|| "Failed to inspect transaction results")?;

        let order_ids: VecSet<u128> = parse_data_from_response(&response)
            .with_context(|| "Failed to parse order IDs from dev-inspect-transaction response")?;

        Ok(order_ids.contents)
    }

    /// Checks the balance of a specific coin for a balance manager.
    ///
    /// # Arguments
    /// * `manager_key` - The key identifying the balance manager.
    /// * `coin_key` - The key identifying the coin.
    ///
    /// # Returns
    /// A tuple containing the coin type as a string and its balance as a floating-point number.
    pub async fn check_manager_balance(
        &self,
        manager_key: &str,
        coin_key: &str,
    ) -> Result<serde_json::Value> {
        let mut ptb = ProgrammableTransactionBuilder::new();

        let coin = self
            .config
            .get_coin(coin_key)
            .with_context(|| format!("Coin not found for key: {}", coin_key))?;

        self.balance_manager
            .check_manager_balance(&mut ptb, manager_key, coin_key)
            .await.with_context(|| "Failed to add check_manager_balance command to PTB")?;

        let response = self
            .dev_inspect_transaction_results(ptb)
            .await
            .with_context(|| "Failed to inspect transaction results")?;

        let parsed_balance: u64 = parse_data_from_response(&response)
            .with_context(|| "Failed to parse balance from dev-inspect-transaction response")?;
        let adjusted_balance = parsed_balance as f64 / coin.scalar as f64;

        Ok(json!({
            "coin_type": coin.type_,
            "balance": format!("{:.9}", adjusted_balance).parse::<f64>()?,
        }))
    }

    /// Mint and transfer trade cap to a receiver.
    /// With trade cap, the receiver can place orders via specified BalanceManager.
    /// # Arguments
    /// * `ptb` - ProgrammableTransactionBuilder instance.
    /// * `manager_key` - The key identifying the balance manager.
    /// * `receiver` - The address of the receiver.
    /// # Returns
    /// None on success, or an error.
    pub async fn mint_and_transfer_trade_cap(
        &self,
        ptb: &mut ProgrammableTransactionBuilder,
        manager_key: &str,
        receiver: SuiAddress,
    ) -> Result<(), anyhow::Error> {
        let manager = self
            .config
            .get_balance_manager(manager_key)
            .ok_or_else(|| anyhow!("Manager not found for key {}", manager_key))?;

        let manager_obj = self.client.read_api().get_object_with_options(
            ObjectID::from_hex_literal(&manager.address)?,
            SuiObjectDataOptions::new()
                .with_content()
                .with_type()
                .with_owner(),
        ).await?;

        match manager_obj.owner() {
            Some(owner) => {
                match owner {
                    Owner::Shared { initial_shared_version, .. } => {
                        let initial_shared_version = initial_shared_version.clone();
                        let manager_argument = ptb.obj(ObjectArg::SharedObject {
                            id: ObjectID::from_hex_literal(&manager.address)?,
                            initial_shared_version,
                            mutable: true,
                        })?;
                        let trade_cap = ptb.programmable_move_call(
                            ObjectID::from_hex_literal(&self.config.deepbook_package_id)?,
                            Identifier::new("balance_manager")?,
                            Identifier::new("mint_trade_cap")?,
                            vec![],
                            vec![manager_argument],
                        );

                        let trade_cap_type = TypeTag::from_str(format!("{}::balance_manager::TradeCap", self.config.deepbook_package_id).as_str())?;
                        let receiver_arg = ptb.pure(receiver)?;
                        ptb.programmable_move_call(
                            ObjectID::from_hex_literal("0x2")?,
                            Identifier::new("transfer")?,
                            Identifier::new("public_transfer")?,
                            vec![trade_cap_type],
                            vec![trade_cap, receiver_arg],
                        );
                    }
                    _ => {
                        return Err(anyhow!("BalanceManager must be a shared object"));
                    }
                }
            },
            None => {
                return Err(anyhow!("BalanceManager has no owner"));
            }
        }

        Ok(())
    }

    /// Return the inspection of the transaction block, or an error upon failure.
    /// Use this method to inspect the results of a transaction before executing it.
    /// It does not execute the transaction.
    async fn dev_inspect_transaction_results(
        &self,
        ptb: ProgrammableTransactionBuilder,
    ) -> Result<DevInspectResults> {
        let tx_data = TransactionKind::ProgrammableTransaction(ptb.finish().to_owned());
        let gas_budget = BigInt::from(10_000);

        self.client
            .read_api()
            .dev_inspect_transaction_block(
                SuiAddress::from_str(&self.config.address).with_context(|| "Invalid sender address in configuration")?,
                tx_data,
                Some(gas_budget),
                None,
                None,
            )
            .await
            .with_context(|| "Failed to dev inspect transaction block")
    }
}

/// Parses data from the dev inspect results(Sui RPC response) and returns the deserialized data.
/// Data type must implement serde::de::DeserializeOwned.
fn parse_data_from_response<T: serde::de::DeserializeOwned>(response: &DevInspectResults) -> Result<T> {
    let bytes = response
        .results
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("Transaction response missing results"))?
        .get(0)
        .ok_or_else(|| anyhow::anyhow!("Transaction response missing first result"))?
        .return_values
        .get(0)
        .ok_or_else(|| anyhow::anyhow!("Transaction response missing return value"))?
        .0
        .clone();

    bcs::from_bytes::<T>(&bytes).context("Failed to decode data from BCS bytes")
}
