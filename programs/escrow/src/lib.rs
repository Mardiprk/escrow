use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount, Transfer};

declare_id!("7ptKBMKwd4CdXWP5NXAoqXzsGxhE5ifmDVqxJMtDP9J5");

#[program]
pub mod escrow {
    use super::*;

    pub fn initialize_escrow(
        ctx: Context<InitializeEscrow>,
        escrow_id: u64,
        amount: u64,
        expiry_time: i64,
    ) -> Result<()> {
        Ok(())
    }

    pub fn release_escrow(ctx: Context<ReleaseEscrow>) -> Result<()> {
        Ok(())
    }

    pub fn cancel_escrow(ctx: CancelEscrow) -> Result<()> {
        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(escrow_id: u64)]
pub struct InitializeEscrow<'info> {}

#[derive(Accounts)]
pub struct ReleaseEscrow<'info> {}

#[derive(Accounts)]
pub struct CancelEscrow<'info> {}

#[account]
#[derive(InitSpace)]
pub struct EscrowAccount {
    pub escrow_id: u64,
    pub depositer: Pubkey,
    pub beneficiary: Pubkey,
    pub mint: Pubkey,
    pub amount: u64,
    pub expiry_time: i64,
    pub is_completed: bool,
    pub bump: u8,
}

#[error_code]
pub enum EscrowError {
    #[msg("Invalid expiry time")]
    InvalidExpiryTime,
    #[msg("Invalid amount")]
    InvalidAmount,
    #[msg("Escrow already completed")]
    EscrowAlreadyCompleted,
    #[msg("Unauthorized cancellation")]
    UnauthorizedCancellation,
    #[msg("Escrow has expired")]
    EscrowExpired,
}
