/*
    Copyright (c) 2026 LITEMINT LLC

    This file is part of CYBERBRAWL-CONTRACTS project.
    Licensed under the MIT License.
    Author: Fred Kyung-jin Rezeau (오경진 吳景振) <hello@kyungj.in>
*/

use crate::types::{Issuer, Storage};
use soroban_sdk::{Address, BytesN, Env};

pub fn get_admin(env: &Env) -> Address {
    env.storage().instance()
        .get::<Storage, Address>(&Storage::Admin)
        .unwrap_or_else(|| panic!("admin not set"))
}

pub fn set_admin(env: &Env, admin: &Address) {
    env.storage().instance()
        .set::<Storage, Address>(&Storage::Admin, admin);
}

pub fn get_ion(env: &Env) -> Address {
    env.storage().instance()
        .get::<Storage, Address>(&Storage::Ion)
        .unwrap_or_else(|| panic!("ion not set"))
}

pub fn set_ion(env: &Env, ion: &Address) {
    env.storage().instance()
        .set::<Storage, Address>(&Storage::Ion, ion);
}

pub fn get_attestor(env: &Env) -> BytesN<32> {
    env.storage().instance()
        .get::<Storage, BytesN<32>>(&Storage::Attestor)
        .unwrap_or_else(|| panic!("attestor not set"))
}

pub fn set_attestor(env: &Env, pubkey: &BytesN<32>) {
    env.storage().instance()
        .set::<Storage, BytesN<32>>(&Storage::Attestor, pubkey);
}

pub fn get_issuer(env: &Env) -> Issuer {
    env.storage().instance()
        .get::<Storage, Issuer>(&Storage::Issuer)
        .unwrap_or_else(|| panic!("issuer not set"))
}

pub fn set_issuer(env: &Env, issuer: &Issuer) {
    env.storage().instance()
        .set::<Storage, Issuer>(&Storage::Issuer, issuer);
}

pub fn extend_ttl(env: &Env) {
    let max_ttl = env.storage().max_ttl();
    let threshold = max_ttl.saturating_sub(120_960);
    env.storage().instance().extend_ttl(threshold, max_ttl);
}
