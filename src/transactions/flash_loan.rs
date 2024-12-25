// Copyright (c) Jarry Han (styu12)
// SPDX-License-Identifier: Apache-2.0
//
// This Rust SDK is inspired by the Sui TypeScript SDK and developed independently by Jarry Han (styu12).

use std::sync::Arc;
use sui_sdk::SuiClient;
use crate::DeepBookConfig;

pub struct FlashLoanContract {
    client: Arc<SuiClient>,
    config: Arc<DeepBookConfig>,
}

impl FlashLoanContract {
    pub fn new(client: Arc<SuiClient>, config: Arc<DeepBookConfig>) -> Self {
        FlashLoanContract { client, config }
    }
}
