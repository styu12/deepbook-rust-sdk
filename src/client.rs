// Copyright (c) Jarry Han (styu12)
// SPDX-License-Identifier: Apache-2.0
//
// This Rust SDK is inspired by the Sui TypeScript SDK and developed independently by Jarry Han (styu12).

use std::str::FromStr;

use crate::transactions::{
    balance_manager::BalanceManagerContract, deepbook::DeepBookContract,
    deepbook_admin::DeepBookAdminContract, flash_loan::FlashLoanContract,
    governance::GovernanceContract,
};
use crate::utils::config::DeepBookConfig;
use anyhow::{anyhow, Result};
use log::debug;
use sui_sdk::rpc_types;
use sui_sdk::rpc_types::SuiObjectDataOptions;
use sui_sdk::types::programmable_transaction_builder::ProgrammableTransactionBuilder;
use sui_sdk::types::transaction::{CallArg, Command, ObjectArg, TransactionKind};
use sui_sdk::SuiClient;
use sui_sdk::types::base_types::SuiAddress;
use sui_sdk::types::collection_types::VecSet;
use sui_sdk::types::sui_serde::BigInt;
use sui_sdk::types::TypeTag;
use sui_types::base_types::ObjectID;
use sui_types::Identifier;
use sui_types::transaction::Argument;
use sui_types::object::Owner;

/// Main client for managing DeepBook operations.
///
/// `DeepBookClient` provides methods to interact with the DeepBook protocol,
/// including managing balances, executing transactions, and interacting with
/// governance and other features.
pub struct DeepBookClient<'a> {
    /// Sui client instance used to interact with the Sui blockchain.
    client: SuiClient,
    /// Configuration for the DeepBook environment.
    config: &'a DeepBookConfig,
    /// Contract for managing account balances.
    pub balance_manager: BalanceManagerContract<'a>,
    /// Contract for interacting with the DeepBook market.
    pub deep_book: DeepBookContract<'a>,
    /// Contract for administrative tasks in DeepBook.
    pub deep_book_admin: DeepBookAdminContract<'a>,
    /// Contract for flash loan operations.
    pub flash_loans: FlashLoanContract<'a>,
    /// Contract for interacting with governance features.
    pub governance: GovernanceContract<'a>,
}

impl<'a> DeepBookClient<'a> {
    /// Creates a new `DeepBookClient` instance.
    ///
    /// # Arguments
    /// * `config` - A configuration object containing environment details.
    ///
    /// # Returns
    /// A fully initialized `DeepBookClient` instance.
    pub fn new(
        client: SuiClient,
        config: &'a DeepBookConfig,
    ) -> Self {
        let balance_manager = BalanceManagerContract::new(&config);
        let deep_book = DeepBookContract::new(&config);
        let deep_book_admin = DeepBookAdminContract::new(&config);
        let flash_loans = FlashLoanContract::new(&config);
        let governance = GovernanceContract::new(&config);

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

    /// Checks if a pool is whitelisted.
    ///
    /// # Arguments
    /// * `pool_key` - The key identifying the pool.
    ///
    /// # Returns
    /// A boolean indicating whether the pool is whitelisted.
    pub async fn is_whitelisted(&self, _: &str) -> Result<bool, String> {
        // Transaction logic placeholder.
        Ok(true)
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
    ) -> Result<Vec<u8>> {
        // Step 1: create a programmable transaction builder to add commands and create a PTB
        let mut ptb = ProgrammableTransactionBuilder::new();

        // Step 2: Add the `account_open_orders` Move call to the PTB
        if let Err(e) = self.deep_book.account_open_orders(pool_key, manager_key, &mut ptb) {
            eprintln!("Failed to add account_open_orders command to PTB: {}", e);
            return Err(e);
        }

        // Step 3: Execute the PTB and fetch the result
        let pt = ptb.finish();
        let gas_budget = BigInt::from(10_000);
        let tx_data = TransactionKind::ProgrammableTransaction(pt.to_owned());

        let response = self
            .client
            .read_api()
            .dev_inspect_transaction_block(
                SuiAddress::from_str(&self.config.address).unwrap(),
                tx_data,
                Some(gas_budget),
                None,
                None,
            )
            .await?;

        // Step 4: Extract the first return value from the transaction results.
        let order_ids_bcs: Vec<u8> = response.results
            .as_ref()
            .and_then(|results| results.get(0))
            .and_then(|result| result.return_values.get(0))
            .map(|value| value.0.clone())
            .unwrap_or_else(|| Vec::new());

        // Step 5: Parse the VecSet using BCS.
        match bcs::from_bytes::<VecSet<u8>>(&order_ids_bcs) {
            Ok(vec_set) => {
                let order_ids_vec: Vec<u8> = vec_set.contents;
                Ok(order_ids_vec)
            }
            Err(e) => {
                eprintln!("Failed to parse order IDs VecSet: {}", e);
                Err(anyhow::anyhow!("Failed to parse VecSet: {}", e))
            }
        }
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
    ) -> Result<serde_json::Value, anyhow::Error> {
        let mut ptb = ProgrammableTransactionBuilder::new();

        let coin = self
            .config
            .get_coin(coin_key)
            .expect("Coin not found");

        if let Err(e) = self.balance_manager.check_manager_balance(&mut ptb, manager_key, coin) {
            eprintln!("Failed to add check_manager_balance command to PTB: {}", e);
            return Err(e);
        }

        let pt = ptb.finish();
        let gas_budget = BigInt::from(10_000);
        let tx_data = TransactionKind::ProgrammableTransaction(pt.to_owned());

        let response = self
            .client
            .read_api()
            .dev_inspect_transaction_block(
                SuiAddress::from_str(&self.config.address).unwrap(),
                tx_data,
                Some(gas_budget),
                None,
                None,
            )
            .await?;

        let bytes = response
            .results
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Missing results"))?
            .get(0)
            .ok_or_else(|| anyhow::anyhow!("Missing first result"))?
            .return_values
            .get(0)
            .ok_or_else(|| anyhow::anyhow!("Missing return value"))?
            .0
            .clone();

        let parsed_balance: u64 = bcs::from_bytes(&bytes)?;
        let balance_number = parsed_balance as f64;
        let adjusted_balance = balance_number / coin.scalar as f64;

        Ok(serde_json::json!({
            "coinType": coin.type_,
            "balance": format!("{:.9}", adjusted_balance).parse::<f64>()?,
        }))
    }

    pub async fn deposit_into_manager(
        &self,
        ptb: &mut ProgrammableTransactionBuilder,
        manager_key: &str,
        coin_key: &str,
        amount_to_deposit: u64,
    ) -> Result<()> {
        let coin = self
            .config
            .get_coin(coin_key)
            .expect("Coin not found");
        let deposit_input = amount_to_deposit * coin.scalar;
        let coin_type = TypeTag::from_str(&coin.type_)?;

        let split_cmd = Command::SplitCoins(
            Argument::GasCoin,
            vec![ptb.pure(deposit_input)?],
        );
        let target_coin = ptb.command(split_cmd);

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
                        ptb.programmable_move_call(
                            ObjectID::from_hex_literal(&self.config.deepbook_package_id)?,
                            Identifier::new("balance_manager")?,
                            Identifier::new("deposit")?,
                            vec![coin_type],
                            vec![manager_argument, target_coin],
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

    // pub async fn create_and_share_balance_manager(
    //     &self,
    //     sender: SuiAddress,
    // ) -> Result<()> {
    //     // Step 1: create a programmable transaction builder to add commands and create a PTB
    //     let mut ptb = ProgrammableTransactionBuilder::new();
    //
    //     // Create an Argument::Input for Pure 6 value of type u64
    //     let input_value = 10u64;
    //     let input_argument = CallArg::Pure(bcs::to_bytes(&input_value).unwrap());
    //
    //     // Add this input to the builder
    //     ptb.input(input_argument)?;
    //
    //     // we need to find the coin we will use as gas
    //     let coins = self
    //         .client
    //         .coin_read_api()
    //         .get_coins(sender, None, None, None)
    //         .await?;
    //     let coin = coins.data.into_iter().next().unwrap();
    //
    //     // 2) split coin
    //     // the amount we want in the new coin, 1000 MIST
    //     let split_coin_amount = ptb.pure(1000u64)?; // note that we need to specify the u64 type
    //     ptb.command(Command::SplitCoins(
    //         Argument::GasCoin,
    //         vec![split_coin_amount],
    //     ));
    //
    //     // Step 2: Add the `create_and_share_balance_manager` Move call to the PTB
    //     if let Err(e) = self.balance_manager.create_and_share_balance_manager(&mut ptb) {
    //         eprintln!("Failed to add create_and_share_balance_manager command to PTB: {}", e);
    //         return Err(e);
    //     }
    //
    //     // Step 3: Execute the PTB and fetch the result
    //     let pt = ptb.finish();
    //
    //     let gas_budget = BigInt::from(10_000);
    //
    //     let tx_data = TransactionKind::ProgrammableTransaction(pt.to_owned());
    //     println!("Transaction data: {:?}", tx_data);
    //
    //     let response = self
    //         .client
    //         .read_api()
    //         .dev_inspect_transaction_block(
    //             SuiAddress::from_str(&self.config.address).unwrap(),
    //             tx_data,
    //             Some(gas_budget),
    //             None,
    //             None,
    //         )
    //         .await?;
    //     println!("Transaction response: {:?}", response);
    //
    //     Ok(())
    // }
}
