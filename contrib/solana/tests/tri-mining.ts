import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { TriMining } from "../target/types/tri_mining";
import { assert } from "chai";

describe("tri-mining", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  const program = anchor.workspace.TriMining as Program<TriMining>;

  it("initializes epoch", async () => {
    const epochId = new anchor.BN(1);
    const blockReward = new anchor.BN(50_000_000);

    const [epochPda] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("epoch"), epochId.toArrayLike(Buffer, "le", 8)],
      program.programId
    );

    await program.methods
      .initializeEpoch(epochId, blockReward)
      .accounts({ miningEpoch: epochPda, authority: provider.wallet.publicKey })
      .rpc();

    const epoch = await program.account.miningEpoch.fetch(epochPda);
    assert.equal(epoch.epochId.toNumber(), 1);
    assert.equal(epoch.blockReward.toNumber(), 50_000_000);
    assert.equal(epoch.totalProofs.toNumber(), 0);
  });

  it("submits valid proof", async () => {
    const epochId = new anchor.BN(1);
    const miner = anchor.web3.Keypair.generate();

    const [epochPda] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("epoch"), epochId.toArrayLike(Buffer, "le", 8)],
      program.programId
    );

    const [proofPda] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("proof"), miner.publicKey.toBuffer(), epochId.toArrayLike(Buffer, "le", 8)],
      program.programId
    );

    const phiResponse = [0, 0, 0, 0];
    const merkleRoot = new Uint8Array(32);
    const signature = new Uint8Array(64);

    await program.methods
      .submitProof(phiResponse, Array.from(merkleRoot), Array.from(signature))
      .accounts({
        miningEpoch: epochPda,
        nodeProof: proofPda,
        miner: miner.publicKey,
      })
      .signers([miner])
      .rpc({ skipPreflight: true });
  });
});
