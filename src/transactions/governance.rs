// Copyright (c) Jarry Han (styu12)
// SPDX-License-Identifier: Apache-2.0
//
// This Rust SDK is inspired by the Sui TypeScript SDK and developed independently by Jarry Han (styu12).

use crate::DeepBookConfig;

pub struct GovernanceContract<'a> {
    config: &'a DeepBookConfig,
}

impl<'a> GovernanceContract<'a> {
    pub fn new(config: &'a DeepBookConfig) -> Self {
        GovernanceContract { config }
    }
}
