// Copyright (c) Jarry Han (styu12)
// SPDX-License-Identifier: Apache-2.0
//
// This Rust SDK is inspired by the Sui TypeScript SDK and developed independently by Jarry Han (styu12).

use crate::utils::config::DeepBookConfig;
use anyhow::anyhow;
use sui_sdk::types::{
    base_types::ObjectID, programmable_transaction_builder::ProgrammableTransactionBuilder,
    transaction::CallArg, Identifier,
};

#[derive(Debug)]
pub struct BalanceManagerContract<'a> {
    config: &'a DeepBookConfig,
}

impl<'a> BalanceManagerContract<'a> {
    /// Creates a new `BalanceManagerContract`.
    pub fn new(config: &'a DeepBookConfig) -> Self {
        Self { config }
    }

    pub fn create_and_share_balance_manager(&self, ptb: &mut ProgrammableTransactionBuilder) {
        let input_value = 10u64;
        let input_argument = CallArg::Pure(bcs::to_bytes(&input_value).unwrap());

        ptb.input(input_argument).map_err(|e| anyhow!(e)).unwrap();

        // add a move call to the PTB
        let _ = ptb
            .move_call(
                ObjectID::from_hex_literal(&self.config.deepbook_package_id).unwrap(),
                Identifier::new("balance_manager").unwrap(),
                Identifier::new("new").unwrap(),
                vec![],
                vec![],
            )
            .unwrap_or_else(|e| panic!("Error creating balance manager: {}", e));

        // ptb.move_call(
        //     ObjectID::from_hex_literal("0x2").unwrap(),
        //     Identifier::new("transfer").unwrap(),
        //     Identifier::new("public_share_object").unwrap(),
        //     vec![TypeTag::Struct(Box::new(StructTag {
        //         address: AccountAddress::from_hex_literal(&self.config.deepbook_package_id).unwrap(),
        //         module: Identifier::new("balance_manager").unwrap(),
        //         name: Identifier::new("BalanceManager").unwrap(),
        //         type_params: vec![],
        //     }))],
        //     vec![CallArg::Object(ImmOrOwnedObject)],
        // )
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
    //
    // /// Deposit funds into the BalanceManager.
    // pub fn deposit_into_manager(
    //     &self,
    //     tx: &mut Transaction,
    //     manager_key: &str,
    //     coin_key: &str,
    //     amount_to_deposit: u64,
    // ) {
    //     tx.set_sender_if_not_set(&self.config.address);
    //
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
    //     let deposit_input = amount_to_deposit * coin.scalar;
    //
    //     tx.move_call(format!(
    //         "{}::balance_manager::deposit",
    //         self.config.deepbook_package_id
    //     ));
    //     tx.add_arguments(vec![tx.object(&manager_id), deposit_input]);
    //     tx.add_type_arguments(vec![coin.type_.clone()]);
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
    // /// Check the balance of the BalanceManager.
    // pub fn check_manager_balance(
    //     &self,
    //     tx: &mut Transaction,
    //     manager_key: &str,
    //     coin_key: &str,
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
    //     tx.move_call(format!(
    //         "{}::balance_manager::balance",
    //         self.config.deepbook_package_id
    //     ));
    //     tx.add_arguments(vec![tx.object(&manager_id)]);
    //     tx.add_type_arguments(vec![coin.type_.clone()]);
    // }
    //
    // /// Generate a trade proof for the BalanceManager.
    // pub fn generate_proof(&self, tx: &mut Transaction, manager_key: &str) {
    //     let balance_manager = self
    //         .config
    //         .get_balance_manager(manager_key)
    //         .expect("Manager not found");
    //
    //     if let Some(trade_cap) = &balance_manager.trade_cap {
    //         self.generate_proof_as_trader(tx, &balance_manager.address, trade_cap);
    //     } else {
    //         self.generate_proof_as_owner(tx, &balance_manager.address);
    //     }
    // }
    //
    // /// Generate a trade proof as the owner.
    // pub fn generate_proof_as_owner(&self, tx: &mut Transaction, manager_id: &str) {
    //     tx.move_call(format!(
    //         "{}::balance_manager::generate_proof_as_owner",
    //         self.config.deepbook_package_id
    //     ));
    //     tx.add_arguments(vec![tx.object(manager_id)]);
    // }
    //
    // /// Generate a trade proof as a trader.
    // pub fn generate_proof_as_trader(
    //     &self,
    //     tx: &mut Transaction,
    //     manager_id: &str,
    //     trade_cap_id: &str,
    // ) {
    //     tx.move_call(format!(
    //         "{}::balance_manager::generate_proof_as_trader",
    //         self.config.deepbook_package_id
    //     ));
    //     tx.add_arguments(vec![tx.object(manager_id), tx.object(trade_cap_id)]);
    // }
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