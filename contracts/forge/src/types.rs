/*
    Copyright (c) 2026 LITEMINT LLC

    This file is part of CYBERBRAWL-CONTRACTS project.
    Licensed under the MIT License.
    Author: Fred Kyung-jin Rezeau (오경진 吳景振) <hello@kyungj.in>
*/

use soroban_sdk::{contracterror, contracttype, Address, BytesN};

#[derive(Clone, Copy)]
pub struct ForgeParams {
    pub max_power: u32,
    pub duration: u64,
    pub premium_bps: [u32; 24],
}

impl ForgeParams {
    pub const fn default() -> Self {
        Self {
            max_power: 24,
            duration: 172_800, // A forge takes two days at power 1, two hours at power 24.
            // P^(ln 3 / ln 24) in basis points, 3x at power 24.
            premium_bps: [
                10000, 12708, 14620, 16148, 17443, 18578, 19595, 20520,
                21373, 22166, 22908, 23608, 24270, 24900, 25501, 26076,
                26629, 27160, 27673, 28168, 28647, 29111, 29562, 30000,
            ],
        }
    }
}

pub struct Quote {
    pub id: BytesN<16>,
    pub receiver: Address,
    pub base: i128,
    pub expiry: u64,
}

#[derive(Clone)]
#[contracttype]
pub struct Entry(pub Address, pub u64, pub u64, pub u32);

#[derive(Clone, Copy)]
#[contracterror]
pub enum Error {
    Expired = 1,
    InvalidAmount = 2,
    ForgeBusy = 3,
    NoEntry = 4,
    NotReady = 5,
    InvalidAttestation = 6,
    InvalidAction = 7,
}

#[derive(Clone)]
#[contracttype]
pub enum Storage {
    Admin,
    Credit,
    Ion,
    Attestor,
}
