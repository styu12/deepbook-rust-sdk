use anyhow::{Context, Result};
use sui_sdk::rpc_types::{SuiObjectDataOptions, SuiObjectResponse};
use sui_sdk::SuiClient;
use sui_types::base_types::ObjectID;
use sui_types::object::Owner;
use sui_types::programmable_transaction_builder::ProgrammableTransactionBuilder;
use sui_types::SUI_CLOCK_OBJECT_ID;
use sui_types::transaction::{Argument, ObjectArg};
use crate::DeepBookConfig;

/// Helper function to create a BalanceManager `Argument` for PTB using manager_key.
/// BalanceManager is a shared object and must be fetched from SuiClient.
pub async fn prepare_balance_manager_argument(
    client: &SuiClient,
    config: &DeepBookConfig,
    ptb: &mut ProgrammableTransactionBuilder,
    manager_key: &str,
) -> Result<Argument> {
    // Retrieve the manager information
    let manager = config
        .get_balance_manager(manager_key)
        .with_context(|| format!("BalanceManager not found for key: {}", manager_key))?;

    prepare_shared_object_argument(
        client,
        ptb,
        &manager.address,
        &true,
    ).await.with_context(|| format!("Failed to prepare balance manager argument for key: {}", manager_key))
}

/// Helper function to create a Pool `Argument` for PTB using manager_key.
/// Pool is a shared object and must be fetched from SuiClient.
pub async fn prepare_pool_argument(
    client: &SuiClient,
    config: &DeepBookConfig,
    ptb: &mut ProgrammableTransactionBuilder,
    pool_key: &str,
) -> Result<Argument> {
    let pool = config
        .get_pool(pool_key)
        .with_context(|| format!("Pool not found for key: {}", pool_key))?;

    prepare_shared_object_argument(
        client,
        ptb,
        &pool.address,
        &true,
    ).await.with_context(|| format!("Failed to prepare pool argument for key: {}", pool_key))
}

/// Helper function to create a SuiClock `Argument` for PTB.
pub async fn prepare_sui_clock_argument(
    client: &SuiClient,
    ptb: &mut ProgrammableTransactionBuilder,
) -> Result<Argument> {
    prepare_shared_object_argument(
        client,
        ptb,
        SUI_CLOCK_OBJECT_ID.to_string().as_str(),
        &false,
    ).await.with_context(|| "Failed to prepare SuiClock argument")
}


pub async fn prepare_shared_object_argument(
    client: &SuiClient,
    ptb: &mut ProgrammableTransactionBuilder,
    object_id: &str,
    mutable: &bool,
) -> Result<Argument> {
    let object = fetch_object(client, object_id).await?;

    match object.owner() {
        Some(Owner::Shared { initial_shared_version, .. }) => {
            let object_argument = ptb.obj(ObjectArg::SharedObject {
                id: ObjectID::from_hex_literal(object_id)
                    .with_context(|| "Invalid ObjectID")?,
                initial_shared_version: initial_shared_version.clone(),
                mutable: mutable.clone(),
            })
                .with_context(|| format!("Failed to create PTB Argument for object id: {}", object_id))?;

            Ok(object_argument)
        }
        Some(_) => Err(anyhow::anyhow!("Shared Objet must be a shared object")),
        None => Err(anyhow::anyhow!("Shared Objet must have Owner::Shared")),
    }
}

pub async fn prepare_imm_or_owned_object_argument(
    client: &SuiClient,
    ptb: &mut ProgrammableTransactionBuilder,
    object_id: &str,
) -> Result<Argument> {
    let object = fetch_object(client, object_id).await?;

    let object_argument = ptb.obj(ObjectArg::ImmOrOwnedObject(
        object
            .object_ref_if_exists()
            .ok_or_else(|| anyhow::anyhow!("Object not found for id: {}", object_id))?
    )).with_context(|| format!("Failed to create PTB Argument for object id: {}", object_id))?;

    Ok(object_argument)
}

pub async fn fetch_object(
    client: &SuiClient,
    object_id: &str,
) -> Result<SuiObjectResponse> {
    let sui_object_response = client.read_api().get_object_with_options(
            ObjectID::from_hex_literal(object_id)
                .with_context(|| "Invalid ObjectID")?,
            SuiObjectDataOptions::new()
                .with_content()
                .with_type()
                .with_owner(),
        )
        .await
        .with_context(|| format!("Failed to fetch object for id: {}", object_id))?;

    Ok(sui_object_response)
}
