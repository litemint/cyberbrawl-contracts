/*
    Copyright (c) 2026 LITEMINT LLC

    This file is part of CYBERBRAWL-CONTRACTS project.
    Licensed under the MIT License.
    Author: Fred Kyung-jin Rezeau (오경진 吳景振) <hello@kyungj.in>
*/

use soroban_sdk::{contractevent, contracttype};

#[contractevent]
pub struct Record {
    #[topic]
    pub day: u32,
}

#[derive(Clone)]
#[contracttype]
pub enum Storage {
    Admin,
    Scribe,
}
