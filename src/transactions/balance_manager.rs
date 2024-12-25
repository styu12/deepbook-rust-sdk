// Copyright (c) Jarry Han (styu12)
// SPDX-License-Identifier: Apache-2.0
//
// This Rust SDK is inspired by the Sui TypeScript SDK and developed independently by Jarry Han (styu12).

use std::{str::FromStr};
use std::sync::Arc;
use crate::utils::config::DeepBookConfig;
use anyhow::{Context, Result};
use sui_sdk::SuiClient;
use sui_sdk::types::{
    base_types::{ObjectID},
    programmable_transaction_builder::ProgrammableTransactionBuilder,
    Identifier,
    TypeTag,
};
use sui_sdk::types::transaction::{Argument};
use sui_types::transaction::Command;
use crate::utils::transactions::prepare_balance_manager_argument;

/// BalanceManagerContract is a struct for managing BalanceManager smart contract operations.
pub struct BalanceManagerContract {
    client: Arc<SuiClient>,
    config: Arc<DeepBookConfig>,
}

impl BalanceManagerContract {
    pub fn new(client: Arc<SuiClient>, config: Arc<DeepBookConfig>) -> Self {
        Self { client, config }
    }

    /// Create a new BalanceManager object and make it a shared object.
    /// # Arguments
    /// * `ptb` - ProgrammableTransactionBuilder instance.
    /// # Returns
    /// None on success, or an error.
    pub fn create_and_share_balance_manager(
        &self,
        ptb: &mut ProgrammableTransactionBuilder,
    ) -> Result<()> {
        let manager = ptb.programmable_move_call(
            ObjectID::from_hex_literal(&self.config.deepbook_package_id)
                .with_context(|| format!("Invalid package ID for deepbook_package_id: {}", self.config.deepbook_package_id))?,
            Identifier::new("balance_manager")
                .with_context(|| "Invalid identifier for 'balance_manager'")?,
            Identifier::new("new")
                .with_context(|| "Invalid identifier for 'new'")?,
            vec![],
            vec![],
        );

        let balance_manager_type = TypeTag::from_str(
            format!("{}::balance_manager::BalanceManager", self.config.deepbook_package_id).as_str()
        ).with_context(|| "Failed to parse balance manager type")?;

        ptb.programmable_move_call(
            ObjectID::from_hex_literal("0x2")
                .with_context(|| "Invalid package ID for sui framework: 0x2")?,
            Identifier::new("transfer")
                .with_context(|| "Invalid identifier for 'transfer'")?,
            Identifier::new("public_share_object")
                .with_context(|| "Invalid identifier for 'public_share_object'")?,
            vec![balance_manager_type],
            vec![manager],
        );

        Ok(())
    }

    /// Deposit funds into a balance manager.
    /// # Arguments
    /// * `ptb` - ProgrammableTransactionBuilder instance.
    /// * `manager_key` - The key identifying the balance manager.
    /// * `coin_key` - The key identifying the coin.
    /// * `amount_to_deposit` - The amount to deposit.
    /// # Returns
    /// None on success, or an error.
    pub async fn deposit_into_manager(
        &self,
        ptb: &mut ProgrammableTransactionBuilder,
        manager_key: &str,
        coin_key: &str,
        amount_to_deposit: f64,
    ) -> Result<()> {
        let coin = self
            .config
            .get_coin(coin_key)
            .with_context(|| format!("Coin not found for key {}", coin_key))?;
        let deposit_input = (amount_to_deposit * coin.scalar as f64) as u64;
        let deposit_argument = ptb.pure(deposit_input)
            .with_context(|| "Failed to prepare deposit input")?;
        let coin_argument = ptb.command(Command::SplitCoins(
            Argument::GasCoin,
            vec![deposit_argument],
        ));

        let coin_type = TypeTag::from_str(&coin.type_)
            .with_context(|| format!("Failed to parse coin type {}", coin.type_))?;
        let manager_argument = prepare_balance_manager_argument(
            &self.client,
            &self.config,
            ptb,
            manager_key,
        ).await.with_context(|| "Failed to prepare manager argument")?;

        ptb.programmable_move_call(
            ObjectID::from_hex_literal(&self.config.deepbook_package_id)
                .with_context(|| "Invalid package ID for deepbook_package_id")?,
            Identifier::new("balance_manager")
                .with_context(|| "Invalid identifier for 'balance_manager'")?,
            Identifier::new("deposit")
                .with_context(|| "Invalid identifier for 'deposit'")?,
            vec![coin_type],
            vec![manager_argument, coin_argument],
        );

        Ok(())
    }

    /// Check the balance of the BalanceManager.
    /// # Arguments
    /// * `ptb` - ProgrammableTransactionBuilder instance.
    /// * `manager_key` - The key identifying the balance manager.
    /// * `coin_key` - The key identifying the coin.
    ///
    /// # Returns
    /// Ok(()) on success, or an error.
    pub async fn check_manager_balance(
        &self,
        ptb: &mut ProgrammableTransactionBuilder,
        manager_key: &str,
        coin_key: &str,
    ) -> Result<()> {
        let coin = self
            .config
            .get_coin(coin_key)
            .with_context(|| format!("Coin not found for key: {}", coin_key))?;
        let coin_type = TypeTag::from_str(&coin.type_)
            .with_context(|| format!("Failed to parse coin type: {}", coin.type_))?;

        let manager_argument = prepare_balance_manager_argument(
            &self.client,
            &self.config,
            ptb,
            manager_key,
        ).await.with_context(|| "Failed to prepare manager argument for key")?;

        ptb.programmable_move_call(
            ObjectID::from_hex_literal(&self.config.deepbook_package_id)
                .with_context(|| "Invalid package ID for deepbook_package_id")?,
            Identifier::new("balance_manager")
                .with_context(|| "Invalid identifier for 'balance_manager'")?,
            Identifier::new("balance")
                .with_context(|| "Invalid identifier for 'balance'")?,
            vec![coin_type],
            vec![manager_argument],
        );

        Ok(())
    }

    /// Generate a trade proof as the owner.
    pub fn generate_proof_as_owner(
        &self,
        ptb: &mut ProgrammableTransactionBuilder,
        manager_argument: Argument,
    ) -> Argument {
        let trade_proof = ptb.programmable_move_call(
            ObjectID::from_hex_literal(&self.config.deepbook_package_id).unwrap(),
            Identifier::new("balance_manager").unwrap(),
            Identifier::new("generate_proof_as_owner").unwrap(),
            vec![],
            vec![manager_argument],
        );

        trade_proof
    }

    /// Generate a trade proof as a trader.
    pub fn generate_proof_as_trader(
        &self,
        ptb: &mut ProgrammableTransactionBuilder,
        manager_argument: Argument,
        trade_cap_argument: Argument,
    ) -> Argument {
        let trade_proof = ptb.programmable_move_call(
            ObjectID::from_hex_literal(&self.config.deepbook_package_id).unwrap(),
            Identifier::new("balance_manager").unwrap(),
            Identifier::new("generate_proof_as_trader").unwrap(),
            vec![],
            vec![manager_argument, trade_cap_argument],
        );

        trade_proof
    }
}
