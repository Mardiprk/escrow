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
pub struct InitializeEscrow<'info> {
    //→ the person locking tokens (signer)
    #[account(mut)]
    pub depoister: Signer<'info>,
    /// CHECK: This is safe as we only store the pubkey
    pub beneficiray: AccountInfo<'info>,
    //→ which token (e.g., USDC, ora ny custom SPL token).
    pub mint: Account<'info, Mint>,
    //→ PDA state account storing escrow metadata.
    #[account(
        init,
        payer = depositer,
        space = EscrowAccount::INIT_SPACE,
        seeds =[b"escrow", escrow_id.to_le_bytes().as_ref()],
        bump
    )]
    pub escrow_account: Account<'info, EscrowAccount>,
    //→ PDA token account holding tokens.
    #[account(
        init,
        payer = depositer,
        token::mint = mint,
        token::authority = escrow_account,
        seeds =[b"escrow", escrow_id.to_le_bytes().as_ref()],
        bump
    )]
    pub escrow_vault: Account<'info, TokenAccount>,
    //→ depositor’s token account (from which tokens are transferred).
    #[account(
        mut,
        constraint = depositer_token_account.owner == depoister.key(),
        constraint = depositer_token_account.mint == mint.key()
    )]
    pub depositer_token_account: Account<'info, TokenAccount>,
    pub token_program: Account<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct ReleaseEscrow<'info> {
    pub authority: Signer<'info>,

    #[account(
        mut,
        seeds =[b"escrow", escrow_account.escrow_id.to_le_bytes().as_ref()],
        bump
    )]
    pub escrow_account: Account<'info, EscrowAccount>,
    #[account(
        mut,
        seeds =[b"escrow", escrow_account.escrow_id.to_le_bytes().as_ref()],
        bump
    )]
    pub escrow_vault: Account<'info, TokenAccount>,

    #[account(
        mut,
        constraint = beneficiary_token_account.owner = escrow_account.beneficiary,
        constraint = beneficiary_token_account.mint = escrow_vault.mint
    )]
    pub beneficiary_token_account: Account<'info, TokenAccount>,
    pub token_program: Account<'info, Token>,
}

#[derive(Accounts)]
pub struct CancelEscrow<'info> {
    pub depositer: Signer<'info>,
    #[account(
        mut,
        seeds =[b"escrow", escrow_account.escrow_id.to_le_bytes().as_ref()],
        bump
    )]
    pub escrow_account: Account<'info, EscrowAccount>,
    #[account(
        mut,
        seeds =[b"escrow", escrow_account.escrow_id.to_le_bytes().as_ref()],
        bump
    )]
    pub escrow_vault: Account<'info, TokenAccount>,
    #[account(
        mut,
        constraint = beneficiary_token_account.owner = depoister.key(),
        constraint = beneficiary_token_account.mint = escrow_vault.mint
    )]
    pub beneficiary_token_account: Account<'info, TokenAccount>,
    pub token_program: Account<'info, Token>,
}
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
