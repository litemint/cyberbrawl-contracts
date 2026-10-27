// Proves "this Engram holds the fact" for one receiver and prints the 288 bytes of proof as hex, the proof argument of Beam's `etch`.
//
//   npm install snarkjs circomlibjs @stellar/stellar-sdk
//   node prove.js <engram.json> <receiver> <engram.wasm> <engram.zkey> [fact] [threshold]
//

const fs = require('fs');
const crypto = require('crypto');
const snarkjs = require('snarkjs');
const { buildPoseidon } = require('circomlibjs');
const { Address } = require('@stellar/stellar-sdk');

(async() => {
    // Fact and threshold default to the Ironclad gate: 4113, at least 1.
    // The receiver is the address of the quote.
    const [file, receiver, wasm, zkey, fact = '4113', threshold = '1'] = process.argv.slice(2);
    if (!zkey) {
        console.error('node prove.js <engram.json> <receiver> <engram.wasm> <engram.zkey> [fact] [threshold]');
        process.exit(1);
    }
    const engram = JSON.parse(fs.readFileSync(file, 'utf8'));
    const poseidon = await buildPoseidon();
    const H = (...values) => poseidon.F.toObject(poseidon(values.map(value => BigInt(value))));

    // The tree: only the nodes above a fact are hashed.
    const zero = [0n];
    for (let level = 0; level < engram.depth; level += 1) {
        zero.push(H(zero[level], zero[level]));
    }
    const facts = new Map(engram.facts.map(entry => [entry.id, BigInt(entry.value)]));
    const node = (level, index) => {
        if (![...facts.keys()].some(key => Math.floor(key / 2 ** level) === index)) {
            return zero[level];
        }
        return level === 0 ? facts.get(index) : H(node(level - 1, index * 2), node(level - 1, index * 2 + 1));
    };
    if (node(engram.depth, 0).toString() !== engram.root) {
        throw new Error('The Engram does not match its own root.');
    }
    const value = facts.get(Number(fact)) || 0n;
    if (value < BigInt(threshold)) {
        throw new Error('The Engram does not hold this fact.');
    }
    const sib = [];
    for (let level = 0; level < engram.depth; level += 1) {
        sib.push(node(level, Math.floor(Number(fact) / 2 ** level) ^ 1).toString());
    }

    // The receiver as a field element: the first 31 bytes of the SHA-256 of its XDR.
    const digest = crypto.createHash('sha256').update(new Address(receiver).toScVal().toXDR()).digest();
    const input = {
        Ax: engram.server.Ax,
        Ay: engram.server.Ay,
        fact,
        threshold,
        drop: '0',
        receiver: BigInt('0x' + digest.subarray(0, 31).toString('hex')).toString(),
        secret: engram.secret,
        accountKey: engram.accountKey,
        value: value.toString(),
        R8x: engram.signature.R8x,
        R8y: engram.signature.R8y,
        S: engram.signature.S,
        sib
    };

    const { proof, publicSignals } = await snarkjs.groth16.fullProve(input, wasm, zkey);
    const hex = number => BigInt(number).toString(16).padStart(64, '0');
    console.log([
        proof.pi_a[0], proof.pi_a[1],
        proof.pi_b[0][0], proof.pi_b[0][1], proof.pi_b[1][0], proof.pi_b[1][1],
        proof.pi_c[0], proof.pi_c[1],
        publicSignals[0]
    ].map(hex).join(''));
    process.exit(0);
})().catch((error) => {
    console.error(error.message);
    process.exit(1);
});
