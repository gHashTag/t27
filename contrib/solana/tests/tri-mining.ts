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
