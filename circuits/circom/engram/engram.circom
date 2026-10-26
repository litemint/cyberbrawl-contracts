/*
    Copyright (c) 2026 LITEMINT LLC

    This file is part of CYBERBRAWL-CONTRACTS project.
    Licensed under the MIT License.
    Author: Fred Kyung-jin Rezeau (오경진 吳景振) <hello@kyungj.in>
*/

pragma circom 2.1.9;

include "circomlib/circuits/poseidon.circom";
include "circomlib/circuits/eddsaposeidon.circom";
include "circomlib/circuits/bitify.circom";
include "circomlib/circuits/comparators.circom";

// One level of the sparse tree: the current node and its sibling, ordered by the position bit.
template Level() {
    signal input cur;
    signal input sib;
    signal input s;
    signal output out;

    s * (1 - s) === 0;
    signal d <== s * (sib - cur);
    component h = Poseidon(2);
    h.inputs[0] <== cur + d;
    h.inputs[1] <== sib - d;
    out <== h.out;
}

// The Engram circuit: a document of facts that never fall, signed by the server.
//
// The server signs Poseidon(commitment of the secret, account key, root of the fact tree) with a Baby Jubjub key
// (EdDSA-Poseidon). Fact F sits at position F of the tree and holds its value; an empty position holds 0.
//
// Public inputs:  Ax, Ay    the server's Engram key
//                 fact      the fact asked for
//                 threshold the least value the fact may hold
//                 drop      a domain for the one-use tag (a contract that does not use the tag passes 0)
//                 receiver  the one the proof is made for (the contract's address_to_field of the receiver)
// Public output:  nullifier Poseidon(accountKey, drop)
// Private:        the secret, the account key, the value of the fact, the signature and the siblings of the path.
//
// It proves: the value sits at position `fact` under a root the server signed for this account, the value is at least
// the threshold, and the proof belongs to `receiver` (a copied proof is useless to any other receiver).
template Engram(DEPTH) {
    signal input Ax;
    signal input Ay;
    signal input fact;
    signal input threshold;
    signal input drop;
    signal input receiver;
    signal output nullifier;

    signal input secret;
    signal input accountKey;
    signal input value;
    signal input R8x;
    signal input R8y;
    signal input S;
    signal input sib[DEPTH];

    component commit = Poseidon(1);
    commit.inputs[0] <== secret;

    // The path from the fact's leaf to the root.
    component pos = Num2Bits(DEPTH);
    pos.in <== fact;
    component lv[DEPTH];
    signal cur[DEPTH + 1];
    cur[0] <== value;
    for (var i = 0; i < DEPTH; i++) {
        lv[i] = Level();
        lv[i].cur <== cur[i];
        lv[i].sib <== sib[i];
        lv[i].s <== pos.out[i];
        cur[i + 1] <== lv[i].out;
    }

    // The fact is at least the threshold.
    component ge = LessEqThan(64);
    ge.in[0] <== threshold;
    ge.in[1] <== value;
    ge.out === 1;

    // The server signed this root for this account.
    component msg = Poseidon(3);
    msg.inputs[0] <== commit.out;
    msg.inputs[1] <== accountKey;
    msg.inputs[2] <== cur[DEPTH];
    component v = EdDSAPoseidonVerifier();
    v.enabled <== 1;
    v.Ax <== Ax;
    v.Ay <== Ay;
    v.R8x <== R8x;
    v.R8y <== R8y;
    v.S <== S;
    v.M <== msg.out;

    // The one-use tag.
    component nf = Poseidon(2);
    nf.inputs[0] <== accountKey;
    nf.inputs[1] <== drop;
    nullifier <== nf.out;

    // Binds the proof to the receiver: the signal takes part in a constraint, so it cannot be dropped or swapped.
    signal rsq <== receiver * receiver;
}

component main {public [Ax, Ay, fact, threshold, drop, receiver]} = Engram(16);
