// Copyright (c) Jarry Han (styu12)
// SPDX-License-Identifier: Apache-2.0
//
// This Rust SDK is inspired by the Sui TypeScript SDK and developed independently by Jarry Han (styu12).

// use crate::transactions::balance_manager::BalanceManagerContract;
use crate::utils::constants::{
    Coin, CoinMap, Pool, PoolMap, MAINNET_COINS, MAINNET_PACKAGE_IDS, MAINNET_POOLS, TESTNET_COINS,
    TESTNET_PACKAGE_IDS, TESTNET_POOLS,
};

pub const FLOAT_SCALAR: u64 = 1_000_000_000;
pub const MAX_TIMESTAMP: u64 = u64::MAX;
pub const GAS_BUDGET: f64 = 0.5 * 500_000_000.0;
pub const DEEP_SCALAR: u64 = 1_000_000;

/// Represents the configuration for the DeepBook environment.
#[derive(Debug)]
pub struct DeepBookConfig {
    pub coins: CoinMap,
    pub pools: PoolMap,
    // pub balance_managers: HashMap<String, BalanceManager>,
    pub address: String,
    pub deepbook_package_id: String,
    pub registry_id: String,
    pub deep_treasury_id: String,
    pub admin_cap: Option<String>,
    // pub balance_manager_contract: BalanceManagerContract,
}

impl DeepBookConfig {
    /// Creates a new `DeepBookConfig` based on the environment.
    ///
    /// # Arguments
    /// * `env` - The environment (`mainnet` or `testnet`).
    /// * `address` - The user address.
    /// * `admin_cap` - Optional admin capability.
    pub fn new(
        env: &str,
        address: String,
        admin_cap: Option<String>,
        // balance_managers: Option<HashMap<String, BalanceManager>>,
        coins: Option<CoinMap>,
        pools: Option<PoolMap>,
    ) -> Self {
        let (default_coins, default_pools, package_ids) = match env {
            "mainnet" => (&MAINNET_COINS, &MAINNET_POOLS, &MAINNET_PACKAGE_IDS),
            _ => (&TESTNET_COINS, &TESTNET_POOLS, &TESTNET_PACKAGE_IDS),
        };

        Self {
            coins: coins.unwrap_or_else(|| (*default_coins).clone()),
            pools: pools.unwrap_or_else(|| (*default_pools).clone()),
            // balance_managers: balance_managers.unwrap_or_default(),
            address,
            deepbook_package_id: package_ids.deepbook_package_id.to_string(),
            registry_id: package_ids.registry_id.to_string(),
            deep_treasury_id: package_ids.deep_treasury_id.to_string(),
            admin_cap,
            // balance_manager_contract: BalanceManagerContract::new(),
        }
    }

    /// Retrieves a coin by its key.
    pub fn get_coin(&self, key: &str) -> Option<&Coin> {
        self.coins.get(key)
    }

    /// Retrieves a pool by its key.
    pub fn get_pool(&self, key: &str) -> Option<&Pool> {
        self.pools.get(key)
    }

    // /// Retrieves a balance manager by its key.
    // pub fn get_balance_manager(&self, key: &str) -> Option<&BalanceManager> {
    //     self.balance_managers.get(key)
    // }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::constants::{MAINNET_COINS, TESTNET_COINS};

    #[test]
    fn test_config_creation_mainnet() {
        let config = DeepBookConfig::new(
            "mainnet",
            "test_address".to_string(),
            Some("admin_cap".to_string()),
            None,
            None,
        );

        assert_eq!(config.address, "test_address");
        assert_eq!(config.admin_cap, Some("admin_cap".to_string()));
        assert_eq!(
            config.deepbook_package_id,
            MAINNET_PACKAGE_IDS.deepbook_package_id
        );
        assert_eq!(config.coins.len(), MAINNET_COINS.len());
    }

    #[test]
    fn test_config_creation_testnet() {
        let config = DeepBookConfig::new(
            "testnet",
            "test_address".to_string(),
            Some("admin_cap".to_string()),
            None,
            None,
        );

        assert_eq!(config.address, "test_address");
        assert_eq!(config.admin_cap, Some("admin_cap".to_string()));
        assert_eq!(
            config.deepbook_package_id,
            TESTNET_PACKAGE_IDS.deepbook_package_id
        );
        assert_eq!(config.coins.len(), TESTNET_COINS.len());
    }

    #[test]
    fn test_config_custom_coins_and_pools() {
        let custom_coins = CoinMap::new();
        let custom_pools = PoolMap::new();

        let config = DeepBookConfig::new(
            "mainnet",
            "custom_address".to_string(),
            None,
            Some(custom_coins.clone()),
            Some(custom_pools.clone()),
        );

        assert_eq!(config.address, "custom_address");
        assert_eq!(config.coins, custom_coins);
        assert_eq!(config.pools, custom_pools);
    }

    #[test]
    fn test_get_coin() {
        let config = DeepBookConfig::new("testnet", "test_address".to_string(), None, None, None);

        let coin = config.get_coin("DEEP");
        assert!(coin.is_some());
        assert_eq!(
            coin.unwrap().address,
            TESTNET_COINS.get("DEEP").unwrap().address
        );

        let nonexistent_coin = config.get_coin("NONEXISTENT");
        assert!(nonexistent_coin.is_none());
    }

    #[test]
    fn test_get_pool() {
        let config = DeepBookConfig::new("testnet", "test_address".to_string(), None, None, None);

        let pool = config.get_pool("DEEP_SUI");
        assert!(pool.is_some());
        assert_eq!(
            pool.unwrap().address,
            TESTNET_POOLS.get("DEEP_SUI").unwrap().address
        );

        let nonexistent_pool = config.get_pool("NONEXISTENT");
        assert!(nonexistent_pool.is_none());
    }

    #[test]
    fn test_invalid_env_defaults_to_testnet() {
        let config = DeepBookConfig::new("unknown", "test_address".to_string(), None, None, None);

        assert_eq!(
            config.deepbook_package_id,
            TESTNET_PACKAGE_IDS.deepbook_package_id
        );
        assert_eq!(config.coins.len(), TESTNET_COINS.len());
    }
}
