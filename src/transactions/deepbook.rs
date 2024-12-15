// Copyright (c) Jarry Han (styu12)
// SPDX-License-Identifier: Apache-2.0
//
// This Rust SDK is inspired by the Sui TypeScript SDK and developed independently by Jarry Han (styu12).

use crate::DeepBookConfig;

pub struct DeepBookContract<'a> {
    config: &'a DeepBookConfig,
}

impl<'a> DeepBookContract<'a> {
    pub fn new(config: &'a DeepBookConfig) -> Self {
        DeepBookContract { config }
    }

    // /// Get open orders for a balance manager in a pool.
    // ///
    // /// # Arguments
    // /// * `pool_key` - The key to identify the pool.
    // /// * `manager_key` - The key of the balance manager.
    // ///
    // /// # Returns
    // /// The transaction block containing the open orders.
    // pub fn account_open_orders(
    //     &self,
    //     pool_key: &str,
    //     manager_key: &str,
    //     ptb: &mut ProgrammableTransactionBuilder,
    // ) {
    //     // Get the pool from the config.
    //     let pool = self
    //         .config
    //         .get_pool(pool_key)
    //         .ok_or_else(|| anyhow!("Pool not found for key: {}", pool_key))?;
    //
    //     // Get the balance manager from the config.
    //     let manager = self
    //         .config
    //         .get_balance_manager(manager_key)
    //         .ok_or_else(|| anyhow!("Balance manager not found for key: {}", manager_key))?;
    //
    //     // Get base and quote coins for the pool.
    //     let base_coin = self
    //         .config
    //         .get_coin(&pool.base_coin)
    //         .ok_or_else(|| anyhow!("Base coin not found for key: {}", pool.base_coin))?;
    //
    //     let quote_coin = self
    //         .config
    //         .get_coin(&pool.quote_coin)
    //         .ok_or_else(|| anyhow!("Quote coin not found for key: {}", pool.quote_coin))?;
    //
    //     // Add the `account_open_orders` Move call to the transaction.
    //     tx.move_call(
    //         format!("{}::pool::account_open_orders", self.config.deepbook_package_id),
    //         vec![tx.object(&pool.address)?, tx.object(&manager.address)?],
    //         vec![base_coin.type_tag(), quote_coin.type_tag()],
    //     );
    // }
}
