// Copyright (c) Jarry Han (styu12)
// SPDX-License-Identifier: Apache-2.0
//
// This Rust SDK is inspired by the Sui TypeScript SDK and developed independently by Jarry Han (styu12).

use std::{str::FromStr};

use anyhow::anyhow;
use sui_sdk::types::{programmable_transaction_builder::ProgrammableTransactionBuilder, transaction::{Argument, Command}, Identifier, TypeTag};
use sui_sdk::types::base_types::{ObjectID, ObjectRef, SequenceNumber, SuiAddress};
use sui_sdk::types::transaction::ObjectArg;
use crate::DeepBookConfig;

pub struct DeepBookContract<'a> {
    config: &'a DeepBookConfig,
}

impl<'a> DeepBookContract<'a> {
    pub fn new(config: &'a DeepBookConfig) -> Self {
        DeepBookContract { config }
    }

    /// Get open orders for a balance manager in a pool.
    ///
    /// # Arguments
    /// * `pool_key` - The key to identify the pool.
    /// * `manager_key` - The key of the balance manager.
    ///
    /// # Returns
    /// The transaction block containing the open orders.
    pub fn account_open_orders(
        &self,
        pool_key: &str,
        manager_key: &str,
        ptb: &mut ProgrammableTransactionBuilder,
    ) -> Result<(), anyhow::Error> {
        // Get the pool from the config.
        let pool = self
            .config
            .get_pool(pool_key)
            .ok_or_else(|| anyhow!("Pool not found for key: {}", pool_key))?;

        // Get the balance manager from the config.
        let manager = self
            .config
            .get_balance_manager(manager_key)
            .ok_or_else(|| anyhow!("Balance manager not found for key: {}", manager_key))?;

        // Get base and quote coins for the pool.
        let base_coin = self
            .config
            .get_coin(&pool.base_coin)
            .ok_or_else(|| anyhow!("Base coin not found for key: {}", pool.base_coin))?;

        let quote_coin = self
            .config
            .get_coin(&pool.quote_coin)
            .ok_or_else(|| anyhow!("Quote coin not found for key: {}", pool.quote_coin))?;

        let pool_address = ptb.pure(SuiAddress::from_str(&pool.address).unwrap())?;
        let manager_address = ptb.pure(SuiAddress::from_str(&manager.address).unwrap())?;

        // TODO: how to get ObjectRef to pass to ptb.obj() & programmable_move_call()?
        //  maybe ptb.pure(objectID) is causing execution error
        // ptb.obj(ObjectArg::SharedObject{
        //     id: ObjectID::from_hex_literal(&pool.address).unwrap(),
        //     initial_shared_version: SequenceNumber::from(0),
        //     mutable: true,
        // })?;

        let base_coin_type = TypeTag::from_str(&base_coin.type_).unwrap();
        let quote_coin_type = TypeTag::from_str(&quote_coin.type_).unwrap();

        ptb.programmable_move_call(
            ObjectID::from_hex_literal(&self.config.deepbook_package_id).unwrap(),
            Identifier::new("pool").unwrap(),
            Identifier::new("account_open_orders").unwrap(),
            vec![base_coin_type, quote_coin_type],
            vec![pool_address, manager_address],
        );

        Ok(())
    }
}
