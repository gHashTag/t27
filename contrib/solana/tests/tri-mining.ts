// @ts-nocheck
import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { TriMining } from "../target/types/tri_mining";
import { assert } from "chai";
import { sha256 } from "js-sha256";

function gf16Mul(a: number, b: number): number {
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

function gf16Dot4(w: number[], x: number[]): number[] {
  return [
    gf16Mul(w[0], x[0]),
    gf16Mul(w[1], x[1]),
    gf16Mul(w[2], x[2]),
    gf16Mul(w[3], x[3]),
  ];
}

const CHAMPION_WEIGHTS: number[][] = [
  [0x4, 0xF, 0xA, 0x7, 0x2, 0x8, 0x6, 0x1, 0xA, 0x2, 0x4, 0xC, 0x0, 0x6, 0x1, 0x5],
  [0xB, 0x7, 0x2, 0x4, 0x6, 0xA, 0x3, 0x7, 0xA, 0x3, 0xF, 0x9, 0x5, 0x1, 0xD, 0x1],
  [0xC, 0x7, 0x3, 0xA, 0x5, 0x2, 0x1, 0xF, 0x4, 0x2, 0x9, 0x7, 0x2, 0x9, 0x0, 0xB],
  [0xD, 0xE, 0x7, 0x9, 0xE, 0x2, 0x6, 0x1, 0xC, 0xF, 0x7, 0xE, 0x7, 0x6, 0x6, 0x1],
  [0xB, 0x7, 0x3, 0x9, 0x2, 0x4, 0xE, 0x1, 0xF, 0x5, 0x9, 0x7, 0xD, 0xB, 0x9, 0x2],
  [0x1, 0x5, 0x1, 0xB, 0x8, 0x2, 0x2, 0xB, 0x9, 0x9, 0x7, 0xB, 0x9, 0x9, 0x3, 0xB],
  [0x2, 0x1, 0xA, 0x7, 0xD, 0x1, 0x2, 0xB, 0x3, 0x7, 0x4, 0xF, 0xC, 0x7, 0x5, 0xD],
  [0xA, 0x8, 0xB, 0x1, 0xC, 0xA, 0x4, 0xC, 0xE, 0x5, 0x7, 0xF, 0x6, 0xA, 0xA, 0xA],
  [0xC, 0x9, 0x7, 0x6, 0xF, 0x4, 0x5, 0x7, 0x1, 0x2, 0xD, 0x0, 0xF, 0xE, 0x6, 0x0],
  [0x7, 0xE, 0xA, 0xE, 0x7, 0xB, 0x5, 0x7, 0x4, 0xC, 0xB, 0x3, 0x7, 0x4, 0xB, 0xE],
  [0xB, 0x8, 0x4, 0x9, 0x0, 0xE, 0x0, 0x6, 0x9, 0x5, 0x1, 0xA, 0x6, 0x5, 0x5, 0x8],
  [0x8, 0x2, 0xC, 0x4, 0x7, 0x6, 0x2, 0x2, 0xF, 0xA, 0xA, 0x1, 0x3, 0xD, 0x0, 0x6],
  [0xA, 0x4, 0x6, 0xF, 0x9, 0xC, 0x4, 0xB, 0xB, 0xD, 0x6, 0x2, 0xA, 0x5, 0x9, 0x5],
  [0x8, 0x6, 0xA, 0x7, 0x0, 0xC, 0x0, 0x8, 0x8, 0xF, 0x4, 0xE, 0x6, 0xA, 0x5, 0x5],
  [0xB, 0x5, 0x1, 0x8, 0xD, 0x8, 0x2, 0x8, 0x0, 0xE, 0xD, 0x4, 0x1, 0x0, 0x7, 0xC],
  [0x2, 0x3, 0xA, 0xE, 0x5, 0x5, 0xC, 0xB, 0x3, 0x8, 0x1, 0xD, 0xA, 0xA, 0x2, 0xF],
];

function gf16Matmul(a: number[][], b: number[][]): number[][] {
  const c: number[][] = [];
  for (let i = 0; i < 16; i++) {
    const row: number[] = [];
    for (let j = 0; j < 16; j++) {
      let acc = 0;
      for (let k = 0; k < 16; k++) {
        acc ^= gf16Mul(a[i][k] & 0xf, b[k][j] & 0xf);
      }
      row.push(acc & 0xf);
    }
    c.push(row);
  }
  return c;
}

function packGf16Matrix(m: number[][]): Buffer {
  const out = Buffer.alloc(128);
  for (let i = 0; i < 16; i++) {
    for (let j = 0; j < 8; j++) {
      out[i * 8 + j] = ((m[i][j * 2] & 0xf) << 4) | (m[i][j * 2 + 1] & 0xf);
    }
  }
  return out;
}

function derivePhiChallengeV2(epochId: number, nodeId: Uint8Array): number[][] {
  const matrix: number[][] = [];
  for (let i = 0; i < 16; i++) {
    const buf = Buffer.alloc(8);
    buf.writeBigUInt64LE(BigInt(epochId));
    const input = Buffer.concat([
      Buffer.from("TRI_PHI_CHALLENGE_V2"),
      buf,
      Buffer.from(nodeId),
      Buffer.from([i]),
    ]);
    const hashBytes = Buffer.from(sha256(input), "hex");
    const row: number[] = [];
    for (let j = 0; j < 16; j++) {
      row.push((hashBytes[j * 2] >> 4) & 0xf);
    }
    matrix.push(row);
  }
  return matrix;
}

function computePhiResponseV2(epochId: number, nodeId: Uint8Array): number[] {
  const challenge = derivePhiChallengeV2(epochId, nodeId);
  const product = gf16Matmul(CHAMPION_WEIGHTS, challenge);
  const packed = packGf16Matrix(product);
  const hashBytes = Buffer.from(sha256(packed), "hex");
  return Array.from(hashBytes);
}

function computePhiChallenge(epochId: number, nodeId: Uint8Array): Uint8Array {
  const preimage = Buffer.concat([
    Buffer.from("TRI_PHI_CHALLENGE_V1"),
    (() => {
      const buf = Buffer.alloc(8);
      buf.writeBigUInt64LE(BigInt(epochId));
      return buf;
    })(),
    Buffer.from(nodeId),
  ]);
  const hashHex = sha256(preimage);
  const hashBytes = Buffer.from(hashHex, "hex");
  return hashBytes.slice(0, 16);
}

function computePhiResponse(epochId: number, nodeId: Uint8Array): number[] {
  const challenge = computePhiChallenge(epochId, nodeId);
  const w = [challenge[0], challenge[1], challenge[2], challenge[3]];
  const x = [nodeId[0], nodeId[1], nodeId[2], nodeId[3]];
  return gf16Dot4(w, x);
}

describe("tri-mining", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  const program = anchor.workspace.TriMining as Program<TriMining>;

  const epochId = new anchor.BN(1);
  const blockReward = new anchor.BN(50_000_000);

  it("initializes epoch", async () => {
    const [epochPda] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("epoch"), epochId.toArrayLike(Buffer, "le", 8)],
      program.programId
    );

    await program.methods
      .initializeEpoch(epochId, blockReward)
      .accounts({
        miningEpoch: epochPda,
        authority: provider.wallet.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();

    const epoch = await program.account.miningEpoch.fetch(epochPda);
    assert.equal(epoch.epochId.toNumber(), 1);
    assert.equal(epoch.blockReward.toNumber(), 50_000_000);
    assert.equal(epoch.totalProofs.toNumber(), 0);
  });

  it("3 nodes submit valid proofs and receive mock rewards", async () => {
    const [epochPda] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("epoch"), epochId.toArrayLike(Buffer, "le", 8)],
      program.programId
    );

    const miners: anchor.web3.Keypair[] = [];
    for (let i = 0; i < 3; i++) {
      const miner = anchor.web3.Keypair.generate();
      const airdropSig = await provider.connection.requestAirdrop(
        miner.publicKey,
        10 * anchor.web3.LAMPORTS_PER_SOL
      );
      await provider.connection.confirmTransaction(airdropSig);
      miners.push(miner);
    }

    for (let i = 0; i < miners.length; i++) {
      const miner = miners[i];
      const nodeId = miner.publicKey.toBuffer();

      const phiResponse = computePhiResponse(1, nodeId);
      const merkleRoot = new Array(32).fill(0);
      const signature = new Array(64).fill(0);

      const [proofPda] = anchor.web3.PublicKey.findProgramAddressSync(
        [
          Buffer.from("proof"),
          miner.publicKey.toBuffer(),
          epochId.toArrayLike(Buffer, "le", 8),
        ],
        program.programId
      );

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
      assert.equal(
        proof.miner.toString(),
        miner.publicKey.toString(),
        `node ${i}: miner mismatch`
      );
      assert.equal(proof.epochId.toNumber(), 1, `node ${i}: epoch mismatch`);
      assert.equal(
        proof.tokensEarned.toNumber(),
        50_000,
        `node ${i}: expected 50_000 tokens (block_reward / 1000)`
      );
    }

    const epoch = await program.account.miningEpoch.fetch(epochPda);
    assert.equal(epoch.totalProofs.toNumber(), 3, "expected 3 total proofs");
    assert.equal(
      epoch.totalTokensMinted.toNumber(),
      150_000,
      "expected 150_000 total tokens (3 x 50_000)"
    );
  });

  it("V2: node submits valid SHA256 response and earns tokens", async () => {
    const [epochPda] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("epoch"), epochId.toArrayLike(Buffer, "le", 8)],
      program.programId
    );

    const miner = anchor.web3.Keypair.generate();
    const airdropSig = await provider.connection.requestAirdrop(
      miner.publicKey,
      10 * anchor.web3.LAMPORTS_PER_SOL
    );
    await provider.connection.confirmTransaction(airdropSig);

    const nodeId = miner.publicKey.toBuffer();
    const phiResponse = computePhiResponseV2(1, nodeId);
    assert.equal(phiResponse.length, 32, "V2 response must be 32 bytes");

    const merkleRoot = new Array(32).fill(0);
    const signature = new Array(64).fill(0);

    const [proofPda] = anchor.web3.PublicKey.findProgramAddressSync(
      [
        Buffer.from("proof_v2"),
        miner.publicKey.toBuffer(),
        epochId.toArrayLike(Buffer, "le", 8),
      ],
      program.programId
    );

    await program.methods
      .submitProofV2(phiResponse, merkleRoot, signature)
      .accounts({
        miningEpoch: epochPda,
        nodeProof: proofPda,
        miner: miner.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([miner])
      .rpc({ skipPreflight: true });

    const proof = await program.account.nodeProofV2.fetch(proofPda);
    assert.equal(proof.miner.toString(), miner.publicKey.toString());
    assert.equal(proof.epochId.toNumber(), 1);
    assert.equal(proof.version, 2, "version field must be 2");
    assert.equal(proof.tokensEarned.toNumber(), 50_000);
    assert.equal(proof.phiResponse.length, 32);
  });

  it("V2: rejects invalid SHA256 response (flipped bit)", async () => {
    const [epochPda] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("epoch"), epochId.toArrayLike(Buffer, "le", 8)],
      program.programId
    );

    const miner = anchor.web3.Keypair.generate();
    const airdropSig = await provider.connection.requestAirdrop(
      miner.publicKey,
      10 * anchor.web3.LAMPORTS_PER_SOL
    );
    await provider.connection.confirmTransaction(airdropSig);

    const nodeId = miner.publicKey.toBuffer();
    const phiResponse = computePhiResponseV2(1, nodeId);
    phiResponse[0] ^= 0x01;

    const merkleRoot = new Array(32).fill(0);
    const signature = new Array(64).fill(0);

    const [proofPda] = anchor.web3.PublicKey.findProgramAddressSync(
      [
        Buffer.from("proof_v2"),
        miner.publicKey.toBuffer(),
        epochId.toArrayLike(Buffer, "le", 8),
      ],
      program.programId
    );

    try {
      await program.methods
        .submitProofV2(phiResponse, merkleRoot, signature)
        .accounts({
          miningEpoch: epochPda,
          nodeProof: proofPda,
          miner: miner.publicKey,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .signers([miner])
        .rpc({ skipPreflight: true });
      assert.fail("expected V2 transaction to fail with PhiChallengeMismatch");
    } catch (err: any) {
      const errorMsg = err.toString();
      const hasError = errorMsg.includes("PhiChallengeMismatch") ||
        errorMsg.includes("phi_challenge mismatch");
      assert.isTrue(
        hasError,
        `expected PhiChallengeMismatch error, got: ${errorMsg.slice(0, 200)}`
      );
    }
  });

  it("rejects invalid phi_response", async () => {
    const [epochPda] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("epoch"), epochId.toArrayLike(Buffer, "le", 8)],
      program.programId
    );

    const miner = anchor.web3.Keypair.generate();
    const airdropSig = await provider.connection.requestAirdrop(
      miner.publicKey,
      10 * anchor.web3.LAMPORTS_PER_SOL
    );
    await provider.connection.confirmTransaction(airdropSig);
    const wrongResponse = [0xff, 0xff, 0xff, 0xff];
    const merkleRoot = new Array(32).fill(0);
    const signature = new Array(64).fill(0);

    const [proofPda] = anchor.web3.PublicKey.findProgramAddressSync(
      [
        Buffer.from("proof"),
        miner.publicKey.toBuffer(),
        epochId.toArrayLike(Buffer, "le", 8),
      ],
      program.programId
    );

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
      assert.fail("expected transaction to fail with PhiChallengeMismatch");
    } catch (err: any) {
      const errorMsg = err.toString();
      const hasError = errorMsg.includes("PhiChallengeMismatch") ||
        errorMsg.includes("phi_challenge mismatch");
      assert.isTrue(
        hasError,
        `expected PhiChallengeMismatch error, got: ${errorMsg.slice(0, 200)}`
      );
    }
  });
});
