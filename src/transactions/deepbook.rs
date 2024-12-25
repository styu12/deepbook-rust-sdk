// Copyright (c) Jarry Han (styu12)
// SPDX-License-Identifier: Apache-2.0
//
// This Rust SDK is inspired by the Sui TypeScript SDK and developed independently by Jarry Han (styu12).

use std::{str::FromStr};
use std::sync::Arc;
use anyhow::{anyhow, Context, Result};
use sui_sdk::SuiClient;
use sui_sdk::types::{programmable_transaction_builder::ProgrammableTransactionBuilder, Identifier, TypeTag};
use sui_sdk::types::base_types::{ObjectID};
use crate::DeepBookConfig;
use crate::transactions::balance_manager::BalanceManagerContract;
use crate::utils::config::{FLOAT_SCALAR, MAX_TIMESTAMP};
use crate::utils::transactions::{prepare_balance_manager_argument, prepare_imm_or_owned_object_argument, prepare_pool_argument, prepare_sui_clock_argument};

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

pub struct DeepBookContract {
    client: Arc<SuiClient>,
    config: Arc<DeepBookConfig>,
    balance_manager_contract: Arc<BalanceManagerContract>
}

impl DeepBookContract {
    pub fn new(client: Arc<SuiClient>, config: Arc<DeepBookConfig>, balance_manager_contract: Arc<BalanceManagerContract>) -> Self {
        DeepBookContract { client, config, balance_manager_contract }
    }

    /// Place a limit order in the given pool with specified parameters.
    ///
    /// # Arguments
    /// * `pool_key` - The key to identify the pool.
    /// * `manager_key` - The key of the balance manager.
    /// * `client_order_id` - Unique identifier for the order.
    /// * `price` - Price of the order.
    /// * `quantity` - Quantity of the order.
    /// * `is_bid` - Whether this is a bid order.
    /// * `expiration` - Expiration timestamp for the order.
    ///
    /// # Returns
    /// Ok(()) on success, or an error.
    pub async fn place_limit_order(
        &self,
        ptb: &mut ProgrammableTransactionBuilder,
        pool_key: &str,
        manager_key: &str,
        client_order_id: &str,
        price: f64,
        quantity: f64,
        is_bid: bool,
        expiration: Option<u64>,
        order_type: Option<OrderType>,
        self_matching_option: Option<SelfMatchingOptions>,
        pay_with_deep: Option<bool>,
    ) -> Result<()> {
        // Resolve default values
        let expiration = expiration.unwrap_or(MAX_TIMESTAMP);
        let order_type = order_type.unwrap_or(OrderType::NoRestriction);
        let self_matching_option = self_matching_option.unwrap_or(SelfMatchingOptions::SelfMatchingAllowed);
        let pay_with_deep = pay_with_deep.unwrap_or(true);

        // Resolve required configuration and types
        let pool = self.config.get_pool(pool_key)
            .with_context(|| format!("Pool not found for key: {}", pool_key))?;
        let manager = self.config.get_balance_manager(manager_key)
            .with_context(|| format!("BalanceManager not found for key: {}", manager_key))?;
        let base_coin = self.config.get_coin(&pool.base_coin)
            .with_context(|| format!("Base coin not found for key: {}", pool.base_coin))?;
        let quote_coin = self.config.get_coin(&pool.quote_coin)
            .with_context(|| format!("Quote coin not found for key: {}", pool.quote_coin))?;
        let base_coin_type = TypeTag::from_str(&base_coin.type_)
            .with_context(|| format!("Failed to parse base coin type: {}", base_coin.type_))?;
        let quote_coin_type = TypeTag::from_str(&quote_coin.type_)
            .with_context(|| format!("Failed to parse quote coin type: {}", quote_coin.type_))?;

        // Calculate input price and quantity
        let input_price = ((price * FLOAT_SCALAR as f64 * quote_coin.scalar as f64) / base_coin.scalar as f64).round() as u64;
        let input_quantity = (quantity * base_coin.scalar as f64).round() as u64;

        // Prepare arguments for PTB
        let pool_argument = prepare_pool_argument(&self.client, &self.config, ptb, pool_key)
            .await.with_context(|| "Failed to prepare pool argument")?;
        let manager_argument = prepare_balance_manager_argument(&self.client, &self.config, ptb, manager_key)
            .await.with_context(|| "Failed to prepare manager argument")?;
        let sui_clock_argument = prepare_sui_clock_argument(&self.client, ptb)
            .await.with_context(|| "Failed to prepare SuiClock argument")?;

        let trade_proof_argument = {
            if let Some(trade_cap_id) = &manager.trade_cap {
                let trade_cap_argument = prepare_imm_or_owned_object_argument(&self.client, ptb, trade_cap_id)
                    .await.with_context(|| format!("Failed to prepare trade cap argument for key: {}", trade_cap_id))?;

                self.balance_manager_contract.generate_proof_as_trader(ptb, manager_argument.clone(), trade_cap_argument)
            } else {
                self.balance_manager_contract.generate_proof_as_owner(ptb, manager_argument.clone())
            }
        };

        let client_order_id_u64: u64 = client_order_id.parse::<u64>()
            .map_err(|e| anyhow!("Failed to parse client_order_id: {}", e))?;
        let client_order_id_pure = ptb.pure(client_order_id_u64)
            .with_context(|| "Failed to prepare client_order_id pure argument")?;
        let order_type_pure = ptb.pure(order_type.as_u8())
            .with_context(|| "Failed to prepare order_type pure argument")?;
        let self_matching_option_pure = ptb.pure(self_matching_option.as_u8())
            .with_context(|| "Failed to prepare self_matching_option pure argument")?;
        let input_price_pure = ptb.pure(input_price)
            .with_context(|| "Failed to prepare input_price pure argument")?;
        let input_quantity_pure = ptb.pure(input_quantity)
            .with_context(|| "Failed to prepare input_quantity pure argument")?;
        let is_bid_pure = ptb.pure(is_bid)
            .with_context(|| "Failed to prepare is_bid pure argument")?;
        let pay_with_deep_pure = ptb.pure(pay_with_deep)
            .with_context(|| "Failed to prepare pay_with_deep pure argument")?;
        let expiration_pure = ptb.pure(expiration)
            .with_context(|| "Failed to prepare expiration pure argument")?;

        // Add the programmable Move call
        ptb.programmable_move_call(
            ObjectID::from_hex_literal(&self.config.deepbook_package_id)?,
            Identifier::new("pool")?,
            Identifier::new("place_limit_order")?,
            vec![base_coin_type, quote_coin_type],
            vec![
                pool_argument,
                manager_argument,
                trade_proof_argument,
                client_order_id_pure,
                order_type_pure,
                self_matching_option_pure,
                input_price_pure,
                input_quantity_pure,
                is_bid_pure,
                pay_with_deep_pure,
                expiration_pure,
                sui_clock_argument,
            ],
        );

        Ok(())
    }

    /// Get open orders for a balance manager in a pool.
    ///
    /// # Arguments
    /// * `pool_key` - The key to identify the pool.
    /// * `manager_key` - The key of the balance manager.
    ///
    /// # Returns
    /// The transaction block containing the open orders.
    pub async fn account_open_orders(
        &self,
        ptb: &mut ProgrammableTransactionBuilder,
        pool_key: &str,
        manager_key: &str,
    ) -> Result<()> {
        let pool = self
            .config
            .get_pool(pool_key)
            .with_context(|| format!("Pool not found for key: {}", pool_key))?;
        let base_coin = self
            .config
            .get_coin(&pool.base_coin)
            .with_context(|| format!("Base coin not found for key: {}", pool.base_coin))?;
        let quote_coin = self
            .config
            .get_coin(&pool.quote_coin)
            .with_context(|| format!("Quote coin not found for key: {}", pool.quote_coin))?;
        let base_coin_type = TypeTag::from_str(&base_coin.type_)?;
        let quote_coin_type = TypeTag::from_str(&quote_coin.type_)?;

        let pool_argument = prepare_pool_argument(
            &self.client,
            &self.config,
            ptb,
            pool_key,
        ).await.with_context(|| "Failed to prepare pool argument")?;
        let manager_argument = prepare_balance_manager_argument(
            &self.client,
            &self.config,
            ptb,
            manager_key,
        ).await.with_context(|| "Failed to prepare manager argument")?;

        ptb.programmable_move_call(
            ObjectID::from_hex_literal(&self.config.deepbook_package_id)?,
            Identifier::new("pool")?,
            Identifier::new("account_open_orders")?,
            vec![base_coin_type, quote_coin_type],
            vec![pool_argument, manager_argument],
        );

        Ok(())
    }
}
