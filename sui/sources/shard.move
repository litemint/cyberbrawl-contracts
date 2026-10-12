/*
    Copyright (c) 2026 LITEMINT LLC

    This file is part of CYBERBRAWL-CONTRACTS project.
    Licensed under the MIT License.
    Author: Fred Kyung-jin Rezeau (오경진 吳景振) <hello@kyungj.in>
*/

module cyberbrawl::shard;

use sui::coin_registry;

public struct SHARD has drop {}

/// Seven decimals to match Stellar. Amount crosses chains 1:1.
const DECIMALS: u8 = 7;

fun init(witness: SHARD, ctx: &mut TxContext) {
    let (currency, treasury) = coin_registry::new_currency_with_otw(
        witness,
        DECIMALS,
        b"SHARD".to_string(),
        b"SHARD".to_string(),
        b"SHARD is the premium in-game currency for Cyberbrawl.".to_string(),
        b"https://static.cyberbrawl.io/shard_logo_512.png".to_string(),
        ctx,
    );
    transfer::public_transfer(currency.finalize(ctx), ctx.sender());
    transfer::public_transfer(treasury, ctx.sender());
}

#[test_only]
public fun init_for_testing(ctx: &mut TxContext) {
    init(SHARD {}, ctx);
}
