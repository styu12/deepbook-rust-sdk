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
use anyhow::Result;
use log::debug;
use sui_sdk::types::programmable_transaction_builder::ProgrammableTransactionBuilder;
use sui_sdk::types::transaction::{CallArg, TransactionData, TransactionKind};
use sui_sdk::SuiClient;
use sui_sdk::types::base_types::SuiAddress;
use sui_sdk::types::collection_types::VecSet;
use sui_sdk::types::sui_serde::BigInt;

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

    /// Checks the balance of a specific coin for a balance manager.
    ///
    /// # Arguments
    /// * `manager_key` - The key identifying the balance manager.
    /// * `coin_key` - The key identifying the coin.
    ///
    /// # Returns
    /// A tuple containing the coin type as a string and its balance as a floating-point number.
    pub async fn check_manager_balance(&self, _: &str, _: &str) -> Result<(String, f64), String> {
        // Transaction logic placeholder.
        Ok(("CoinType".to_string(), 1000.0))
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
    ) -> Result<Vec<u128>> {
        // Step 1: create a programmable transaction builder to add commands and create a PTB
        let mut ptb = ProgrammableTransactionBuilder::new();

        // Create an Argument::Input for Pure 6 value of type u64
        let input_value = 10u64;
        let input_argument = CallArg::Pure(bcs::to_bytes(&input_value).unwrap());

        // Add this input to the builder
        ptb.input(input_argument)?;

        // Step 2: Add the `account_open_orders` Move call to the PTB
        if let Err(e) = self.deep_book.account_open_orders(pool_key, manager_key, &mut ptb) {
            eprintln!("Failed to add account_open_orders command to PTB: {}", e);
            return Err(e);
        }

        // Step 3: Execute the PTB and fetch the result
        let pt = ptb.finish();
        println!("PTB: {:?}", pt);

        let gas_budget = BigInt::from(10_000);

        let tx_data = TransactionKind::ProgrammableTransaction(pt.to_owned());
        println!("Transaction data: {:?}", tx_data);

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
        println!("Transaction response: {:?}", response);

        Ok(vec![])

        // TODO: Implement the rest of the account_open_orders logic - return open order IDs.
        // // Step 4: Extract the first return value from the transaction results.
        // let order_ids_bcs: Vec<u8> = response.results[0]
        //     .return_values
        //     .get(0)
        //     .ok_or_else(|| anyhow::anyhow!("Missing return value"))?
        //     .0
        //     .clone();
        // println!("Order IDs BCS: {:?}", order_ids_bcs);
        //
        // // Step 5: Parse the VecSet using BCS.
        // let vec_set: VecSet<u8> = bcs::from_bytes(&order_ids_bcs)?;
        // println!("Order IDs VecSet: {:?}", vec_set);
        // Ok(vec_set)
    }
}
