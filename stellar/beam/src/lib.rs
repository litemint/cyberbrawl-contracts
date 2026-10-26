/*
    Copyright (c) 2026 LITEMINT LLC

    This file is part of CYBERBRAWL-CONTRACTS project.
    Licensed under the MIT License.
    Author: Fred Kyung-jin Rezeau (오경진 吳景振) <hello@kyungj.in>
*/

#![no_std]

mod beam;
mod circuit;
mod storage;
mod types;
mod zk;

use soroban_sdk::{contract, contractimpl, Address, Bytes, BytesN, Env};
use types::{Error, Issuer};

#[contract]
pub struct Beam;

#[contractimpl]
impl Beam {
    pub fn __constructor(env: Env, admin: Address, ion: Address,
        attestor: BytesN<32>, issuer: Issuer) {
        beam::initialize(&env, admin, ion, attestor, issuer)
    }

    pub fn etch(env: Env, payer: Address, attestation: Bytes,
        proof: Bytes) -> Result<(), Error> {
        beam::etch(&env, payer, attestation, proof)
    }

    pub fn withdraw(env: Env, card: Address, amount: i128,
        to: Address) -> Result<(), Error> {
        beam::withdraw(&env, card, amount, to)
    }

    pub fn set_attestor(env: Env, pubkey: BytesN<32>) -> Result<(), Error> {
        beam::set_attestor(&env, pubkey)
    }

    pub fn set_issuer(env: Env, issuer: Issuer) -> Result<(), Error> {
        beam::set_issuer(&env, issuer)
    }

    pub fn upgrade(env: Env, hash: BytesN<32>) -> Result<(), Error> {
        beam::upgrade(&env, hash)
    }
}

#[cfg(test)]
mod vector;
#[cfg(test)]
mod test;
