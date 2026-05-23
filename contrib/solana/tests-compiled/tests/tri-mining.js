"use strict";
var __createBinding = (this && this.__createBinding) || (Object.create ? (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    var desc = Object.getOwnPropertyDescriptor(m, k);
    if (!desc || ("get" in desc ? !m.__esModule : desc.writable || desc.configurable)) {
      desc = { enumerable: true, get: function() { return m[k]; } };
    }
    Object.defineProperty(o, k2, desc);
}) : (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    o[k2] = m[k];
}));
var __setModuleDefault = (this && this.__setModuleDefault) || (Object.create ? (function(o, v) {
    Object.defineProperty(o, "default", { enumerable: true, value: v });
}) : function(o, v) {
    o["default"] = v;
});
var __importStar = (this && this.__importStar) || function (mod) {
    if (mod && mod.__esModule) return mod;
    var result = {};
    if (mod != null) for (var k in mod) if (k !== "default" && Object.prototype.hasOwnProperty.call(mod, k)) __createBinding(result, mod, k);
    __setModuleDefault(result, mod);
    return result;
};
Object.defineProperty(exports, "__esModule", { value: true });
// @ts-nocheck
const anchor = __importStar(require("@coral-xyz/anchor"));
const chai_1 = require("chai");
const js_sha256_1 = require("js-sha256");
function gf16Mul(a, b) {
    let aa = a & 0xf;
    let bb = b & 0xf;
    let p = 0;
    for (let i = 0; i < 4; i++) {
        if ((bb & 1) !== 0) {
            p ^= aa;
        }
        const carry = aa & 0x8;
        aa = (aa << 1) & 0xf;
        if (carry !== 0) {
            aa ^= 0x3;
        }
        bb >>= 1;
    }
    return p;
}
function gf16Dot4(w, x) {
    return [
        gf16Mul(w[0], x[0]),
        gf16Mul(w[1], x[1]),
        gf16Mul(w[2], x[2]),
        gf16Mul(w[3], x[3]),
    ];
}
function computePhiChallenge(epochId, nodeId) {
    const preimage = Buffer.concat([
        Buffer.from("TRI_PHI_CHALLENGE_V1"),
        (() => {
            const buf = Buffer.alloc(8);
            buf.writeBigUInt64LE(BigInt(epochId));
            return buf;
        })(),
        Buffer.from(nodeId),
    ]);
    const hashHex = (0, js_sha256_1.sha256)(preimage);
    const hashBytes = Buffer.from(hashHex, "hex");
    return hashBytes.slice(0, 16);
}
function computePhiResponse(epochId, nodeId) {
    const challenge = computePhiChallenge(epochId, nodeId);
    const w = [challenge[0], challenge[1], challenge[2], challenge[3]];
    const x = [nodeId[0], nodeId[1], nodeId[2], nodeId[3]];
    return gf16Dot4(w, x);
}
describe("tri-mining", () => {
    const provider = anchor.AnchorProvider.env();
    anchor.setProvider(provider);
    const program = anchor.workspace.TriMining;
    const epochId = new anchor.BN(1);
    const blockReward = new anchor.BN(50000000);
    it("initializes epoch", async () => {
        const [epochPda] = anchor.web3.PublicKey.findProgramAddressSync([Buffer.from("epoch"), epochId.toArrayLike(Buffer, "le", 8)], program.programId);
        await program.methods
            .initializeEpoch(epochId, blockReward)
            .accounts({
            miningEpoch: epochPda,
            authority: provider.wallet.publicKey,
            systemProgram: anchor.web3.SystemProgram.programId,
        })
            .rpc();
        const epoch = await program.account.miningEpoch.fetch(epochPda);
        chai_1.assert.equal(epoch.epochId.toNumber(), 1);
        chai_1.assert.equal(epoch.blockReward.toNumber(), 50000000);
        chai_1.assert.equal(epoch.totalProofs.toNumber(), 0);
    });
    it("3 nodes submit valid proofs and receive mock rewards", async () => {
        const [epochPda] = anchor.web3.PublicKey.findProgramAddressSync([Buffer.from("epoch"), epochId.toArrayLike(Buffer, "le", 8)], program.programId);
        const miners = [];
        for (let i = 0; i < 3; i++) {
            const miner = anchor.web3.Keypair.generate();
            const airdropSig = await provider.connection.requestAirdrop(miner.publicKey, 10 * anchor.web3.LAMPORTS_PER_SOL);
            await provider.connection.confirmTransaction(airdropSig);
            miners.push(miner);
        }
        for (let i = 0; i < miners.length; i++) {
            const miner = miners[i];
            const nodeId = miner.publicKey.toBuffer();
            const phiResponse = computePhiResponse(1, nodeId);
            const merkleRoot = new Array(32).fill(0);
            const signature = new Array(64).fill(0);
            const [proofPda] = anchor.web3.PublicKey.findProgramAddressSync([
                Buffer.from("proof"),
                miner.publicKey.toBuffer(),
                epochId.toArrayLike(Buffer, "le", 8),
            ], program.programId);
            await program.methods
                .submitProof(phiResponse, merkleRoot, signature)
                .accounts({
                miningEpoch: epochPda,
                nodeProof: proofPda,
                miner: miner.publicKey,
                systemProgram: anchor.web3.SystemProgram.programId,
            })
                .signers([miner])
                .rpc({ skipPreflight: true });
            const proof = await program.account.nodeProof.fetch(proofPda);
            chai_1.assert.equal(proof.miner.toString(), miner.publicKey.toString(), `node ${i}: miner mismatch`);
            chai_1.assert.equal(proof.epochId.toNumber(), 1, `node ${i}: epoch mismatch`);
            chai_1.assert.equal(proof.tokensEarned.toNumber(), 50000, `node ${i}: expected 50_000 tokens (block_reward / 1000)`);
        }
        const epoch = await program.account.miningEpoch.fetch(epochPda);
        chai_1.assert.equal(epoch.totalProofs.toNumber(), 3, "expected 3 total proofs");
        chai_1.assert.equal(epoch.totalTokensMinted.toNumber(), 150000, "expected 150_000 total tokens (3 x 50_000)");
    });
    it("rejects invalid phi_response", async () => {
        const [epochPda] = anchor.web3.PublicKey.findProgramAddressSync([Buffer.from("epoch"), epochId.toArrayLike(Buffer, "le", 8)], program.programId);
        const miner = anchor.web3.Keypair.generate();
        const airdropSig = await provider.connection.requestAirdrop(miner.publicKey, 10 * anchor.web3.LAMPORTS_PER_SOL);
        await provider.connection.confirmTransaction(airdropSig);
        const wrongResponse = [0xff, 0xff, 0xff, 0xff];
        const merkleRoot = new Array(32).fill(0);
        const signature = new Array(64).fill(0);
        const [proofPda] = anchor.web3.PublicKey.findProgramAddressSync([
            Buffer.from("proof"),
            miner.publicKey.toBuffer(),
            epochId.toArrayLike(Buffer, "le", 8),
        ], program.programId);
        try {
            await program.methods
                .submitProof(wrongResponse, merkleRoot, signature)
                .accounts({
                miningEpoch: epochPda,
                nodeProof: proofPda,
                miner: miner.publicKey,
                systemProgram: anchor.web3.SystemProgram.programId,
            })
                .signers([miner])
                .rpc({ skipPreflight: true });
            chai_1.assert.fail("expected transaction to fail with PhiChallengeMismatch");
        }
        catch (err) {
            const errorMsg = err.toString();
            const hasError = errorMsg.includes("PhiChallengeMismatch") ||
                errorMsg.includes("phi_challenge mismatch");
            chai_1.assert.isTrue(hasError, `expected PhiChallengeMismatch error, got: ${errorMsg.slice(0, 200)}`);
        }
    });
});
