/*
    Copyright (c) 2026 LITEMINT LLC

    This file is part of CYBERBRAWL-CONTRACTS project.
    Licensed under the MIT License.
    Author: Fred Kyung-jin Rezeau (오경진 吳景振) <hello@kyungj.in>
*/

use crate::types::Storage;
use soroban_sdk::{Address, Env};

pub fn get_admin(env: &Env) -> Address {
    env.storage().instance()
        .get::<Storage, Address>(&Storage::Admin)
        .unwrap_or_else(|| panic!("admin not set"))
}

pub fn set_admin(env: &Env, admin: &Address) {
    env.storage().instance()
        .set::<Storage, Address>(&Storage::Admin, admin);
}

pub fn extend_ttl(env: &Env) {
    let max_ttl = env.storage().max_ttl();
    let threshold = max_ttl.saturating_sub(120_960);
    env.storage().instance().extend_ttl(threshold, max_ttl);
}
