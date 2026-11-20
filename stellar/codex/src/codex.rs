/*
    Copyright (c) 2026 LITEMINT LLC

    This file is part of CYBERBRAWL-CONTRACTS project.
    Licensed under the MIT License.
    Author: Fred Kyung-jin Rezeau (오경진 吳景振) <hello@kyungj.in>
*/

use crate::{storage, types::Record};
use soroban_sdk::{Address, Bytes, BytesN, ContractExecutable, Env};

pub fn initialize(env: &Env, admin: Address) {
    admin.require_auth();

    storage::set_admin(env, &admin);
    storage::extend_ttl(env);
}

// The boards stay in the transaction.
pub fn inscribe(env: &Env, day: u32, _boards: Bytes) {
    let scribe = storage::get_scribe(env);
    scribe.require_auth();

    Record { day }.publish(env);
    storage::extend_ttl(env);
}

pub fn set_scribe(env: &Env, scribe: Address) {
    let admin = storage::get_admin(env);
    admin.require_auth();

    storage::set_scribe(env, &scribe);
    storage::extend_ttl(env);
}

pub fn upgrade(env: &Env, hash: BytesN<32>) {
    let admin = storage::get_admin(env);
    admin.require_auth();

    env.deployer().update_current_contract(ContractExecutable::Wasm(hash));
    storage::extend_ttl(env);
}
