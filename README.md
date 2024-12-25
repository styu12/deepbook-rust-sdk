# Deepbook Rust SDK

The **Deepbook Rust SDK** provides a simple and efficient way to interact with the Deepbook protocol. This SDK is inspired by the [Sui TypeScript Deepbook SDK](https://github.com/MystenLabs/sui/tree/main/sdk/deepbook-v3) and offers similar functionality for Rust developers.

## Features

- Comprehensive support for interacting with Deepbook's API.
- Manage accounts, orders, balances, and transactions programmatically.
- Built-in utilities for common operations.

## Installation

To include the Deepbook Rust SDK in your project, add it to your `Cargo.toml`:

```toml
[dependencies]
deepbook = "0.0.1"
```
*Note: Ensure your Rust version is 1.83.0 or higher.*

## Quick Start

Here’s an example of how to use the Deepbook Rust SDK to fetch account balances:

```rust
use deepbook::client::DeepBookClient;

#[tokio::main]
async fn main() {
    // Initialize Sui Client for interacting with the Sui API
    let (sui_client, sender) = setup_for_read().await?;
    
    let db_config = DeepBookConfig::new(
        env,
        sender.to_string(),
        None,
        Some(balance_managers),
        None,
        None,
    );
    
    let client = DeepBookClient::new(
        Arc::new(sui_client.clone()),
        Arc::new(db_config),
    );

    match db_client.check_manager_balance("MANAGER_1", "SUI").await {
        Ok(balance) => {
            println!("[manager balance]\n {:?}\n", balance);
        },
        Err(e) => {
            println!("Error fetching balance");
            for source in e.chain() {
                println!("Caused by: {}", source);
            }
        },
    }
}
```

### Examples

The examples directory contains additional examples for using the SDK:
- [Create and Share a new Balance Manager](./examples/create_and_share_balance_manager.rs)
- [Deposit funds into a balance manager](./examples/deposit_into_manager.rs)
- [Checking manager balances](./examples/check_manager_balance.rs)
- [Place a new Limit Order](./examples/place_limit_order.rs)
- [Fetching open orders for an account](./examples/account_open_orders.rs)

### Documentation

Full API documentation is available at [Sui Deepbook SDK Docs](https://docs.sui.io/standards/deepbookv3-sdk).
<br />*(Currently this documentation is TypeScript SDK Documentation)*