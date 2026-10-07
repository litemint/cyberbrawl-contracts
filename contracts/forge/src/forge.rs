/*
    Copyright (c) 2026 LITEMINT LLC

    This file is part of CYBERBRAWL-CONTRACTS project.
    Licensed under the MIT License.
    Author: Fred Kyung-jin Rezeau (오경진 吳景振) <hello@kyungj.in>
*/

use crate::{storage, types::{Entry, Error, ForgeParams, Quote}};
use soroban_sdk::{token, Address, Bytes, BytesN, ContractExecutable, Env};

// Attestation: [id:16][receiver:56][base:8][expiry:8], base as CREDIT per ION in basis points.
const ID_LEN: u32 = 16;
const ADDRESS_LEN: u32 = 56;
const BODY_LEN: u32 = ID_LEN + ADDRESS_LEN + 8 + 8;
const SIGNATURE_LEN: u32 = 64;

const PARAMS: ForgeParams = ForgeParams::default();
const BPS: i128 = 10_000;

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

    let id: BytesN<16> = data.slice(0..ID_LEN).try_into().unwrap();
    let receiver = Address::from_string_bytes(&data.slice(ID_LEN..ID_LEN + ADDRESS_LEN));
    let base = u64_at(data, ID_LEN + ADDRESS_LEN) as i128;
    let expiry = u64_at(data, ID_LEN + ADDRESS_LEN + 8);

    Ok(Quote { id, receiver, base, expiry })
}

pub fn initialize(env: &Env, admin: Address, credit: Address,
    ion: Address, attestor: BytesN<32>) {
    admin.require_auth();

    storage::set_admin(env, &admin);
    storage::set_credit(env, &credit);
    storage::set_ion(env, &ion);
    storage::set_attestor(env, &attestor);
    storage::extend_ttl(env);
}

pub fn ignite(env: &Env, payer: Address, amount: i128, power: u32,
    attestation: Bytes) -> Result<Entry, Error> {
    payer.require_auth();

    let now = env.ledger().timestamp();
    let quote = verify_attestation(env, &attestation)?;
    if quote.expiry < now {
        return Err(Error::Expired);
    }
    if amount <= 0 || power == 0 || power > PARAMS.max_power {
        return Err(Error::InvalidAmount);
    }
    let stroops = u64::try_from(amount).map_err(|_| Error::InvalidAmount)?;

    // The forge is single task.
    if storage::has_entry(env, &quote.id) {
        return Err(Error::ForgeBusy);
    }

    let premium = PARAMS.premium_bps[power as usize - 1] as i128;
    let cost = amount * quote.base * premium / (BPS * BPS);
    let ready = now + PARAMS.duration / power as u64;

    // Never outlives the forge it opens.
    if quote.expiry >= ready {
        return Err(Error::InvalidAttestation);
    }

    token::Client::new(env, &storage::get_credit(env)).burn(&payer, &cost);

    let entry = Entry(quote.receiver, stroops, ready, power);
    storage::set_entry(env, &quote.id, &entry);
    storage::extend_ttl(env);

    Ok(entry)
}

pub fn collect(env: &Env, id: BytesN<16>) -> Result<i128, Error> {
    let Entry(receiver, stroops, ready, _) = storage::get_entry(env, &id).ok_or(Error::NoEntry)?;
    if env.ledger().timestamp() < ready {
        return Err(Error::NotReady);
    }

    let amount = stroops as i128;
    token::Client::new(env, &storage::get_ion(env))
        .transfer(&env.current_contract_address(), &receiver, &amount);

    storage::remove_entry(env, &id);
    storage::extend_ttl(env);

    Ok(amount)
}

pub fn extract(env: &Env, payer: Address, id: BytesN<16>, attestation: Bytes) -> Result<i128, Error> {
    payer.require_auth();

    let now = env.ledger().timestamp();
    let quote = verify_attestation(env, &attestation)?;
    if quote.expiry < now {
        return Err(Error::Expired);
    }
    if quote.id != id {
        return Err(Error::InvalidAttestation);
    }

    let Entry(receiver, stroops, ready, power) = storage::get_entry(env, &id).ok_or(Error::NoEntry)?;
    if quote.receiver != receiver {
        return Err(Error::InvalidAttestation);
    }
    if now >= ready || power == 0 {
        return Err(Error::InvalidAction);
    }

    let shortest = PARAMS.duration / PARAMS.max_power as u64;
    let elapsed = now - (ready - PARAMS.duration / power as u64);
    if elapsed < shortest {
        return Err(Error::InvalidAction);
    }
    let now_power = ((PARAMS.duration + elapsed - 1) / elapsed) as u32;
    if now_power <= power || now_power > PARAMS.max_power {
        return Err(Error::InvalidAction);
    }

    // Round up to the next power, nothing beats power 24.
    let amount = stroops as i128;
    let step = (PARAMS.premium_bps[now_power as usize - 1] - PARAMS.premium_bps[power as usize - 1]) as i128;
    let cost = amount * quote.base * step / (BPS * BPS);
    token::Client::new(env, &storage::get_credit(env)).burn(&payer, &cost);
    token::Client::new(env, &storage::get_ion(env))
        .transfer(&env.current_contract_address(), &receiver, &amount);

    storage::remove_entry(env, &id);
    storage::extend_ttl(env);

    Ok(amount)
}

pub fn set_attestor(env: &Env, pubkey: BytesN<32>) -> Result<(), Error> {
    let admin = storage::get_admin(env);
    admin.require_auth();

    storage::set_attestor(env, &pubkey);
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
