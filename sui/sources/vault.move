/*
    Copyright (c) 2026 LITEMINT LLC

    This file is part of CYBERBRAWL-CONTRACTS project.
    Licensed under the MIT License.
    Author: Fred Kyung-jin Rezeau (오경진 吳景振) <hello@kyungj.in>
*/

module cyberbrawl::vault;

use std::string::String;
use sui::coin::{Self, Coin, TreasuryCap};
use sui::event;
use sui::kiosk::{Self, Kiosk, KioskOwnerCap};
use sui::transfer_policy::{Self, TransferPolicy};
use kiosk::kiosk_lock_rule;
use kiosk::royalty_rule;
use cyberbrawl::card::{Self, Card};
use cyberbrawl::credit::CREDIT;
use cyberbrawl::ion::ION;
use cyberbrawl::shard::SHARD;

#[error(code = 1)]
const ENotAdmin: vector<u8> = b"Not the admin";
#[error(code = 2)]
const EInvalidPlayer: vector<u8> = b"Invalid player id";
#[error(code = 3)]
const EInvalidAmount: vector<u8> = b"Invalid amount";
#[error(code = 4)]
const EWrongVersion: vector<u8> = b"Vault version mismatch";
#[error(code = 5)]
const ENotTheStrongroom: vector<u8> = b"Not the vault's kiosk";

const VERSION: u64 = 1;
const PLAYER_MAX: u64 = 32;
const KIND_CREDIT: u8 = 0;
const KIND_ION: u8 = 1;
const KIND_CARD: u8 = 2;
const KIND_SHARD: u8 = 3;

public struct Vault has key {
    id: UID,
    version: u64,
    admin: address,
    credit: TreasuryCap<CREDIT>,
    ion: TreasuryCap<ION>,
    shard: TreasuryCap<SHARD>,
    credit_out: u64,
    ion_out: u64,
    shard_out: u64,
    cards_out: u64,
    strongroom: ID,
    strongroom_cap: KioskOwnerCap,
}

public struct Withdraw has copy, drop {
    player: String,
    kind: u8,
    amount: u64,
    code: String,
    to: address,
}

public struct Deposit has copy, drop {
    player: String,
    kind: u8,
    amount: u64,
    code: String,
    sender: address,
}

public fun create(credit: TreasuryCap<CREDIT>, ion: TreasuryCap<ION>, shard: TreasuryCap<SHARD>, ctx: &mut TxContext) {
    let (strongroom, strongroom_cap) = kiosk::new(ctx);
    transfer::share_object(Vault {
        id: object::new(ctx),
        version: VERSION,
        admin: ctx.sender(),
        credit,
        ion,
        shard,
        credit_out: 0,
        ion_out: 0,
        shard_out: 0,
        cards_out: 0,
        strongroom: object::id(&strongroom),
        strongroom_cap,
    });
    transfer::public_share_object(strongroom);
}

public fun set_admin(vault: &mut Vault, admin: address, ctx: &TxContext) {
    require_version(vault);
    require_admin(vault, ctx);
    vault.admin = admin;
}

public fun withdraw_credit(vault: &mut Vault, player: String, amount: u64, to: address, ctx: &mut TxContext) {
    require_version(vault);
    require_admin(vault, ctx);
    require_player(&player);
    assert!(amount > 0, EInvalidAmount);
    vault.credit_out = vault.credit_out + amount;
    transfer::public_transfer(coin::mint(&mut vault.credit, amount, ctx), to);
    event::emit(Withdraw { player, kind: KIND_CREDIT, amount, code: b"".to_string(), to });
}

public fun withdraw_ion(vault: &mut Vault, player: String, amount: u64, to: address, ctx: &mut TxContext) {
    require_version(vault);
    require_admin(vault, ctx);
    require_player(&player);
    assert!(amount > 0, EInvalidAmount);
    vault.ion_out = vault.ion_out + amount;
    transfer::public_transfer(coin::mint(&mut vault.ion, amount, ctx), to);
    event::emit(Withdraw { player, kind: KIND_ION, amount, code: b"".to_string(), to });
}

public fun withdraw_shard(vault: &mut Vault, player: String, amount: u64, to: address, ctx: &mut TxContext) {
    require_version(vault);
    require_admin(vault, ctx);
    require_player(&player);
    assert!(amount > 0, EInvalidAmount);
    vault.shard_out = vault.shard_out + amount;
    transfer::public_transfer(coin::mint(&mut vault.shard, amount, ctx), to);
    event::emit(Withdraw { player, kind: KIND_SHARD, amount, code: b"".to_string(), to });
}

public fun withdraw_card(vault: &mut Vault, player: String, code: String, name: String, description: String, image_url: String, thumbnail_url: String, tier: String, to: address, ctx: &mut TxContext) {
    require_version(vault);
    require_admin(vault, ctx);
    require_player(&player);
    vault.cards_out = vault.cards_out + 1;
    transfer::public_transfer(card::mint(code, name, description, image_url, thumbnail_url, tier, ctx), to);
    event::emit(Withdraw { player, kind: KIND_CARD, amount: 1, code, to });
}

public fun deposit_credit(vault: &mut Vault, player: String, coin: Coin<CREDIT>, ctx: &TxContext) {
    require_version(vault);
    require_player(&player);
    let amount = coin.value();
    assert!(amount > 0, EInvalidAmount);
    vault.credit_out = vault.credit_out - amount;
    coin::burn(&mut vault.credit, coin);
    event::emit(Deposit { player, kind: KIND_CREDIT, amount, code: b"".to_string(), sender: ctx.sender() });
}

public fun deposit_ion(vault: &mut Vault, player: String, coin: Coin<ION>, ctx: &TxContext) {
    require_version(vault);
    require_player(&player);
    let amount = coin.value();
    assert!(amount > 0, EInvalidAmount);
    vault.ion_out = vault.ion_out - amount;
    coin::burn(&mut vault.ion, coin);
    event::emit(Deposit { player, kind: KIND_ION, amount, code: b"".to_string(), sender: ctx.sender() });
}

public fun deposit_shard(vault: &mut Vault, player: String, coin: Coin<SHARD>, ctx: &TxContext) {
    require_version(vault);
    require_player(&player);
    let amount = coin.value();
    assert!(amount > 0, EInvalidAmount);
    vault.shard_out = vault.shard_out - amount;
    coin::burn(&mut vault.shard, coin);
    event::emit(Deposit { player, kind: KIND_SHARD, amount, code: b"".to_string(), sender: ctx.sender() });
}

public fun deposit_card(vault: &mut Vault, player: String, card: Card, ctx: &TxContext) {
    require_version(vault);
    require_player(&player);
    vault.cards_out = vault.cards_out - 1;
    let code = card::burn(card);
    event::emit(Deposit { player, kind: KIND_CARD, amount: 1, code, sender: ctx.sender() });
}

public fun deposit_card_from_kiosk(vault: &mut Vault, strongroom: &mut Kiosk, from: &mut Kiosk, item: ID, policy: &mut TransferPolicy<Card>, player: String, ctx: &mut TxContext) {
    require_version(vault);
    require_player(&player);
    assert!(object::id(strongroom) == vault.strongroom, ENotTheStrongroom);
    let (card, mut request) = kiosk::purchase<Card>(from, item, coin::zero(ctx));
    let code = *card.code();
    vault.cards_out = vault.cards_out - 1;
    kiosk::lock(strongroom, &vault.strongroom_cap, policy, card);
    if (transfer_policy::has_rule<Card, kiosk_lock_rule::Rule>(policy)) {
        kiosk_lock_rule::prove(&mut request, strongroom);
    };
    if (transfer_policy::has_rule<Card, royalty_rule::Rule>(policy)) {
        royalty_rule::pay(policy, &mut request, coin::zero(ctx));
    };
    transfer_policy::confirm_request(policy, request);
    event::emit(Deposit { player, kind: KIND_CARD, amount: 1, code, sender: ctx.sender() });
}

public fun version(vault: &Vault): u64 {
    vault.version
}

public fun admin(vault: &Vault): address {
    vault.admin
}

public fun credit_out(vault: &Vault): u64 {
    vault.credit_out
}

public fun ion_out(vault: &Vault): u64 {
    vault.ion_out
}

public fun shard_out(vault: &Vault): u64 {
    vault.shard_out
}

public fun cards_out(vault: &Vault): u64 {
    vault.cards_out
}

public fun strongroom(vault: &Vault): ID {
    vault.strongroom
}

fun require_version(vault: &Vault) {
    assert!(vault.version == VERSION, EWrongVersion);
}

fun require_admin(vault: &Vault, ctx: &TxContext) {
    assert!(ctx.sender() == vault.admin, ENotAdmin);
}

fun require_player(player: &String) {
    let length = player.length();
    assert!(length > 0 && length <= PLAYER_MAX, EInvalidPlayer);
}
