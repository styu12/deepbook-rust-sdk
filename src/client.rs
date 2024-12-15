// Copyright (c) Jarry Han (styu12)
// SPDX-License-Identifier: Apache-2.0
//
// This Rust SDK is inspired by the Sui TypeScript SDK and developed independently by Jarry Han (styu12).

use crate::transactions::{
    balance_manager::BalanceManagerContract, deepbook::DeepBookContract,
    deepbook_admin::DeepBookAdminContract, flash_loan::FlashLoanContract,
    governance::GovernanceContract,
};
use crate::utils::config::DeepBookConfig;
use anyhow::Result;
use log::debug;

/// Main client for managing DeepBook operations.
///
/// `DeepBookClient` provides methods to interact with the DeepBook protocol,
/// including managing balances, executing transactions, and interacting with
/// governance and other features.
pub struct DeepBookClient<'a> {
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
    pub fn new(config: &'a DeepBookConfig) -> Self {
        let balance_manager = BalanceManagerContract::new(&config);
        let deep_book = DeepBookContract::new(&config);
        let deep_book_admin = DeepBookAdminContract::new(&config);
        let flash_loans = FlashLoanContract::new(&config);
        let governance = GovernanceContract::new(&config);

        debug!("DeepBook client initialized, config: {:?}", config);

        DeepBookClient {
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

    // /// Get open orders for a balance manager in a pool.
    // ///
    // /// # Arguments
    // /// * `pool_key` - The key of the pool.
    // /// * `manager_key` - The key of the balance manager.
    // ///
    // /// # Returns
    // /// A vector of open order IDs.
    // pub async fn account_open_orders(
    //     &self,
    //     pool_key: &str,
    //     manager_key: &str,
    // ) -> Result<Vec<u128>> {
    //     // Step 1: Create a new transaction.
    //     let mut tx = Transaction::new();
    //
    //     // Step 2: Add the accountOpenOrders Move call to the transaction.
    //     tx.add(self.balance_manager.account_open_orders(pool_key, manager_key));
    //
    //     // Step 3: Inspect the transaction block.
    //     let sender = normalize_sui_address(&self.address);
    //     let res = self
    //         .client
    //         .dev_inspect_transaction_block(sender, tx.build())
    //         .await?;
    //
    //     // Step 4: Extract the first return value from the transaction results.
    //     let order_ids_bcs: Vec<u8> = res.results[0]
    //         .return_values
    //         .get(0)
    //         .ok_or_else(|| anyhow::anyhow!("Missing return value"))?
    //         .0
    //         .clone();
    //
    //     // Step 5: Parse the VecSet using BCS.
    //     let vec_set: VecSet = bcs::from_bytes(&order_ids_bcs)?;
    //     Ok(vec_set.constants)
    // }
}
