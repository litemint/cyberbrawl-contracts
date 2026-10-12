/*
    Copyright (c) 2026 LITEMINT LLC

    This file is part of CYBERBRAWL-CONTRACTS project.
    Licensed under the MIT License.
    Author: Fred Kyung-jin Rezeau (오경진 吳景振) <hello@kyungj.in>
*/

module cyberbrawl::card;

use std::string::String;
use sui::display_registry::{Self, DisplayRegistry};
use sui::package::{Self, Publisher};
use sui::transfer_policy;
use sui::vec_map::{Self, VecMap};
use kiosk::kiosk_lock_rule;
use kiosk::royalty_rule;

/// Royalty on every Kiosk sale, in basis points.
const ROYALTY_BPS: u16 = 500;

public struct CARD has drop {}

public struct Card has key, store {
    id: UID,
    code: String,
    name: String,
    description: String,
    image_url: String,
    thumbnail_url: String,
    attributes: VecMap<String, String>,
}

fun init(witness: CARD, ctx: &mut TxContext) {
    let publisher = package::claim(witness, ctx);
    let (mut policy, policy_cap) = transfer_policy::new<Card>(&publisher, ctx);
    kiosk_lock_rule::add(&mut policy, &policy_cap);
    royalty_rule::add(&mut policy, &policy_cap, ROYALTY_BPS, 0);
    transfer::public_share_object(policy);
    transfer::public_transfer(policy_cap, ctx.sender());
    transfer::public_transfer(publisher, ctx.sender());
}

public fun register_display(registry: &mut DisplayRegistry, publisher: &mut Publisher, ctx: &mut TxContext) {
    let (mut display, cap) = display_registry::new_with_publisher<Card>(registry, publisher, ctx);
    display.set(&cap, b"name".to_string(), b"{name}".to_string());
    display.set(&cap, b"description".to_string(), b"{description}".to_string());
    display.set(&cap, b"image_url".to_string(), b"https://cdn.litemint.com/cards/{code}.webp".to_string());
    display.set(&cap, b"thumbnail_url".to_string(), b"{thumbnail_url}".to_string());
    display.set(&cap, b"link".to_string(), b"https://undercity.cyberbrawl.io/market/{code}".to_string());
    display.set(&cap, b"project_url".to_string(), b"https://cyberbrawl.io".to_string());
    display.set(&cap, b"creator".to_string(), b"Litemint".to_string());
    display.share();
    transfer::public_transfer(cap, ctx.sender());
}

public(package) fun mint(code: String, name: String, description: String, image_url: String, thumbnail_url: String, tier: String, ctx: &mut TxContext): Card {
    let mut attributes = vec_map::empty();
    attributes.insert(b"Tier".to_string(), tier);
    Card { id: object::new(ctx), code, name, description, image_url, thumbnail_url, attributes }
}

public(package) fun burn(card: Card): String {
    let Card { id, code, name: _, description: _, image_url: _, thumbnail_url: _, attributes: _ } = card;
    id.delete();
    code
}

public fun code(card: &Card): &String {
    &card.code
}

public fun name(card: &Card): &String {
    &card.name
}

public fun description(card: &Card): &String {
    &card.description
}

public fun image_url(card: &Card): &String {
    &card.image_url
}

public fun thumbnail_url(card: &Card): &String {
    &card.thumbnail_url
}

public fun attributes(card: &Card): &VecMap<String, String> {
    &card.attributes
}

#[test_only]
public fun init_for_testing(ctx: &mut TxContext) {
    init(CARD {}, ctx);
}
