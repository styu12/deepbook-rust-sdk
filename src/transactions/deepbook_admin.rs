// Copyright (c) Jarry Han (styu12)
// SPDX-License-Identifier: Apache-2.0
//
// This Rust SDK is inspired by the Sui TypeScript SDK and developed independently by Jarry Han (styu12).

use std::sync::Arc;
use sui_sdk::SuiClient;
use crate::DeepBookConfig;

pub struct DeepBookAdminContract {
    client: Arc<SuiClient>,
    config: Arc<DeepBookConfig>,
}

impl DeepBookAdminContract {
    pub fn new(client: Arc<SuiClient>, config: Arc<DeepBookConfig>) -> Self {
        DeepBookAdminContract { client, config }
    }
}
