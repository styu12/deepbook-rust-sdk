# Deepbook Rust SDK

The **Deepbook Rust SDK** provides a simple and efficient way to interact with the Deepbook protocol. This SDK is inspired by the [Sui TypeScript Deepbook SDK](https://github.com/MystenLabs/sui/tree/main/sdk/deepbook-v3) and offers similar functionality for Rust developers.

## Features

- Comprehensive support for interacting with Deepbook's API.
- Manage accounts, orders, balances, and transactions programmatically.
- Built-in utilities for common operations.
- Fully asynchronous, leveraging the power of Rust's async runtime.

## Installation

To include the Deepbook Rust SDK in your project, add it to your `Cargo.toml`:

```toml
[dependencies]
deepbook-sdk = "0.1.0"
```
*Note: Ensure your Rust version is 1.83.0 or higher.*

## Quick Start

Here’s an example of how to use the Deepbook Rust SDK to fetch account balances:

```rust
use deepbook_sdk::client::DeepbookClient;

#[tokio::main]
async fn main() {
    let client = DeepbookClient::new("https://api.deepbook.com");

    match client.get_balance("account123").await {
        Ok(balance) => println!("Account balance: {}", balance),
        Err(e) => eprintln!("Error fetching balance: {}", e),
    }
}
```

### Examples

The examples directory contains additional examples for using the SDK:
- Fetching open orders for an account.
- Managing liquidity pools.
- Governance-related functions.

### Documentation

Full API documentation is available at [Sui Deepbook Docs](https://docs.sui.io/standards/deepbookv3-sdk).