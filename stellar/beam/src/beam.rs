/*
    Copyright (c) 2026 LITEMINT LLC

    This file is part of CYBERBRAWL-CONTRACTS project.
    Licensed under the MIT License.
    Author: Fred Kyung-jin Rezeau (오경진 吳景振) <hello@kyungj.in>
*/

use crate::{circuit, storage, types::{BeamParams, Error, Issuer, Quote}, zk};
use soroban_sdk::{token, Address, Bytes, BytesN, ContractExecutable, Env};

// Attestation: [card:56][receiver:56][price:8][tier:1][expiry:8].
const ADDRESS_LEN: u32 = 56;
const BODY_LEN: u32 = ADDRESS_LEN + ADDRESS_LEN + 8 + 1 + 8;
const SIGNATURE_LEN: u32 = 64;

const PARAMS: BeamParams = BeamParams::default();

const CARD: i128 = 1; // Stroop.

fn u64_at(data: &Bytes, at: u32) -> u64 {
    let bytes: BytesN<8> = data.slice(at..at + 8).try_into().unwrap();
    u64::from_be_bytes(bytes.to_array())
}

fn verify_attestation(env: &Env, data: &Bytes) -> Result<Quote, Error> {
    if data.len() != BODY_LEN + SIGNATURE_LEN {
        return Err(Error::InvalidAttestation);
    }

    let signature: BytesN<64> = data.slice(BODY_LEN..BODY_LEN + SIGNATURE_LEN).try_into().unwrap();
    env.crypto().ed25519_verify(&storage::get_attestor(env), &data.slice(0..BODY_LEN), &signature);

    let card = Address::from_string_bytes(&data.slice(0..ADDRESS_LEN));
    let receiver = Address::from_string_bytes(&data.slice(ADDRESS_LEN..ADDRESS_LEN * 2));
    let price = u64_at(data, ADDRESS_LEN * 2) as i128;
    let tier = data.get(ADDRESS_LEN * 2 + 8).unwrap() as u32;
    let expiry = u64_at(data, ADDRESS_LEN * 2 + 9);

    Ok(Quote { card, receiver, price, tier, expiry })
}

pub fn initialize(env: &Env, admin: Address, ion: Address,
    attestor: BytesN<32>, issuer: Issuer) {
    admin.require_auth();

    storage::set_admin(env, &admin);
    storage::set_ion(env, &ion);
    storage::set_attestor(env, &attestor);
    storage::set_issuer(env, &issuer);
    storage::extend_ttl(env);
}

pub fn etch(env: &Env, payer: Address, attestation: Bytes, proof: Bytes) -> Result<(), Error> {
    payer.require_auth();

    let quote = verify_attestation(env, &attestation)?;
    if quote.expiry < env.ledger().timestamp() {
        return Err(Error::Expired);
    }

    // A tier above 0 is a gate: the proof that the Engram holds the fact the tier asks for.
    if quote.tier > 0 {
        let gate = PARAMS.gates.get(quote.tier as usize - 1).ok_or(Error::InvalidAttestation)?;
        let (raw, nullifier) = circuit::extract(env, &proof).ok_or(Error::InvalidProof)?;
        let inputs = circuit::inputs(env, &storage::get_issuer(env), gate.fact, gate.threshold,
            &quote.receiver, nullifier);
        if !zk::verify_groth16(env, &circuit::KEYS, &raw, &inputs) {
            return Err(Error::InvalidProof);
        }
    }

    let card = token::Client::new(env, &quote.card);
    if card.balance(&env.current_contract_address()) < CARD {
        return Err(Error::NoStock);
    }
    token::Client::new(env, &storage::get_ion(env)).burn(&payer, &quote.price);
    card.transfer(&env.current_contract_address(), &quote.receiver, &CARD);
    storage::extend_ttl(env);

    Ok(())
}

pub fn withdraw(env: &Env, card: Address, amount: i128, to: Address) -> Result<(), Error> {
    let admin = storage::get_admin(env);
    admin.require_auth();

    token::Client::new(env, &card).transfer(&env.current_contract_address(), &to, &amount);
    storage::extend_ttl(env);

    Ok(())
}

pub fn set_attestor(env: &Env, pubkey: BytesN<32>) -> Result<(), Error> {
    let admin = storage::get_admin(env);
    admin.require_auth();

    storage::set_attestor(env, &pubkey);
    storage::extend_ttl(env);

    Ok(())
}

pub fn set_issuer(env: &Env, issuer: Issuer) -> Result<(), Error> {
    let admin = storage::get_admin(env);
    admin.require_auth();

    storage::set_issuer(env, &issuer);
    storage::extend_ttl(env);

    Ok(())
}

pub fn upgrade(env: &Env, hash: BytesN<32>) -> Result<(), Error> {
    let admin = storage::get_admin(env);
    admin.require_auth();

    env.deployer().update_current_contract(ContractExecutable::Wasm(hash));
    storage::extend_ttl(env);

    Ok(())
}
