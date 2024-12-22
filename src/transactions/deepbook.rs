// Copyright (c) Jarry Han (styu12)
// SPDX-License-Identifier: Apache-2.0
//
// This Rust SDK is inspired by the Sui TypeScript SDK and developed independently by Jarry Han (styu12).

use std::{str::FromStr};

use anyhow::anyhow;
use sui_sdk::types::{programmable_transaction_builder::ProgrammableTransactionBuilder, Identifier, TypeTag};
use sui_sdk::types::base_types::{ObjectID, SequenceNumber};
use sui_sdk::types::transaction::ObjectArg;
use crate::DeepBookConfig;

#[derive(Debug)]
pub enum OrderType {
    NoRestriction,
    ImmediateOrCancel,
    FillOrKill,
    PostOnly
}

impl OrderType {
    pub fn as_u8(&self) -> u8 {
        match self {
            OrderType::NoRestriction => 0,
            OrderType::ImmediateOrCancel => 1,
            OrderType::FillOrKill => 2,
            OrderType::PostOnly => 3,
        }
    }
}

#[derive(Debug)]
pub enum SelfMatchingOptions {
    SelfMatchingAllowed,
    CancelTaker,
    CancelMaker
}

impl SelfMatchingOptions {
    pub fn as_u8(&self) -> u8 {
        match self {
            SelfMatchingOptions::SelfMatchingAllowed => 0,
            SelfMatchingOptions::CancelTaker => 1,
            SelfMatchingOptions::CancelMaker => 2,
        }
    }
}

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
        let pool = self
            .config
            .get_pool(pool_key)
            .ok_or_else(|| anyhow!("Pool not found for key: {}", pool_key))?;

        let manager = self
            .config
            .get_balance_manager(manager_key)
            .ok_or_else(|| anyhow!("Balance manager not found for key: {}", manager_key))?;

        let base_coin = self
            .config
            .get_coin(&pool.base_coin)
            .ok_or_else(|| anyhow!("Base coin not found for key: {}", pool.base_coin))?;

        let quote_coin = self
            .config
            .get_coin(&pool.quote_coin)
            .ok_or_else(|| anyhow!("Quote coin not found for key: {}", pool.quote_coin))?;

        let pool = ptb.obj(ObjectArg::SharedObject{
            id: ObjectID::from_hex_literal(&pool.address).unwrap(),
            initial_shared_version: SequenceNumber::from(0),
            mutable: true,
        })?;
        let manager = ptb.obj(ObjectArg::SharedObject{
            id: ObjectID::from_hex_literal(&manager.address).unwrap(),
            initial_shared_version: SequenceNumber::from(0),
            mutable: false,
        })?;

        let base_coin_type = TypeTag::from_str(&base_coin.type_)?;
        let quote_coin_type = TypeTag::from_str(&quote_coin.type_)?;

        ptb.programmable_move_call(
            ObjectID::from_hex_literal(&self.config.deepbook_package_id)?,
            Identifier::new("pool")?,
            Identifier::new("account_open_orders")?,
            vec![base_coin_type, quote_coin_type],
            vec![pool, manager],
        );

        Ok(())
    }
}
