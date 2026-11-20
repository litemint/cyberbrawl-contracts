/*
    Copyright (c) 2026 LITEMINT LLC

    This file is part of CYBERBRAWL-CONTRACTS project.
    Licensed under the MIT License.
    Author: Fred Kyung-jin Rezeau (오경진 吳景振) <hello@kyungj.in>
*/

extern crate std;
use crate::{types::Record, *};
use soroban_sdk::{symbol_short, testutils::{Address as _, AuthorizedFunction, AuthorizedInvocation, Events as _}, Event as _, IntoVal};
use std::vec::Vec;

const DAY: u32 = 20261120;

fn setup(env: &Env) -> (CodexClient<'_>, Address, Address) {
    env.mock_all_auths();
    let admin = Address::generate(env);
    let scribe = Address::generate(env);
    let contract = env.register(Codex, CodexArgs::__constructor(&admin));
    let client = CodexClient::new(env, &contract);
    client.set_scribe(&scribe);
    (client, scribe, contract)
}

fn varint(out: &mut Vec<u8>, mut value: u64) {
    while value >= 0x80 {
        out.push(value as u8 | 0x80);
        value >>= 7;
    }
    out.push(value as u8);
}

// [version:1][board:1][count:2][entries], entry [length:1][id][score:varint].
fn boards(env: &Env, entries: u16, id: &[u8]) -> Bytes {
    let mut out: Vec<u8> = Vec::from([1u8]);
    for (board, score) in [(0u8, 99u64), (1, 2_400)] {
        out.push(board);
        out.extend_from_slice(&entries.to_be_bytes());
        for _ in 0..entries {
            out.push(id.len() as u8);
            out.extend_from_slice(id);
            varint(&mut out, score);
        }
    }
    Bytes::from_slice(env, &out)
}

#[test]
fn test_inscribe() {
    let env = Env::default();
    let (client, _, contract) = setup(&env);
    let data = boards(&env, 100, b"eu0erq");
    client.inscribe(&DAY, &data);

    assert_eq!(env.events().all(), [Record { day: DAY }.to_xdr(&env, &contract)]);
}

#[test]
fn test_inscribe_by_the_scribe() {
    let env = Env::default();
    let (client, scribe, contract) = setup(&env);
    let data = boards(&env, 1, b"eu0erq");
    client.inscribe(&DAY, &data);

    assert_eq!(env.auths(), [(scribe, AuthorizedInvocation {
        function: AuthorizedFunction::Contract((contract, symbol_short!("inscribe"), (DAY, data).into_val(&env))),
        sub_invocations: [].into(),
    })]);
}

#[test]
#[should_panic(expected = "scribe not set")]
fn test_inscribe_without_a_scribe() {
    let env = Env::default();
    env.mock_all_auths();
    let contract = env.register(Codex, CodexArgs::__constructor(&Address::generate(&env)));
    CodexClient::new(&env, &contract).inscribe(&DAY, &boards(&env, 1, b"eu0erq"));
}

#[test]
fn test_full_day() {
    // 1,000 entries, every id at its longest.
    let env = Env::default();
    let (client, _, _) = setup(&env);
    let data = boards(&env, 1_000, b"abcdefghij");
    client.inscribe(&DAY, &data);
    std::println!("full day, {} bytes, cpu instructions: {}", data.len(), env.cost_estimate().budget().cpu_instruction_cost());

    assert_eq!(data.len(), 1 + (3 + 1_000 * 12) + (3 + 1_000 * 13));
}
