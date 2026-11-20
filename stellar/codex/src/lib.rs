/*
    Copyright (c) 2026 LITEMINT LLC

    This file is part of CYBERBRAWL-CONTRACTS project.
    Licensed under the MIT License.
    Author: Fred Kyung-jin Rezeau (오경진 吳景振) <hello@kyungj.in>
*/

#![no_std]

mod codex;
mod storage;
mod types;

use soroban_sdk::{contract, contractimpl, Address, Bytes, BytesN, Env};

#[contract]
pub struct Codex;

#[contractimpl]
impl Codex {
    pub fn __constructor(env: Env, admin: Address) {
        codex::initialize(&env, admin)
    }

    pub fn inscribe(env: Env, day: u32, boards: Bytes) {
        codex::inscribe(&env, day, boards)
    }

    pub fn set_scribe(env: Env, scribe: Address) {
        codex::set_scribe(&env, scribe)
    }

    pub fn upgrade(env: Env, hash: BytesN<32>) {
        codex::upgrade(&env, hash)
    }
}

#[cfg(test)]
mod test;
