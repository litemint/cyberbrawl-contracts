/*
    Copyright (c) 2026 LITEMINT LLC

    This file is part of CYBERBRAWL-CONTRACTS project.
    Licensed under the MIT License.
    Author: Fred Kyung-jin Rezeau (오경진 吳景振) <hello@kyungj.in>
*/

use soroban_sdk::{contracterror, contracttype, Address, BytesN};

// The Engram fact and the least value it may hold.
#[derive(Clone, Copy)]
pub struct Gate {
    pub fact: u32,
    pub threshold: u64,
}

#[derive(Clone, Copy)]
pub struct BeamParams {
    pub gates: [Gate; 1],
}

impl BeamParams {
    pub const fn default() -> Self {
        Self {
            // Tier 1: fact 4096 + achievement index (17), at least 1.
            gates: [Gate { fact: 4113, threshold: 1 }],
        }
    }
}

pub struct Quote {
    pub card: Address,
    pub receiver: Address,
    pub price: i128,
    pub tier: u32,
    pub expiry: u64,
}

pub struct VerificationKeys {
    pub alpha: [u8; 64],
    pub beta: [u8; 128],
    pub gamma: [u8; 128],
    pub delta: [u8; 128],
    pub ic: &'static [[u8; 64]],
}

// Engram key (point of Baby Jubjub: Ax, Ay).
#[derive(Clone)]
#[contracttype]
pub struct Issuer(pub BytesN<32>, pub BytesN<32>);

#[derive(Clone, Copy)]
#[contracterror]
pub enum Error {
    Expired = 1,
    InvalidAttestation = 2,
    InvalidProof = 3,
    NoStock = 4,
}

#[derive(Clone)]
#[contracttype]
pub enum Storage {
    Admin,
    Ion,
    Attestor,
    Issuer,
}
