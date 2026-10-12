/*
    Copyright (c) 2026 LITEMINT LLC

    This file is part of CYBERBRAWL-CONTRACTS project.
    Licensed under the MIT License.
    Author: Fred Kyung-jin Rezeau (오경진 吳景振) <hello@kyungj.in>
*/

extern crate std;
use crate::*;
use ed25519_dalek::{Signer, SigningKey};
use soroban_sdk::{testutils::{Address as _, Ledger}, token, Address, Bytes, BytesN, Env};
use std::{println, vec::Vec};

const BASE: u64 = 242_400; // 24.24 CREDIT per ION, in basis points.
const ID: [u8; 16] = [1u8; 16];

struct Fixture<'a> {
    env: &'a Env,
    client: ForgeClient<'a>,
    admin: Address,
    contract: Address,
    credit: Address,
    ion: Address,
    receiver: Address,
    key: SigningKey,
}

fn setup(env: &Env, stock: i128) -> Fixture<'_> {
    env.mock_all_auths();
    env.ledger().set_timestamp(1_000_000);
    let admin = Address::generate(env);
    let credit = env.register_stellar_asset_contract_v2(Address::generate(env)).address();
    let ion = env.register_stellar_asset_contract_v2(Address::generate(env)).address();
    let receiver = Address::generate(env);
    let key = SigningKey::from_bytes(&[7u8; 32]);
    let attestor = BytesN::from_array(env, &key.verifying_key().to_bytes());
    let contract = env.register(Forge, ForgeArgs::__constructor(&admin, &credit, &ion, &attestor));
    let client = ForgeClient::new(env, &contract);

    token::StellarAssetClient::new(env, &ion).mint(&contract, &stock);

    Fixture { env, client, admin, contract, credit, ion, receiver, key }
}

impl Fixture<'_> {
    fn quote(&self, id: [u8; 16], base: u64, expiry: u64) -> Bytes {
        self.quote_to(&self.receiver, id, base, expiry)
    }

    fn quote_to(&self, receiver: &Address, id: [u8; 16], base: u64, expiry: u64) -> Bytes {
        let mut strkey = [0u8; 56];
        receiver.to_string().copy_into_slice(&mut strkey);
        let mut body = [0u8; 88];
        body[0..16].copy_from_slice(&id);
        body[16..72].copy_from_slice(&strkey);
        body[72..80].copy_from_slice(&base.to_be_bytes());
        body[80..88].copy_from_slice(&expiry.to_be_bytes());

        let mut data: Vec<u8> = Vec::from(body);
        data.extend_from_slice(&self.key.sign(&body).to_bytes());
        Bytes::from_slice(self.env, &data)
    }

    fn payer(&self, credit: i128) -> Address {
        let payer = Address::generate(self.env);
        token::StellarAssetClient::new(self.env, &self.credit).mint(&payer, &credit);
        payer
    }

    fn valid(&self) -> u64 {
        self.env.ledger().timestamp() + 60
    }

    fn balance(&self, asset: &Address, who: &Address) -> i128 {
        token::Client::new(self.env, asset).balance(who)
    }
}

fn code<T, C, E>(result: Result<Result<T, C>, Result<Error, E>>) -> u32 {
    match result {
        Err(Ok(error)) => error as u32,
        _ => panic!("expected a contract error"),
    }
}

#[test]
fn test_set_attestor() {
    let env = Env::default();
    let mut f = setup(&env, 10_000_000_000);
    let payer = f.payer(100_000_000_000);
    let rotated = SigningKey::from_bytes(&[9u8; 32]);

    f.client.set_attestor(&BytesN::from_array(&env, &rotated.verifying_key().to_bytes()));

    // Admin auth.
    assert_eq!(env.auths().len(), 1);
    assert_eq!(env.auths()[0].0, f.admin);

    // The old key is dead, the new one works.
    assert!(f.client.try_ignite(&payer, &10_000_000, &1, &f.quote(ID, BASE, f.valid())).is_err());
    f.key = rotated;
    f.client.ignite(&payer, &10_000_000, &1, &f.quote(ID, BASE, f.valid()));
}

#[test]
fn test_ignite_collect() {
    let env = Env::default();
    let f = setup(&env, 10_000_000_000);
    let payer = f.payer(100_000_000_000);
    let id = BytesN::from_array(&env, &ID);

    let entry = f.client.ignite(&payer, &100_000_000, &1, &f.quote(ID, BASE, f.valid()));

    // Payer auth.
    assert_eq!(env.auths()[0].0, payer);

    // 10 ION at power 1, locked for two days, for the receiver.
    assert_eq!(entry.2, 1_000_000 + 172_800);
    assert_eq!(entry.0, f.receiver);
    assert_eq!(entry.1, 100_000_000);
    assert_eq!(entry.3, 1);
    assert_eq!(f.balance(&f.credit, &payer), 100_000_000_000 - 2_424_000_000);

    // Too early.
    assert_eq!(code(f.client.try_collect(&id)), 5);
    assert_eq!(f.balance(&f.ion, &f.receiver), 0);

    // Public, no auth, paid to the receiver and not to the payer.
    env.ledger().set_timestamp(entry.2);
    assert_eq!(f.client.collect(&id), 100_000_000);
    assert_eq!(env.auths().len(), 0);
    assert_eq!(f.balance(&f.ion, &f.receiver), 100_000_000);
    assert_eq!(f.balance(&f.ion, &payer), 0);
    assert_eq!(f.balance(&f.ion, &f.contract), 9_900_000_000);

    // Collected once, the id is free again.
    assert_eq!(code(f.client.try_collect(&id)), 4);
    f.client.ignite(&payer, &100_000_000, &1, &f.quote(ID, BASE, f.valid()));
}

#[test]
fn test_ignite_premium() {
    // (power, cost, seconds) for 300 ION at a base of 24.24 CREDIT per ION, the seconds never depend on the amount.
    const REF: [(u32, i128, u64); 24] = [
        ( 1,    72720000000, 172800),
        ( 2,    92412576000,  86400),
        ( 3,   106316640000,  57600),
        ( 4,   117428256000,  43200),
        ( 5,   126845496000,  34560),
        ( 6,   135099216000,  28800),
        ( 7,   142494840000,  24685),
        ( 8,   149221440000,  21600),
        ( 9,   155424456000,  19200),
        (10,   161191152000,  17280),
        (11,   166586976000,  15709),
        (12,   171677376000,  14400),
        (13,   176491440000,  13292),
        (14,   181072800000,  12342),
        (15,   185443272000,  11520),
        (16,   189624672000,  10800),
        (17,   193646088000,  10164),
        (18,   197507520000,   9600),
        (19,   201238056000,   9094),
        (20,   204837696000,   8640),
        (21,   208320984000,   8228),
        (22,   211695192000,   7854),
        (23,   214974864000,   7513),
        (24,   218160000000,   7200),
    ];
    let env = Env::default();
    let f = setup(&env, 100_000_000_000);
    let payer = f.payer(10_000_000_000_000);

    for (power, cost, seconds) in REF.iter() {
        let before = f.balance(&f.credit, &payer);
        let entry = f.client.ignite(&payer, &3_000_000_000, power, &f.quote([*power as u8; 16], BASE, f.valid()));

        assert_eq!(before - f.balance(&f.credit, &payer), *cost, "cost mismatch at power {}", power);
        assert_eq!(entry.2, 1_000_000 + seconds, "time mismatch at power {}", power);
    }
}

#[test]
fn test_ignite_forge_busy() {
    let env = Env::default();
    let f = setup(&env, 10_000_000_000);
    let payer = f.payer(100_000_000_000);
    let other = f.payer(100_000_000_000);

    f.client.ignite(&payer, &100_000_000, &1, &f.quote(ID, BASE, f.valid()));

    // Single task, whoever pays.
    assert_eq!(code(f.client.try_ignite(&payer, &100_000_000, &1, &f.quote(ID, BASE, f.valid()))), 3);
    assert_eq!(code(f.client.try_ignite(&other, &100_000_000, &1, &f.quote(ID, BASE, f.valid()))), 3);
    f.client.ignite(&other, &100_000_000, &1, &f.quote([2u8; 16], BASE, f.valid()));
}

#[test]
fn test_ignite_attestation() {
    let env = Env::default();
    let f = setup(&env, 10_000_000_000);
    let payer = f.payer(10_000_000_000_000);
    let now = env.ledger().timestamp();

    // Expired, then valid at the last second.
    assert_eq!(code(f.client.try_ignite(&payer, &10_000_000, &1, &f.quote([1u8; 16], BASE, now - 1))), 1);
    f.client.ignite(&payer, &10_000_000, &1, &f.quote([1u8; 16], BASE, now));

    // Length.
    let mut short = f.quote([4u8; 16], BASE, f.valid());
    short.pop_back();
    assert_eq!(code(f.client.try_ignite(&payer, &10_000_000, &1, &short)), 6);
    let mut long = f.quote([4u8; 16], BASE, f.valid());
    long.push_back(0);
    assert_eq!(code(f.client.try_ignite(&payer, &10_000_000, &1, &long)), 6);
}

#[test]
fn test_ignite_expiry() {
    let env = Env::default();
    let f = setup(&env, 10_000_000_000);
    let payer = f.payer(100_000_000_000);
    let now = env.ledger().timestamp();

    // Before the end of the forge: 7,200 seconds at power 24, 172,800 at power 1.
    assert_eq!(code(f.client.try_ignite(&payer, &10_000_000, &24, &f.quote([1u8; 16], BASE, now + 7_200))), 6);
    let entry = f.client.ignite(&payer, &10_000_000, &24, &f.quote([1u8; 16], BASE, now + 7_199));
    assert_eq!(code(f.client.try_ignite(&payer, &10_000_000, &1, &f.quote([2u8; 16], BASE, now + 172_800))), 6);
    f.client.ignite(&payer, &10_000_000, &1, &f.quote([2u8; 16], BASE, now + 172_799));

    // Dead by the time it can be collected, no second use.
    env.ledger().set_timestamp(entry.2);
    f.client.collect(&BytesN::from_array(&env, &[1u8; 16]));
    assert_eq!(code(f.client.try_ignite(&payer, &10_000_000, &24, &f.quote([1u8; 16], BASE, now + 7_199))), 1);
}

#[test]
#[should_panic]
fn test_ignite_bad_signature() {
    let env = Env::default();
    let f = setup(&env, 10_000_000_000);
    let payer = f.payer(100_000_000_000);
    let mut attestation = f.quote(ID, BASE, f.valid());

    // Tampered base.
    attestation.set(79, attestation.get(79).unwrap() ^ 1);
    f.client.ignite(&payer, &100_000_000, &1, &attestation);
}

#[test]
fn test_ignite_amounts() {
    let env = Env::default();
    let f = setup(&env, 10_000_000_000);
    let payer = f.payer(100_000_000_000);

    assert_eq!(code(f.client.try_ignite(&payer, &0, &1, &f.quote(ID, BASE, f.valid()))), 2);
    assert_eq!(code(f.client.try_ignite(&payer, &-1, &1, &f.quote(ID, BASE, f.valid()))), 2);
    assert_eq!(code(f.client.try_ignite(&payer, &100_000_000, &0, &f.quote(ID, BASE, f.valid()))), 2);
    assert_eq!(code(f.client.try_ignite(&payer, &100_000_000, &25, &f.quote(ID, BASE, f.valid()))), 2);
    assert_eq!(code(f.client.try_ignite(&payer, &(u64::MAX as i128 + 1), &1, &f.quote(ID, BASE, f.valid()))), 2);
}

#[test]
fn test_collect_refill() {
    let env = Env::default();
    let f = setup(&env, 50_000_000);
    let payer = f.payer(100_000_000_000);
    let id = BytesN::from_array(&env, &ID);

    // Ignite is never tied to the stock, 10 ION ordered against 5 in stock.
    let entry = f.client.ignite(&payer, &100_000_000, &1, &f.quote(ID, BASE, f.valid()));

    // Collect fails, the entry stays.
    env.ledger().set_timestamp(entry.2);
    assert!(f.client.try_collect(&id).is_err());
    assert_eq!(f.balance(&f.ion, &f.receiver), 0);

    // A refill, and the same collect goes through.
    token::StellarAssetClient::new(&env, &f.ion).mint(&f.contract, &50_000_000);
    assert_eq!(f.client.collect(&id), 100_000_000);
    assert_eq!(f.balance(&f.ion, &f.receiver), 100_000_000);
}

#[test]
fn test_collect_guards() {
    let env = Env::default();
    let f = setup(&env, 10_000_000_000);
    let payer = f.payer(100_000_000_000);
    let id = BytesN::from_array(&env, &ID);

    assert_eq!(code(f.client.try_collect(&id)), 4);
    let entry = f.client.ignite(&payer, &100_000_000, &1, &f.quote(ID, BASE, f.valid()));

    // Too early, then anyone, always the receiver.
    assert_eq!(code(f.client.try_collect(&id)), 5);
    env.ledger().set_timestamp(entry.2);
    assert_eq!(f.client.collect(&id), 100_000_000);
    assert_eq!(f.balance(&f.ion, &f.receiver), 100_000_000);
}

// Cost of an ignite, from the premium table: amount x base x premium / 10,000^2.
fn ignite_cost(amount: i128, base: u64, power: usize) -> i128 {
    amount * base as i128 * crate::types::ForgeParams::default().premium_bps[power - 1] as i128 / 100_000_000
}

#[test]
fn test_extract() {
    let env = Env::default();
    let f = setup(&env, 10_000_000_000);
    let payer = f.payer(100_000_000_000);
    let id = BytesN::from_array(&env, &ID);

    // 10 ION at power 1: 242.4 CREDIT, two days.
    let entry = f.client.ignite(&payer, &100_000_000, &1, &f.quote(ID, BASE, f.valid()));
    let after_ignite = f.balance(&f.credit, &payer);

    // A day in, the forge that finishes now is power 2: the premium step from 1 to 2, 0.2708 x 242.4 = 65.64 CREDIT.
    env.ledger().set_timestamp(entry.2 - 86_400);
    assert_eq!(f.client.extract(&payer, &id, &f.quote(ID, BASE, f.valid())), 100_000_000);
    assert_eq!(env.auths()[0].0, payer);
    assert_eq!(after_ignite - f.balance(&f.credit, &payer), 656_419_200);
    assert_eq!(f.balance(&f.ion, &f.receiver), 100_000_000);
    assert_eq!(f.balance(&f.ion, &f.contract), 9_900_000_000);

    // Start plus extraction is the upfront price of power 2.
    assert_eq!(100_000_000_000 - f.balance(&f.credit, &payer), ignite_cost(100_000_000, BASE, 2));

    // Gone: nothing to collect, the id is free, the attestation is dead with the entry.
    assert_eq!(code(f.client.try_collect(&id)), 4);
    assert_eq!(code(f.client.try_extract(&payer, &id, &f.quote(ID, BASE, f.valid()))), 4);
    f.client.ignite(&payer, &100_000_000, &1, &f.quote(ID, BASE, f.valid()));
}

#[test]
fn test_extract_prices() {
    // (power paid, seconds elapsed, power that finishes now): start plus extraction equals the upfront price of
    // that power, exactly, for 300 ION at 24.24.
    const CASES: [(u32, u64, usize); 10] = [
        ( 1, 172_799,  2), // A second short of done still rounds up to the next step.
        ( 1,  86_400,  2),
        ( 1,  72_000,  3), // 2.4 rounds up to 3.
        ( 1,  43_200,  4),
        ( 1,  21_600,  8),
        ( 1,   7_200, 24), // Two hours in, the ceiling.
        ( 2,  43_200,  4),
        ( 2,  10_000, 18), // 17.28 rounds up to 18.
        (12,   7_200, 24),
        (23,   7_200, 24),
    ];
    let env = Env::default();
    let f = setup(&env, 100_000_000_000);
    let payer = f.payer(10_000_000_000_000);
    let amount: i128 = 3_000_000_000;

    for (i, (power, elapsed, now_power)) in CASES.iter().enumerate() {
        let id = [i as u8 + 1; 16];
        let key = BytesN::from_array(&env, &id);
        let start = env.ledger().timestamp();
        let before = f.balance(&f.credit, &payer);
        f.client.ignite(&payer, &amount, power, &f.quote(id, BASE, f.valid()));

        env.ledger().set_timestamp(start + elapsed);
        f.client.extract(&payer, &key, &f.quote(id, BASE, f.valid()));
        let paid = before - f.balance(&f.credit, &payer);
        assert_eq!(paid, ignite_cost(amount, BASE, *now_power), "power {} after {} s should price as power {}", power, elapsed, now_power);
        assert!(paid > ignite_cost(amount, BASE, *power as usize), "an extraction always costs something");
        env.ledger().set_timestamp(start);
    }

    // The base of the minute is the attested one: half the base, half the step.
    let entry = f.client.ignite(&payer, &amount, &1, &f.quote([99u8; 16], BASE, f.valid()));
    env.ledger().set_timestamp(entry.2 - 86_400);
    let before = f.balance(&f.credit, &payer);
    f.client.extract(&payer, &BytesN::from_array(&env, &[99u8; 16]), &f.quote([99u8; 16], BASE / 2, f.valid()));
    assert_eq!(before - f.balance(&f.credit, &payer), (ignite_cost(amount, BASE, 2) - ignite_cost(amount, BASE, 1)) / 2);
}

#[test]
fn test_extract_too_soon() {
    let env = Env::default();
    let f = setup(&env, 10_000_000_000);
    let payer = f.payer(100_000_000_000);
    let id = BytesN::from_array(&env, &ID);
    let start = env.ledger().timestamp();

    // Nothing beats power 24: under two hours in, there is no power that finishes now.
    f.client.ignite(&payer, &100_000_000, &1, &f.quote(ID, BASE, f.valid()));
    assert_eq!(code(f.client.try_extract(&payer, &id, &f.quote(ID, BASE, f.valid()))), 7);
    env.ledger().set_timestamp(start + 7_199);
    assert_eq!(code(f.client.try_extract(&payer, &id, &f.quote(ID, BASE, f.valid()))), 7);
    env.ledger().set_timestamp(start + 7_200);
    f.client.extract(&payer, &id, &f.quote(ID, BASE, f.valid()));

    // A power 24 forge can never be extracted, only collected.
    env.ledger().set_timestamp(start);
    let entry = f.client.ignite(&payer, &100_000_000, &24, &f.quote(ID, BASE, f.valid()));
    env.ledger().set_timestamp(entry.2 - 1);
    assert_eq!(code(f.client.try_extract(&payer, &id, &f.quote(ID, BASE, f.valid()))), 7);
    env.ledger().set_timestamp(entry.2);
    assert_eq!(code(f.client.try_extract(&payer, &id, &f.quote(ID, BASE, f.valid()))), 7);
    assert_eq!(f.client.collect(&id), 100_000_000);
}

#[test]
fn test_extract_guards() {
    let env = Env::default();
    let f = setup(&env, 10_000_000_000);
    let payer = f.payer(100_000_000_000);
    let id = BytesN::from_array(&env, &ID);
    let now = env.ledger().timestamp();

    // No forge for the id.
    assert_eq!(code(f.client.try_extract(&payer, &id, &f.quote(ID, BASE, f.valid()))), 4);
    let entry = f.client.ignite(&payer, &100_000_000, &1, &f.quote(ID, BASE, f.valid()));
    env.ledger().set_timestamp(entry.2 - 86_400);

    // Expired, another forge's attestation, another receiver's, wrong length.
    assert_eq!(code(f.client.try_extract(&payer, &id, &f.quote(ID, BASE, now - 1))), 1);
    assert_eq!(code(f.client.try_extract(&payer, &id, &f.quote([2u8; 16], BASE, f.valid()))), 6);
    let stranger = Address::generate(&env);
    assert_eq!(code(f.client.try_extract(&payer, &id, &f.quote_to(&stranger, ID, BASE, f.valid()))), 6);
    let mut short = f.quote(ID, BASE, f.valid());
    short.pop_back();
    assert_eq!(code(f.client.try_extract(&payer, &id, &short)), 6);
    assert_eq!(f.balance(&f.ion, &f.receiver), 0);

    // Short of CREDIT: nothing paid, the entry stays.
    let poor = f.payer(1_000_000);
    assert!(f.client.try_extract(&poor, &id, &f.quote(ID, BASE, f.valid())).is_err());
    assert_eq!(f.balance(&f.ion, &f.receiver), 0);
    assert_eq!(code(f.client.try_collect(&id)), 5);

    // Ready: a free collect, not an extraction.
    env.ledger().set_timestamp(entry.2);
    assert_eq!(code(f.client.try_extract(&payer, &id, &f.quote(ID, BASE, f.valid()))), 7);
    assert_eq!(f.client.collect(&id), 100_000_000);
}

#[test]
fn test_legacy_entry() {
    let env = Env::default();
    let f = setup(&env, 10_000_000_000);
    let payer = f.payer(100_000_000_000);
    let id = BytesN::from_array(&env, &ID);
    let ready = env.ledger().timestamp() + 172_800;

    // An entry written by v0.1.1: three fields, no power.
    env.as_contract(&f.contract, || {
        env.storage().persistent().set(&id, &(f.receiver.clone(), 100_000_000u64, ready));
    });

    // Still a running forge: busy for ignite, not extractable, collected as before.
    assert_eq!(code(f.client.try_ignite(&payer, &100_000_000, &1, &f.quote(ID, BASE, f.valid()))), 3);
    env.ledger().set_timestamp(ready - 86_400);
    assert_eq!(code(f.client.try_extract(&payer, &id, &f.quote(ID, BASE, f.valid()))), 7);
    assert_eq!(code(f.client.try_collect(&id)), 5);
    env.ledger().set_timestamp(ready);
    assert_eq!(f.client.collect(&id), 100_000_000);
    assert_eq!(f.balance(&f.ion, &f.receiver), 100_000_000);
}

#[test]
#[should_panic]
fn test_extract_bad_signature() {
    let env = Env::default();
    let f = setup(&env, 10_000_000_000);
    let payer = f.payer(100_000_000_000);
    let id = BytesN::from_array(&env, &ID);
    let entry = f.client.ignite(&payer, &100_000_000, &1, &f.quote(ID, BASE, f.valid()));
    env.ledger().set_timestamp(entry.2 - 86_400);

    // Tampered base.
    let mut attestation = f.quote(ID, BASE, f.valid());
    attestation.set(79, attestation.get(79).unwrap() ^ 1);
    f.client.extract(&payer, &id, &attestation);
}

#[test]
fn test_upgrade() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let credit = Address::generate(&env);
    let ion = Address::generate(&env);
    let attestor = BytesN::from_array(&env, &[1u8; 32]);
    let contract_id = env.register(Forge, ForgeArgs::__constructor(&admin, &credit, &ion, &attestor));
    let client = ForgeClient::new(&env, &contract_id);
    let wasm = include_bytes!("../../../target/wasm32v1-none/release/forge.wasm");
    let wasm_hash = env.deployer().upload_contract_wasm(Bytes::from_slice(&env, wasm));

    env.mock_all_auths();
    client.upgrade(&wasm_hash);

    // Admin auth, setup survives.
    assert_eq!(env.auths().len(), 1);
    assert_eq!(env.auths()[0].0, admin);
    client.set_attestor(&attestor);
}

#[test]
fn test_examples() {
    // (amount in stroops, power, base in basis points).
    const ROWS: [(i128, u32, u64); 9] = [
        (1_000_000_000, 1, 242_400),
        (2_000_000_000, 1, 242_400),
        (3_000_000_000, 1, 242_400),
        (3_000_000_000, 2, 242_400),
        (3_000_000_000, 8, 242_400),
        (3_000_000_000, 24, 242_400),
        (1_000_000_000, 24, 242_400),
        (100_000_000_000, 1, 242_400),
        (3_000_000_000, 1, 100_000),
    ];
    let env = Env::default();
    let f = setup(&env, 10_000_000_000_000_000_000);
    let payer = f.payer(100_000_000_000_000);

    // Ignite.
    println!("\n{:>15} {:>5} {:>8} {:>15} {:>9}", "amount", "power", "base", "cost", "locked");
    for (i, (amount, power, base)) in ROWS.iter().enumerate() {
        let before = f.balance(&f.credit, &payer);
        let entry = f.client.ignite(&payer, amount, power, &f.quote([i as u8 + 1; 16], *base, f.valid()));
        let seconds = entry.2 - env.ledger().timestamp();
        println!(
            "{:>15} {:>5} {:>8} {:>15} {:>9}",
            amount,
            power,
            base,
            before - f.balance(&f.credit, &payer),
            std::format!("{}h {:02}m", seconds / 3600, seconds % 3600 / 60)
        );
    }

    // Collect.
    let id = BytesN::from_array(&env, &[1u8; 16]);
    env.ledger().set_timestamp(env.ledger().timestamp() + 172_800);
    println!("\ncollect two days later: {} stroops paid to the receiver", f.client.collect(&id));
}
