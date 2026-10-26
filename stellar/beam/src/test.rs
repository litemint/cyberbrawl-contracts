/*
    Copyright (c) 2026 LITEMINT LLC

    This file is part of CYBERBRAWL-CONTRACTS project.
    Licensed under the MIT License.
    Author: Fred Kyung-jin Rezeau (오경진 吳景振) <hello@kyungj.in>
*/

extern crate std;
use crate::*;
use ed25519_dalek::{Signer, SigningKey};
use soroban_sdk::{contract, contractimpl, testutils::{Address as _, Ledger}, token, Address, Bytes, BytesN, Env};
use std::vec::Vec;

#[contract]
struct Wallet;

#[contractimpl]
impl Wallet {}

const PRICE: u64 = 9_000_000_000; // 900 ION, in stroops.
const STOCK: i128 = 10_000; // Cards in the contract, in stroops of the asset.

struct Fixture<'a> {
    env: &'a Env,
    client: BeamClient<'a>,
    admin: Address,
    contract: Address,
    ion: Address,
    card: Address,
    receiver: Address,
    key: SigningKey,
}

fn setup(env: &Env, stock: i128) -> Fixture<'_> {
    env.mock_all_auths();
    env.ledger().set_timestamp(1_000_000);
    let admin = Address::generate(env);
    let ion = env.register_stellar_asset_contract_v2(Address::generate(env)).address();
    let card = env.register_stellar_asset_contract_v2(Address::generate(env)).address();
    let receiver = Address::from_str(env, vector::RECEIVER);
    env.register_at(&receiver, Wallet, ());
    let key = SigningKey::from_bytes(&[7u8; 32]);
    let attestor = BytesN::from_array(env, &key.verifying_key().to_bytes());
    let issuer = Issuer(BytesN::from_array(env, &vector::AX), BytesN::from_array(env, &vector::AY));
    let contract = env.register(Beam, BeamArgs::__constructor(&admin, &ion, &attestor, &issuer));
    let client = BeamClient::new(env, &contract);

    token::StellarAssetClient::new(env, &card).mint(&contract, &stock);
    token::StellarAssetClient::new(env, &ion).mint(&receiver, &(PRICE as i128 * 10));

    Fixture { env, client, admin, contract, ion, card, receiver, key }
}

impl Fixture<'_> {
    fn quote(&self, tier: u8, expiry: u64) -> Bytes {
        self.quote_to(&self.receiver, tier, expiry)
    }

    fn quote_to(&self, receiver: &Address, tier: u8, expiry: u64) -> Bytes {
        let mut card = [0u8; 56];
        self.card.to_string().copy_into_slice(&mut card);
        let mut to = [0u8; 56];
        receiver.to_string().copy_into_slice(&mut to);
        let mut body = [0u8; 129];
        body[0..56].copy_from_slice(&card);
        body[56..112].copy_from_slice(&to);
        body[112..120].copy_from_slice(&PRICE.to_be_bytes());
        body[120] = tier;
        body[121..129].copy_from_slice(&expiry.to_be_bytes());

        let mut data: Vec<u8> = Vec::from(body);
        data.extend_from_slice(&self.key.sign(&body).to_bytes());
        Bytes::from_slice(self.env, &data)
    }

    fn proof(&self) -> Bytes {
        Bytes::from_slice(self.env, &vector::PROOF)
    }

    fn valid(&self) -> u64 {
        self.env.ledger().timestamp() + 1_800
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
fn test_address_field() {
    // The field the circuit script computed in JS is the one the contract computes.
    let env = Env::default();
    let receiver = Address::from_str(&env, vector::RECEIVER);
    let field = zk::address_to_field(&env, &receiver);
    assert_eq!(field.to_be_bytes(), Bytes::from_slice(&env, &vector::RECEIVER_FIELD));
}

#[test]
fn test_gate_is_the_proven_gate() {
    // Tier 1 asks what the vector proves: the Ironclad, at least 1.
    let gate = types::BeamParams::default().gates[0];
    assert_eq!(gate.fact, vector::FACT);
    assert_eq!(gate.threshold, vector::THRESHOLD);
}

#[test]
fn test_open_order() {
    let env = Env::default();
    let f = setup(&env, STOCK);
    f.client.etch(&f.receiver, &f.quote(0, f.valid()), &Bytes::new(&env));

    assert_eq!(f.balance(&f.card, &f.receiver), 1);
    assert_eq!(f.balance(&f.card, &f.contract), STOCK - 1);
    assert_eq!(f.balance(&f.ion, &f.receiver), PRICE as i128 * 9);
}

#[test]
fn test_gated_order_with_a_proof() {
    let env = Env::default();
    let f = setup(&env, STOCK);
    f.client.etch(&f.receiver, &f.quote(1, f.valid()), &f.proof());
    std::println!("gated order, cpu instructions: {}", env.cost_estimate().budget().cpu_instruction_cost());

    assert_eq!(f.balance(&f.card, &f.receiver), 1);
    assert_eq!(f.balance(&f.ion, &f.receiver), PRICE as i128 * 9);
}

#[test]
fn test_gated_order_without_a_proof() {
    let env = Env::default();
    let f = setup(&env, STOCK);
    assert_eq!(code(f.client.try_etch(&f.receiver, &f.quote(1, f.valid()), &Bytes::new(&env))), 3);

    // The right length, but not a proof: the host rejects a point off the curve, or the pairing says no.
    let mut forged = vector::PROOF;
    forged[0] ^= 1;
    assert!(f.client.try_etch(&f.receiver, &f.quote(1, f.valid()), &Bytes::from_slice(&env, &forged)).is_err());
    assert_eq!(f.balance(&f.card, &f.receiver), 0);
}

#[test]
fn test_proof_is_for_one_receiver() {
    // The proof was made for the wallet it names: another wallet cannot use it.
    let env = Env::default();
    let f = setup(&env, STOCK);
    let other = Address::generate(&env);
    token::StellarAssetClient::new(&env, &f.ion).mint(&other, &(PRICE as i128));
    assert_eq!(code(f.client.try_etch(&other, &f.quote_to(&other, 1, f.valid()), &f.proof())), 3);
    assert_eq!(f.balance(&f.card, &other), 0);
}

#[test]
fn test_proof_is_for_one_issuer() {
    // A new Engram key: Engrams signed under the old one stop proving.
    let env = Env::default();
    let f = setup(&env, STOCK);
    let mut ax = vector::AX;
    ax[31] ^= 1;
    f.client.set_issuer(&Issuer(BytesN::from_array(&env, &ax), BytesN::from_array(&env, &vector::AY)));
    assert_eq!(code(f.client.try_etch(&f.receiver, &f.quote(1, f.valid()), &f.proof())), 3);
}

#[test]
fn test_expired_quote() {
    let env = Env::default();
    let f = setup(&env, STOCK);
    let quote = f.quote(0, f.valid());
    env.ledger().set_timestamp(f.valid() + 1);
    assert_eq!(code(f.client.try_etch(&f.receiver, &quote, &Bytes::new(&env))), 1);
}

#[test]
fn test_invalid_attestation() {
    let env = Env::default();
    let f = setup(&env, STOCK);
    assert_eq!(code(f.client.try_etch(&f.receiver, &Bytes::from_slice(&env, &[0u8; 10]), &Bytes::new(&env))), 2);

    // A tier the contract does not know.
    assert_eq!(code(f.client.try_etch(&f.receiver, &f.quote(2, f.valid()), &f.proof())), 2);
}

#[test]
fn test_no_stock() {
    let env = Env::default();
    let f = setup(&env, 0);
    assert_eq!(code(f.client.try_etch(&f.receiver, &f.quote(0, f.valid()), &Bytes::new(&env))), 4);
    assert_eq!(f.balance(&f.ion, &f.receiver), PRICE as i128 * 10);
}

#[test]
fn test_a_quote_is_a_price_not_a_ticket() {
    let env = Env::default();
    let f = setup(&env, STOCK);
    let quote = f.quote(0, f.valid());
    f.client.etch(&f.receiver, &quote, &Bytes::new(&env));
    f.client.etch(&f.receiver, &quote, &Bytes::new(&env));
    assert_eq!(f.balance(&f.card, &f.receiver), 2);
    assert_eq!(f.balance(&f.ion, &f.receiver), PRICE as i128 * 8);
}

#[test]
fn test_order_to_a_wallet_the_payer_is_not() {
    // The payer burns, the receiver gets the card.
    let env = Env::default();
    let f = setup(&env, STOCK);
    let payer = Address::generate(&env);
    token::StellarAssetClient::new(&env, &f.ion).mint(&payer, &(PRICE as i128));
    f.client.etch(&payer, &f.quote(0, f.valid()), &Bytes::new(&env));
    assert_eq!(f.balance(&f.card, &f.receiver), 1);
    assert_eq!(f.balance(&f.ion, &payer), 0);
}

#[test]
fn test_set_attestor() {
    let env = Env::default();
    let mut f = setup(&env, STOCK);
    let rotated = SigningKey::from_bytes(&[9u8; 32]);
    f.client.set_attestor(&BytesN::from_array(&env, &rotated.verifying_key().to_bytes()));
    f.key = rotated;
    f.client.etch(&f.receiver, &f.quote(0, f.valid()), &Bytes::new(&env));
    assert_eq!(f.balance(&f.card, &f.receiver), 1);
}

#[test]
fn test_withdraw() {
    let env = Env::default();
    let f = setup(&env, STOCK);
    let to = Address::generate(&env);
    f.client.withdraw(&f.card, &4_000, &to);
    assert_eq!(f.balance(&f.card, &to), 4_000);
    assert_eq!(f.balance(&f.card, &f.contract), STOCK - 4_000);
    assert_eq!(f.admin, f.admin);
}
