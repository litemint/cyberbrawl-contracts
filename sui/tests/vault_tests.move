/*
    Copyright (c) 2026 LITEMINT LLC

    This file is part of CYBERBRAWL-CONTRACTS project.
    Licensed under the MIT License.
    Author: Fred Kyung-jin Rezeau (오경진 吳景振) <hello@kyungj.in>
*/

#[test_only]
module cyberbrawl::vault_tests;

use sui::coin::{Coin, TreasuryCap};
use sui::kiosk::{Self, Kiosk, KioskOwnerCap};
use sui::test_scenario as ts;
use sui::transfer_policy::{Self, TransferPolicy, TransferPolicyCap};
use kiosk::kiosk_lock_rule;
use kiosk::royalty_rule;
use cyberbrawl::card::{Self, Card};
use cyberbrawl::credit::{Self, CREDIT};
use cyberbrawl::ion::{Self, ION};
use cyberbrawl::shard::{Self, SHARD};
use cyberbrawl::vault::{Self, Vault};

const ADMIN: address = @0xA;
const PLAYER: address = @0xB;
const OTHER: address = @0xC;

const ONE: u64 = 10_000_000; // Seven decimals.

fun setup(): ts::Scenario {
    let mut s = ts::begin(ADMIN);
    credit::init_for_testing(s.ctx());
    ion::init_for_testing(s.ctx());
    shard::init_for_testing(s.ctx());
    card::init_for_testing(s.ctx());
    s.next_tx(ADMIN);
    let credit_cap = s.take_from_sender<TreasuryCap<CREDIT>>();
    let ion_cap = s.take_from_sender<TreasuryCap<ION>>();
    let shard_cap = s.take_from_sender<TreasuryCap<SHARD>>();
    vault::create(credit_cap, ion_cap, shard_cap, s.ctx());
    s.next_tx(ADMIN);
    s
}

#[test]
fun credit_round_trip() {
    let mut s = setup();
    let mut vault = s.take_shared<Vault>();
    assert!(vault.admin() == ADMIN);
    assert!(vault.version() == 1);

    // Withdraw: minted to the player.
    vault::withdraw_credit(&mut vault, b"jieun2".to_string(), 5 * ONE, PLAYER, s.ctx());
    assert!(vault.credit_out() == 5 * ONE);
    s.next_tx(PLAYER);
    let coin = s.take_from_sender<Coin<CREDIT>>();
    assert!(coin.value() == 5 * ONE);

    // Deposit: burned, the id carried for the server.
    vault::deposit_credit(&mut vault, b"jieun2".to_string(), coin, s.ctx());
    assert!(vault.credit_out() == 0);
    ts::return_shared(vault);
    s.end();
}

#[test]
fun ion_round_trip() {
    let mut s = setup();
    let mut vault = s.take_shared<Vault>();
    vault::withdraw_ion(&mut vault, b"jieun2".to_string(), ONE, PLAYER, s.ctx());
    assert!(vault.ion_out() == ONE);
    s.next_tx(PLAYER);
    let coin = s.take_from_sender<Coin<ION>>();
    vault::deposit_ion(&mut vault, b"jieun2".to_string(), coin, s.ctx());
    assert!(vault.ion_out() == 0);
    ts::return_shared(vault);
    s.end();
}

#[test]
fun shard_round_trip() {
    let mut s = setup();
    let mut vault = s.take_shared<Vault>();
    vault::withdraw_shard(&mut vault, b"jieun2".to_string(), 3 * ONE, PLAYER, s.ctx());
    assert!(vault.shard_out() == 3 * ONE);
    s.next_tx(PLAYER);
    let coin = s.take_from_sender<Coin<SHARD>>();
    vault::deposit_shard(&mut vault, b"jieun2".to_string(), coin, s.ctx());
    assert!(vault.shard_out() == 0);
    ts::return_shared(vault);
    s.end();
}

#[test]
fun card_round_trip() {
    let mut s = setup();
    let mut vault = s.take_shared<Vault>();
    vault::withdraw_card(&mut vault, b"jieun2".to_string(), b"CARD00073".to_string(), b"Double Time".to_string(), b"Cyberbrawl Legendary collectible card.".to_string(), b"https://cdn.litemint.com/cards/CARD00073.mp4".to_string(), b"https://cdn.litemint.com/cards/CARD00073.png".to_string(), b"Legendary".to_string(), PLAYER, s.ctx());
    assert!(vault.cards_out() == 1);
    s.next_tx(PLAYER);
    let card = s.take_from_sender<Card>();
    assert!(card.code() == b"CARD00073".to_string());
    assert!(card.name() == b"Double Time".to_string());
    assert!(card.description() == b"Cyberbrawl Legendary collectible card.".to_string());
    assert!(card.image_url() == b"https://cdn.litemint.com/cards/CARD00073.mp4".to_string());
    assert!(card.thumbnail_url() == b"https://cdn.litemint.com/cards/CARD00073.png".to_string());
    assert!(card.attributes().get(&b"Tier".to_string()) == b"Legendary".to_string());
    // Locked in kiosks once sold.
    let policy = s.take_shared<TransferPolicy<Card>>();
    assert!(transfer_policy::has_rule<Card, kiosk_lock_rule::Rule>(&policy));
    assert!(transfer_policy::has_rule<Card, royalty_rule::Rule>(&policy));
    ts::return_shared(policy);
    vault::deposit_card(&mut vault, b"jieun2".to_string(), card, s.ctx());
    assert!(vault.cards_out() == 0);
    ts::return_shared(vault);
    s.end();
}

#[test]
fun card_comes_back_from_a_kiosk() {
    let mut s = setup();
    let mut vault = s.take_shared<Vault>();
    vault::withdraw_card(&mut vault, b"jieun2".to_string(), b"CARD00073".to_string(), b"Double Time".to_string(), b"Cyberbrawl Legendary collectible card.".to_string(), b"https://cdn.litemint.com/cards/CARD00073.mp4".to_string(), b"https://cdn.litemint.com/cards/CARD00073.png".to_string(), b"Legendary".to_string(), PLAYER, s.ctx());
    ts::return_shared(vault);

    s.next_tx(PLAYER);
    let card = s.take_from_sender<Card>();
    let item = object::id(&card);
    let (mut mine, my_cap) = kiosk::new(s.ctx());
    let mut policy = s.take_shared<TransferPolicy<Card>>();
    kiosk::lock(&mut mine, &my_cap, &policy, card);
    assert!(kiosk::is_locked(&mine, item));
    transfer::public_share_object(mine);
    transfer::public_transfer(my_cap, PLAYER);
    ts::return_shared(policy);

    // Deposit.
    s.next_tx(PLAYER);
    let mut vault = s.take_shared<Vault>();
    let mut strongroom = s.take_shared_by_id<Kiosk>(vault.strongroom());
    let mut mine = s.take_shared<Kiosk>();
    let my_cap = s.take_from_sender<KioskOwnerCap>();
    let mut policy = s.take_shared<TransferPolicy<Card>>();
    kiosk::list<Card>(&mut mine, &my_cap, item, 0);
    vault::deposit_card_from_kiosk(&mut vault, &mut strongroom, &mut mine, item, &mut policy, b"jieun2".to_string(), s.ctx());
    assert!(vault.cards_out() == 0);
    assert!(!kiosk::has_item(&mine, item));
    assert!(kiosk::has_item(&strongroom, item) && kiosk::is_locked(&strongroom, item));
    ts::return_shared(policy);
    ts::return_shared(mine);
    ts::return_shared(strongroom);
    ts::return_shared(vault);
    s.return_to_sender(my_cap);
    s.end();
}

#[test]
fun a_deposit_pays_no_royalty() {
    let mut s = setup();
    // 5% royalty.
    let mut policy = s.take_shared<TransferPolicy<Card>>();
    let mut vault = s.take_shared<Vault>();
    vault::withdraw_card(&mut vault, b"jieun2".to_string(), b"CARD00073".to_string(), b"Double Time".to_string(), b"d".to_string(), b"u".to_string(), b"t".to_string(), b"Legendary".to_string(), PLAYER, s.ctx());
    ts::return_shared(vault);
    ts::return_shared(policy);

    s.next_tx(PLAYER);
    let card = s.take_from_sender<Card>();
    let item = object::id(&card);
    let (mut mine, my_cap) = kiosk::new(s.ctx());
    let mut policy = s.take_shared<TransferPolicy<Card>>();
    kiosk::lock(&mut mine, &my_cap, &policy, card);
    kiosk::list<Card>(&mut mine, &my_cap, item, 0);
    let mut vault = s.take_shared<Vault>();
    let mut strongroom = s.take_shared_by_id<Kiosk>(vault.strongroom());
    vault::deposit_card_from_kiosk(&mut vault, &mut strongroom, &mut mine, item, &mut policy, b"jieun2".to_string(), s.ctx());
    assert!(vault.cards_out() == 0);
    assert!(kiosk::is_locked(&strongroom, item));
    transfer::public_share_object(mine);
    transfer::public_transfer(my_cap, PLAYER);
    ts::return_shared(strongroom);
    ts::return_shared(vault);
    ts::return_shared(policy);
    s.end();
}

#[test, expected_failure(abort_code = ::cyberbrawl::vault::ENotTheStrongroom)]
fun a_deposit_needs_the_strongroom() {
    let mut s = setup();
    let mut vault = s.take_shared<Vault>();
    vault::withdraw_card(&mut vault, b"jieun2".to_string(), b"CARD00073".to_string(), b"Double Time".to_string(), b"d".to_string(), b"u".to_string(), b"t".to_string(), b"Legendary".to_string(), PLAYER, s.ctx());
    ts::return_shared(vault);
    s.next_tx(PLAYER);
    let card = s.take_from_sender<Card>();
    let item = object::id(&card);
    let (mut mine, my_cap) = kiosk::new(s.ctx());
    let (mut other, _other_cap) = kiosk::new(s.ctx());
    let mut policy = s.take_shared<TransferPolicy<Card>>();
    kiosk::place_and_list(&mut mine, &my_cap, card, 0);
    let mut vault = s.take_shared<Vault>();
    vault::deposit_card_from_kiosk(&mut vault, &mut other, &mut mine, item, &mut policy, b"jieun2".to_string(), s.ctx());
    abort 0
}

#[test]
fun admin_changes_hands() {
    let mut s = setup();
    let mut vault = s.take_shared<Vault>();
    vault::set_admin(&mut vault, OTHER, s.ctx());
    assert!(vault.admin() == OTHER);
    s.next_tx(OTHER);
    vault::withdraw_credit(&mut vault, b"jieun2".to_string(), ONE, PLAYER, s.ctx());
    assert!(vault.credit_out() == ONE);
    ts::return_shared(vault);
    s.end();
}

#[test, expected_failure(abort_code = ::cyberbrawl::vault::ENotAdmin)]
fun only_the_admin_withdraws() {
    let mut s = setup();
    let mut vault = s.take_shared<Vault>();
    s.next_tx(OTHER);
    vault::withdraw_credit(&mut vault, b"jieun2".to_string(), ONE, PLAYER, s.ctx());
    ts::return_shared(vault);
    s.end();
}

#[test, expected_failure(abort_code = ::cyberbrawl::vault::EInvalidPlayer)]
fun a_deposit_needs_a_player() {
    let mut s = setup();
    let mut vault = s.take_shared<Vault>();
    vault::withdraw_credit(&mut vault, b"jieun2".to_string(), ONE, PLAYER, s.ctx());
    s.next_tx(PLAYER);
    let coin = s.take_from_sender<Coin<CREDIT>>();
    vault::deposit_credit(&mut vault, b"".to_string(), coin, s.ctx());
    ts::return_shared(vault);
    s.end();
}

#[test, expected_failure(abort_code = ::cyberbrawl::vault::EInvalidAmount)]
fun nothing_is_withdrawn_for_nothing() {
    let mut s = setup();
    let mut vault = s.take_shared<Vault>();
    vault::withdraw_credit(&mut vault, b"jieun2".to_string(), 0, PLAYER, s.ctx());
    ts::return_shared(vault);
    s.end();
}
