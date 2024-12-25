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
use crate::utils::config::{DeepBookConfig, FLOAT_SCALAR, MAX_TIMESTAMP};
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
use sui_types::base_types::{ObjectID, ObjectRef, SequenceNumber};
use sui_types::{Identifier, SUI_CLOCK_OBJECT_ID};
use sui_types::transaction::Argument;
use sui_types::object::Owner;
use crate::transactions::deepbook::{OrderType, SelfMatchingOptions};

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

    pub async fn create_and_share_balance_manager(
        &self,
        ptb: &mut ProgrammableTransactionBuilder,
    ) -> Result<()> {
        if let Err(e) = self.balance_manager.create_and_share_balance_manager(ptb) {
            eprintln!("Failed to add create_and_share_balance_manager command to PTB: {}", e);
            return Err(e);
        }

        Ok(())
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
        match bcs::from_bytes::<VecSet<u128>>(&order_ids_bcs) {
            Ok(vec_set) => {
                let order_ids_vec: Vec<u128> = vec_set.contents;
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
            .expect("Coin not found");
        let deposit_input = (amount_to_deposit * coin.scalar as f64) as u64;
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

    /// Mint and transfer trade cap to a receiver.
    /// With trade cap, the receiver can place orders via specified BalanceManager.
    /// # Arguments
    /// * `ptb` - ProgrammableTransactionBuilder instance.
    /// * `manager_key` - The key identifying the balance manager.
    /// * `receiver` - The address of the receiver.
    /// # Returns
    /// None on success, or an error.
    pub async fn mint_and_transfer_trade_cap(
        &self,
        ptb: &mut ProgrammableTransactionBuilder,
        manager_key: &str,
        receiver: SuiAddress,
    ) -> Result<(), anyhow::Error> {
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
                        let trade_cap = ptb.programmable_move_call(
                            ObjectID::from_hex_literal(&self.config.deepbook_package_id)?,
                            Identifier::new("balance_manager")?,
                            Identifier::new("mint_trade_cap")?,
                            vec![],
                            vec![manager_argument],
                        );

                        let trade_cap_type = TypeTag::from_str(format!("{}::balance_manager::TradeCap", self.config.deepbook_package_id).as_str())?;
                        let receiver_arg = ptb.pure(receiver)?;
                        ptb.programmable_move_call(
                            ObjectID::from_hex_literal("0x2")?,
                            Identifier::new("transfer")?,
                            Identifier::new("public_transfer")?,
                            vec![trade_cap_type],
                            vec![trade_cap, receiver_arg],
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
    /// None on success, or an error.
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
    ) -> Result<(), anyhow::Error> {
        // default values
        let expiration = expiration.unwrap_or(MAX_TIMESTAMP);
        let order_type = order_type.unwrap_or(OrderType::NoRestriction);
        let self_matching_option = self_matching_option.unwrap_or(SelfMatchingOptions::SelfMatchingAllowed);
        let pay_with_deep = pay_with_deep.unwrap_or(true);

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

        // Calculate input price and quantity
        // TODO: do I have to multiply by FLOAT_SCALAR?
        let input_price = ((price * FLOAT_SCALAR as f64 * quote_coin.scalar as f64) / base_coin.scalar as f64).round() as u64;
        // let input_price = ((price * quote_coin.scalar as f64) / base_coin.scalar as f64).round() as u64;
        let input_quantity = (quantity * base_coin.scalar as f64).round() as u64;

        // Convert to ObjectArgs
        let mut pool_argument: Option<Argument> = None;
        let mut manager_argument: Option<Argument> = None;
        let mut trade_proof_argument: Option<Argument> = None;

        let pool_obj = self.client.read_api().get_object_with_options(
            ObjectID::from_hex_literal(&pool.address)?,
            SuiObjectDataOptions::new()
                .with_content()
                .with_type()
                .with_owner(),
        ).await?;
        match pool_obj.owner() {
            Some(owner) => {
                match owner {
                    Owner::Shared { initial_shared_version, .. } => {
                        let initial_shared_version = initial_shared_version.clone();
                        pool_argument = Some(ptb.obj(ObjectArg::SharedObject {
                            id: ObjectID::from_hex_literal(&pool.address)?,
                            initial_shared_version,
                            mutable: true,
                        })?);
                    }
                    _ => {
                        return Err(anyhow!("Pool must be a shared object"));
                    }
                }
            }
            _ => {
                return Err(anyhow!("Pool has no owner"));
            }
        }

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
                        manager_argument = Some(ptb.obj(ObjectArg::SharedObject {
                            id: ObjectID::from_hex_literal(&manager.address)?,
                            initial_shared_version,
                            mutable: true,
                        })?);
                    }
                    _ => {
                        return Err(anyhow!("BalanceManager must be a shared object"));
                    }
                }
            }
            _ => {
                return Err(anyhow!("BalanceManager has no owner"));
            }
        }

        if let Some(trade_cap) = &manager.trade_cap {
            let trade_cap_obj = self.client.read_api().get_object_with_options(
                ObjectID::from_hex_literal(trade_cap)?,
                SuiObjectDataOptions::new()
                    .with_content()
                    .with_type()
                    .with_owner(),
            ).await?;
            let trade_cap_argument = ptb.obj(ObjectArg::ImmOrOwnedObject(
                trade_cap_obj
                    .object_ref_if_exists()
                    .ok_or_else(|| anyhow!("Trade cap not found"))?
            ))?;

            trade_proof_argument = Some(self.balance_manager.generate_proof_as_trader(ptb, manager_argument.unwrap(), trade_cap_argument));
        } else {
            trade_proof_argument = Some(self.balance_manager.generate_proof_as_owner(ptb, manager_argument.unwrap()));
        }

        let base_coin_type = TypeTag::from_str(&base_coin.type_)?;
        let quote_coin_type = TypeTag::from_str(&quote_coin.type_)?;

        let sui_clock_obj = self.client.read_api().get_object_with_options(
            ObjectID::from_hex_literal(SUI_CLOCK_OBJECT_ID.to_string().as_str())?,
            SuiObjectDataOptions::new()
                .with_content()
                .with_type()
                .with_owner(),
        ).await?;

        let mut sui_clock_argument: Option<Argument> = None;
        match sui_clock_obj.owner() {
            Some(owner) => {
                match owner {
                    Owner::Shared { initial_shared_version, .. } => {
                        let initial_shared_version = initial_shared_version.clone();
                        sui_clock_argument = Some(ptb.obj(ObjectArg::SharedObject {
                            id: ObjectID::from_hex_literal(SUI_CLOCK_OBJECT_ID.to_string().as_str())?,
                            initial_shared_version,
                            mutable: false,
                        })?);
                    }
                    _ => {
                        return Err(anyhow!("SuiClock must be a shared object"));
                    }
                }
            },
            None => {
                return Err(anyhow!("SuiClock has no owner"));
            }
        }

        println!("client_order_id: {:?}", client_order_id);
        let client_order_id_u64: u64 = client_order_id
            .parse::<u64>()
            .map_err(|e| anyhow!("Failed to parse client_order_id: {}", e))?;
        println!("client_order_id_u64: {:?}", client_order_id_u64);
        let client_order_id_arg = ptb.pure(client_order_id_u64)?;
        let order_type_arg = ptb.pure(order_type.as_u8())?;
        let self_matching_option_arg = ptb.pure(self_matching_option.as_u8())?;
        let input_price_arg = ptb.pure(input_price)?;
        let input_quantity_arg = ptb.pure(input_quantity)?;
        let is_bid_arg = ptb.pure(is_bid)?;
        let pay_with_deep_arg = ptb.pure(pay_with_deep)?;
        let expiration_arg = ptb.pure(expiration)?;

        // Add the programmable Move call
        ptb.programmable_move_call(
            ObjectID::from_hex_literal(&self.config.deepbook_package_id)?,
            Identifier::new("pool")?,
            Identifier::new("place_limit_order")?,
            vec![base_coin_type, quote_coin_type],
            vec![
                pool_argument.unwrap(),
                manager_argument.unwrap(),
                trade_proof_argument.unwrap(),
                client_order_id_arg,
                order_type_arg,
                self_matching_option_arg,
                input_price_arg,
                input_quantity_arg,
                is_bid_arg,
                pay_with_deep_arg,
                expiration_arg,
                sui_clock_argument.unwrap(),
            ],
        );

        Ok(())
    }
}
