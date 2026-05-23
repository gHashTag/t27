use anchor_lang::prelude::*;

declare_id!("GAoPb3sVjtg1ey7gRtVGWtWFxGhK1eazdKanz3LZKmqV");

const CHAMPION_WEIGHTS: [[u8; 16]; 16] = [
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

#[program]
pub mod tri_mining {
    use super::*;

    pub fn initialize_epoch(ctx: Context<InitializeEpoch>, epoch_id: u64, block_reward: u64) -> Result<()> {
        let epoch = &mut ctx.accounts.mining_epoch;
        epoch.epoch_id = epoch_id;
        epoch.block_reward = block_reward;
        epoch.total_proofs = 0;
        epoch.total_tokens_minted = 0;
        epoch.authority = ctx.accounts.authority.key();
        Ok(())
    }

    pub fn submit_proof(ctx: Context<SubmitProof>, phi_response: [u8; 4], merkle_root: [u8; 32], signature: [u8; 64]) -> Result<()> {
        let epoch = &mut ctx.accounts.mining_epoch;
        let node_proof = &mut ctx.accounts.node_proof;

        let node_id = ctx.accounts.miner.key();
        let challenge = compute_phi_challenge(epoch.epoch_id, node_id.as_ref());

        require!(
            verify_phi_response(&challenge, &phi_response, node_id.as_ref()),
            TriError::PhiChallengeMismatch
        );

        node_proof.miner = node_id;
        node_proof.epoch_id = epoch.epoch_id;
        node_proof.phi_response = phi_response;
        node_proof.merkle_root = merkle_root;
        node_proof.signature = signature;
        node_proof.tokens_earned = epoch.block_reward / 1000;
        node_proof.timestamp = Clock::get()?.unix_timestamp;

        epoch.total_proofs += 1;
        epoch.total_tokens_minted += node_proof.tokens_earned;

        emit!(ProofSubmitted {
            miner: node_id,
            epoch_id: epoch.epoch_id,
            tokens: node_proof.tokens_earned,
        });

        Ok(())
    }

    pub fn submit_proof_v2(ctx: Context<SubmitProofV2>, phi_response: [u8; 32], merkle_root: [u8; 32], signature: [u8; 64]) -> Result<()> {
        let epoch = &mut ctx.accounts.mining_epoch;
        let node_proof = &mut ctx.accounts.node_proof;

        let node_id = ctx.accounts.miner.key();
        let node_id_bytes = node_id.as_ref();

        let challenge = derive_phi_challenge_v2(epoch.epoch_id, node_id_bytes);

        require!(
            verify_phi_response_v2(&challenge, &phi_response),
            TriError::PhiChallengeMismatch
        );

        node_proof.miner = node_id;
        node_proof.epoch_id = epoch.epoch_id;
        node_proof.phi_response = phi_response;
        node_proof.merkle_root = merkle_root;
        node_proof.signature = signature;
        node_proof.tokens_earned = epoch.block_reward / 1000;
        node_proof.timestamp = Clock::get()?.unix_timestamp;

        epoch.total_proofs += 1;
        epoch.total_tokens_minted += node_proof.tokens_earned;

        emit!(ProofSubmitted {
            miner: node_id,
            epoch_id: epoch.epoch_id,
            tokens: node_proof.tokens_earned,
        });

        Ok(())
    }
}

fn compute_phi_challenge(epoch_id: u64, node_id: &[u8]) -> [u8; 16] {
    use anchor_lang::solana_program::hash::hash;
    let mut preimage = Vec::new();
    preimage.extend_from_slice(b"TRI_PHI_CHALLENGE_V1");
    preimage.extend_from_slice(&epoch_id.to_le_bytes());
    preimage.extend_from_slice(node_id);
    let h = hash(&preimage);
    let mut challenge = [0u8; 16];
    challenge.copy_from_slice(&h.to_bytes()[..16]);
    challenge
}

fn verify_phi_response(challenge: &[u8; 16], response: &[u8; 4], node_id: &[u8]) -> bool {
    let w: [u8; 4] = match challenge[..4].try_into() {
        Ok(v) => v,
        Err(_) => return false,
    };
    let x: [u8; 4] = match node_id.get(..4) {
        Some(s) => [s[0], s[1], s[2], s[3]],
        None => return false,
    };
    let expected = gf16_dot4(&w, &x);
    response == expected.as_slice()
}

fn gf16_mul(a: u8, b: u8) -> u8 {
    let (mut a, mut b, mut p) = (a & 0xF, b & 0xF, 0u8);
    for _ in 0..4 {
        if b & 1 != 0 {
            p ^= a;
        }
        let carry = a & 0x8;
        a = (a << 1) & 0xF;
        if carry != 0 {
            a ^= 0x3;
        }
        b >>= 1;
    }
    p
}

fn gf16_dot4(w: &[u8; 4], x: &[u8; 4]) -> Vec<u8> {
    vec![gf16_mul(w[0], x[0]), gf16_mul(w[1], x[1]), gf16_mul(w[2], x[2]), gf16_mul(w[3], x[3])]
}

fn gf16_matmul(a: &[[u8; 16]; 16], b: &[[u8; 16]; 16]) -> [[u8; 16]; 16] {
    let mut c = [[0u8; 16]; 16];
    for i in 0..16 {
        for j in 0..16 {
            let mut acc = 0u8;
            for k in 0..16 {
                acc ^= gf16_mul(a[i][k] & 0xF, b[k][j] & 0xF);
            }
            c[i][j] = acc & 0xF;
        }
    }
    c
}

fn pack_gf16_matrix(m: &[[u8; 16]; 16]) -> [u8; 128] {
    let mut out = [0u8; 128];
    for i in 0..16 {
        for j in 0..8 {
            out[i * 8 + j] = ((m[i][j * 2] & 0xF) << 4) | (m[i][j * 2 + 1] & 0xF);
        }
    }
    out
}

fn derive_phi_challenge_v2(epoch_id: u64, node_id: &[u8]) -> [[u8; 16]; 16] {
    use anchor_lang::solana_program::hash::hash;
    let prefix = b"TRI_PHI_CHALLENGE_V2";
    let epoch_bytes = epoch_id.to_le_bytes();
    let mut matrix = [[0u8; 16]; 16];
    for i in 0u8..16 {
        let mut input = Vec::with_capacity(prefix.len() + 8 + node_id.len() + 1);
        input.extend_from_slice(prefix);
        input.extend_from_slice(&epoch_bytes);
        input.extend_from_slice(node_id);
        input.push(i);
        let h = hash(&input);
        let h_bytes = h.to_bytes();
        for j in 0..16 {
            matrix[i as usize][j] = (h_bytes[j * 2] >> 4) & 0xF;
        }
    }
    matrix
}

fn verify_phi_response_v2(challenge: &[[u8; 16]; 16], response: &[u8; 32]) -> bool {
    let product = gf16_matmul(&CHAMPION_WEIGHTS, challenge);
    let packed = pack_gf16_matrix(&product);
    let expected = hash_to_32(&packed);
    response.iter().zip(expected.iter()).fold(0u8, |acc, (a, b)| acc | (a ^ b)) == 0
}

fn hash_to_32(data: &[u8]) -> [u8; 32] {
    use anchor_lang::solana_program::hash::hash;
    let h = hash(data);
    let mut out = [0u8; 32];
    out.copy_from_slice(&h.to_bytes()[..32]);
    out
}

#[derive(Accounts)]
#[instruction(epoch_id: u64)]
pub struct InitializeEpoch<'info> {
    #[account(
        init,
        payer = authority,
        space = 8 + 8 + 8 + 8 + 8 + 32,
        seeds = [b"epoch", epoch_id.to_le_bytes().as_ref()],
        bump
    )]
    pub mining_epoch: Account<'info, MiningEpoch>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(phi_response: [u8; 4], merkle_root: [u8; 32], signature: [u8; 64])]
pub struct SubmitProof<'info> {
    #[account(mut)]
    pub mining_epoch: Account<'info, MiningEpoch>,
    #[account(
        init,
        payer = miner,
        space = 8 + 32 + 8 + 4 + 32 + 64 + 8 + 8,
        seeds = [b"proof", miner.key().as_ref(), mining_epoch.epoch_id.to_le_bytes().as_ref()],
        bump
    )]
    pub node_proof: Account<'info, NodeProof>,
    #[account(mut)]
    pub miner: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(phi_response: [u8; 32], merkle_root: [u8; 32], signature: [u8; 64])]
pub struct SubmitProofV2<'info> {
    #[account(mut)]
    pub mining_epoch: Account<'info, MiningEpoch>,
    #[account(
        init,
        payer = miner,
        space = 8 + 32 + 8 + 32 + 32 + 64 + 8 + 8,
        seeds = [b"proof-v2", miner.key().as_ref(), mining_epoch.epoch_id.to_le_bytes().as_ref()],
        bump
    )]
    pub node_proof: Account<'info, NodeProofV2>,
    #[account(mut)]
    pub miner: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[account]
pub struct MiningEpoch {
    pub epoch_id: u64,
    pub block_reward: u64,
    pub total_proofs: u64,
    pub total_tokens_minted: u64,
    pub authority: Pubkey,
}

#[account]
pub struct NodeProof {
    pub miner: Pubkey,
    pub epoch_id: u64,
    pub phi_response: [u8; 4],
    pub merkle_root: [u8; 32],
    pub signature: [u8; 64],
    pub tokens_earned: u64,
    pub timestamp: i64,
}

#[account]
pub struct NodeProofV2 {
    pub miner: Pubkey,
    pub epoch_id: u64,
    pub phi_response: [u8; 32],
    pub merkle_root: [u8; 32],
    pub signature: [u8; 64],
    pub tokens_earned: u64,
    pub timestamp: i64,
}

#[error_code]
pub enum TriError {
    #[msg("phi_challenge mismatch — response does not match challenge")]
    PhiChallengeMismatch,
    #[msg("merkle proof invalid")]
    MerkleProofInvalid,
    #[msg("Ed25519 signature verification failed")]
    SignatureInvalid,
    #[msg("epoch already submitted")]
    EpochAlreadySubmitted,
}

#[event]
pub struct ProofSubmitted {
    #[index]
    pub miner: Pubkey,
    #[index]
    pub epoch_id: u64,
    pub tokens: u64,
}
