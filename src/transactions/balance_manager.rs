// Copyright (c) Jarry Han (styu12)
// SPDX-License-Identifier: Apache-2.0
//
// This Rust SDK is inspired by the Sui TypeScript SDK and developed independently by Jarry Han (styu12).

use std::{str::FromStr};
use crate::utils::config::DeepBookConfig;
use anyhow::anyhow;
use sui_sdk::types::{
    base_types::{ObjectID, SequenceNumber},
    programmable_transaction_builder::ProgrammableTransactionBuilder,
    transaction::CallArg,
    Identifier,
    TypeTag,
};
use sui_sdk::types::transaction::{Argument, ObjectArg};
use crate::utils::constants::Coin;

#[derive(Debug)]
pub struct BalanceManagerContract<'a> {
    config: &'a DeepBookConfig,
}

impl<'a> BalanceManagerContract<'a> {
    /// Creates a new `BalanceManagerContract`.
    pub fn new(config: &'a DeepBookConfig) -> Self {
        Self { config }
    }

    pub fn create_and_share_balance_manager(
        &self,
        ptb: &mut ProgrammableTransactionBuilder,
    ) -> Result<(), anyhow::Error> {
        let manager = ptb.programmable_move_call(
            ObjectID::from_hex_literal(&self.config.deepbook_package_id)?,
            Identifier::new("balance_manager")?,
            Identifier::new("new")?,
            vec![],
            vec![],
        );

        let balance_manager_type = TypeTag::from_str(format!("{}::balance_manager::BalanceManager", self.config.deepbook_package_id).as_str())?;
        ptb.programmable_move_call(
            ObjectID::from_hex_literal("0x2")?,
            Identifier::new("transfer")?,
            Identifier::new("public_share_object")?,
            vec![balance_manager_type],
            vec![manager],
        );

        Ok(())
    }

    /// Check the balance of the BalanceManager.
    pub fn check_manager_balance(
        &self,
        ptb: &mut ProgrammableTransactionBuilder,
        manager_key: &str,
        coin: &Coin,
    ) -> Result<(), anyhow::Error> {
        let manager = self
            .config
            .get_balance_manager(manager_key)
            .ok_or_else(|| anyhow!("Manager not found for key {}", manager_key))?;

        let coin_type = TypeTag::from_str(&coin.type_)?;
        let manager_obj = ptb.obj(ObjectArg::SharedObject {
            id: ObjectID::from_hex_literal(&manager.address)?,
            initial_shared_version: 0.into(),
            mutable: false,
        })?;

        ptb.programmable_move_call(
            ObjectID::from_hex_literal(&self.config.deepbook_package_id)?,
            Identifier::new("balance_manager")?,
            Identifier::new("balance")?,
            vec![coin_type],
            vec![manager_obj],
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

    // /// Create and share a new BalanceManager.
    // pub fn create_and_share_balance_manager_test(&self, tx: &mut Transaction) {
    //     let manager = tx.move_call(format!(
    //         "{}::balance_manager::new",
    //         self.config.deepbook_package_id
    //     ));
    //
    //     tx.move_call(format!("0x2::transfer::public_share_object"));
    //     tx.add_arguments(vec![manager]);
    // }

    // /// Deposit funds into the BalanceManager.
    // pub fn deposit_into_manager(
    //     &self,
    //     ptb: &mut ProgrammableTransactionBuilder,
    //     manager_key: &str,
    //     coin_type: TypeTag,
    //     initial_shared_version: SequenceNumber,
    // ) -> Result<(), anyhow::Error> {
    //
    //     let manager = self
    //         .config
    //         .get_balance_manager(manager_key)
    //         .ok_or_else(|| anyhow!("Manager not found for key {}", manager_key))?;
    //
    //     let manager_obj = ptb.obj(ObjectArg::SharedObject {
    //         id: ObjectID::from_hex_literal(&manager.address)?,
    //         initial_shared_version,
    //         mutable: false,
    //     })?;
    //
    //     ptb.programmable_move_call(
    //         ObjectID::from_hex_literal(&self.config.deepbook_package_id)?,
    //         Identifier::new("balance_manager")?,
    //         Identifier::new("deposit")?,
    //         vec![coin_type],
    //         vec![manager_obj, Argument::Result(0)],
    //     );
    //
    //     Ok(())
    // }
    //
    // /// Withdraw funds from the BalanceManager.
    // pub fn withdraw_from_manager(
    //     &self,
    //     tx: &mut Transaction,
    //     manager_key: &str,
    //     coin_key: &str,
    //     amount_to_withdraw: u64,
    //     recipient: &str,
    // ) {
    //     let manager_id = self
    //         .config
    //         .get_balance_manager(manager_key)
    //         .expect("Manager not found")
    //         .address
    //         .clone();
    //
    //     let coin = self
    //         .config
    //         .get_coin(coin_key)
    //         .expect("Coin not found");
    //
    //     let withdraw_input = amount_to_withdraw * coin.scalar;
    //     let coin_object = tx.move_call(format!(
    //         "{}::balance_manager::withdraw",
    //         self.config.deepbook_package_id
    //     ));
    //     tx.add_arguments(vec![tx.object(&manager_id), withdraw_input]);
    //     tx.add_type_arguments(vec![coin.type_.clone()]);
    //     tx.transfer_objects(vec![coin_object], recipient);
    // }
    //
    // /// Withdraw all funds from the BalanceManager.
    // pub fn withdraw_all_from_manager(
    //     &self,
    //     tx: &mut Transaction,
    //     manager_key: &str,
    //     coin_key: &str,
    //     recipient: &str,
    // ) {
    //     let manager_id = self
    //         .config
    //         .get_balance_manager(manager_key)
    //         .expect("Manager not found")
    //         .address
    //         .clone();
    //
    //     let coin = self
    //         .config
    //         .get_coin(coin_key)
    //         .expect("Coin not found");
    //
    //     let withdrawal_coin = tx.move_call(format!(
    //         "{}::balance_manager::withdraw_all",
    //         self.config.deepbook_package_id
    //     ));
    //     tx.add_arguments(vec![tx.object(&manager_id)]);
    //     tx.add_type_arguments(vec![coin.type_.clone()]);
    //     tx.transfer_objects(vec![withdrawal_coin], recipient);
    // }
    //
    //


    //
    // /// Get the owner of the BalanceManager.
    // pub fn owner(&self, tx: &mut Transaction, manager_key: &str) {
    //     let manager_id = self
    //         .config
    //         .get_balance_manager(manager_key)
    //         .expect("Manager not found")
    //         .address
    //         .clone();
    //
    //     tx.move_call(format!(
    //         "{}::balance_manager::owner",
    //         self.config.deepbook_package_id
    //     ));
    //     tx.add_arguments(vec![tx.object(&manager_id)]);
    // }
    //
    // /// Get the ID of the BalanceManager.
    // pub fn id(&self, tx: &mut Transaction, manager_key: &str) {
    //     let manager_id = self
    //         .config
    //         .get_balance_manager(manager_key)
    //         .expect("Manager not found")
    //         .address
    //         .clone();
    //
    //     tx.move_call(format!(
    //         "{}::balance_manager::id",
    //         self.config.deepbook_package_id
    //     ));
    //     tx.add_arguments(vec![tx.object(&manager_id)]);
    // }
}
