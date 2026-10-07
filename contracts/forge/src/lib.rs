/*
    Copyright (c) 2026 LITEMINT LLC

    This file is part of CYBERBRAWL-CONTRACTS project.
    Licensed under the MIT License.
    Author: Fred Kyung-jin Rezeau (오경진 吳景振) <hello@kyungj.in>
*/

#![no_std]

mod forge;
mod storage;
mod types;

use soroban_sdk::{contract, contractimpl, Address, Bytes, BytesN, Env};
use types::{Entry, Error};

#[contract]
pub struct Forge;

#[contractimpl]
impl Forge {
    pub fn __constructor(env: Env, admin: Address, credit: Address,
        ion: Address, attestor: BytesN<32>) {
        forge::initialize(&env, admin, credit, ion, attestor)
    }

    pub fn ignite(env: Env, payer: Address, amount: i128, power: u32,
        attestation: Bytes) -> Result<Entry, Error> {
        forge::ignite(&env, payer, amount, power, attestation)
    }

    pub fn collect(env: Env, id: BytesN<16>) -> Result<i128, Error> {
        forge::collect(&env, id)
    }

    pub fn extract(env: Env, payer: Address, id: BytesN<16>,
        attestation: Bytes) -> Result<i128, Error> {
        forge::extract(&env, payer, id, attestation)
    }

    pub fn set_attestor(env: Env, pubkey: BytesN<32>) -> Result<(), Error> {
        forge::set_attestor(&env, pubkey)
    }

    pub fn upgrade(env: Env, hash: BytesN<32>) -> Result<(), Error> {
        forge::upgrade(&env, hash)
    }
}

#[cfg(test)]
mod test;
