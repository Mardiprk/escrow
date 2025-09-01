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
        let escrow = &mut ctx.accounts.escrow_account;
        let clock = Clock::get()?;

        require!(expiry_time > clock.unix_timestamp, EscrowError::InvalidExpiryTime);
        require!(amount > 0, EscrowError::InvalidAmount);

        escrow.escrow_id = escrow_id;
        escrow.depositor = ctx.accounts.depositor.key();
        escrow.beneficiary = ctx.accounts.beneficiary.key();
        escrow.mint = ctx.accounts.mint.key();
        escrow.amount = amount;
        escrow.expiry_time = expiry_time;
        escrow.is_completed = false;
        escrow.bump = *ctx.bumps.get("escrow_account").unwrap();

        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.depositor_token_account.to_account_info(),
                    to: ctx.accounts.escrow_vault.to_account_info(),
                    authority: ctx.accounts.depositor.to_account_info(),
                },
            ),
            amount,
        )?;

        emit!(EscrowInitialized {
            escrow_id,
            depositor: ctx.accounts.depositor.key(),
            beneficiary: ctx.accounts.beneficiary.key(),
            amount,
            expiry_time,
        });

        Ok(())
    }

    pub fn release_escrow(ctx: Context<ReleaseEscrow>) -> Result<()> {
        let escrow = &mut ctx.accounts.escrow_account;
        let clock = Clock::get()?;

        require!(!escrow.is_completed, EscrowError::EscrowAlreadyCompleted);

        let authority_seeds = &[
            b"escrow".as_ref(),
            &escrow.escrow_id.to_le_bytes(),
            &[escrow.bump],
        ];
        let signer = &[&authority_seeds[..]];

        token::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.escrow_vault.to_account_info(),
                    to: ctx.accounts.beneficiary_token_account.to_account_info(),
                    authority: ctx.accounts.escrow_account.to_account_info(),
                },
                signer,
            ),
            escrow.amount,
        )?;

        escrow.is_completed = true;

        emit!(EscrowReleased {
            escrow_id: escrow.escrow_id,
            beneficiary: escrow.beneficiary,
            amount: escrow.amount,
            released_by: ctx.accounts.authority.key(),
        });

        Ok(())
    }

    pub fn cancel_escrow(ctx: Context<CancelEscrow>) -> Result<()> {
        let escrow = &mut ctx.accounts.escrow_account;
        let clock = Clock::get()?;

        require!(!escrow.is_completed, EscrowError::EscrowAlreadyCompleted);
        require!(
            ctx.accounts.depositor.key() == escrow.depositor,
            EscrowError::UnauthorizedCancellation
        );
        require!(
            clock.unix_timestamp < escrow.expiry_time,
            EscrowError::EscrowExpired
        );

        let authority_seeds = &[
            b"escrow".as_ref(),
            &escrow.escrow_id.to_le_bytes(),
            &[escrow.bump],
        ];
        let signer = &[&authority_seeds[..]];

        // Return tokens to depositor
        token::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.escrow_vault.to_account_info(),
                    to: ctx.accounts.depositor_token_account.to_account_info(),
                    authority: ctx.accounts.escrow_account.to_account_info(),
                },
                signer,
            ),
            escrow.amount,
        )?;

        escrow.is_completed = true;

        emit!(EscrowCancelled {
            escrow_id: escrow.escrow_id,
            depositor: escrow.depositor,
            amount: escrow.amount,
        });

        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(escrow_id: u64)]
pub struct InitializeEscrow<'info> {
    #[account(mut)]
    pub depositor: Signer<'info>,

    /// CHECK: This is safe as we only store the pubkey
    pub beneficiary: AccountInfo<'info>,

    pub mint: Account<'info, Mint>,

    #[account(
        init,
        payer = depositor,
        space = EscrowAccount::LEN,
        seeds = [b"escrow", escrow_id.to_le_bytes().as_ref()],
        bump
    )]
    pub escrow_account: Account<'info, EscrowAccount>,

    #[account(
        init,
        payer = depositor,
        token::mint = mint,
        token::authority = escrow_account,
        seeds = [b"vault", escrow_id.to_le_bytes().as_ref()],
        bump
    )]
    pub escrow_vault: Account<'info, TokenAccount>,

    #[account(
        mut,
        constraint = depositor_token_account.owner == depositor.key(),
        constraint = depositor_token_account.mint == mint.key()
    )]
    pub depositor_token_account: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct ReleaseEscrow<'info> {
    pub authority: Signer<'info>,

    #[account(
        mut,
        seeds = [b"escrow", escrow_account.escrow_id.to_le_bytes().as_ref()],
        bump = escrow_account.bump
    )]
    pub escrow_account: Account<'info, EscrowAccount>,

    #[account(
        mut,
        seeds = [b"vault", escrow_account.escrow_id.to_le_bytes().as_ref()],
        bump
    )]
    pub escrow_vault: Account<'info, TokenAccount>,

    #[account(
        mut,
        constraint = beneficiary_token_account.owner == escrow_account.beneficiary,
        constraint = beneficiary_token_account.mint == escrow_vault.mint
    )]
    pub beneficiary_token_account: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct CancelEscrow<'info> {
    pub depositor: Signer<'info>,

    #[account(
        mut,
        seeds = [b"escrow", escrow_account.escrow_id.to_le_bytes().as_ref()],
        bump = escrow_account.bump
    )]
    pub escrow_account: Account<'info, EscrowAccount>,

    #[account(
        mut,
        seeds = [b"vault", escrow_account.escrow_id.to_le_bytes().as_ref()],
        bump
    )]
    pub escrow_vault: Account<'info, TokenAccount>,

    #[account(
        mut,
        constraint = depositor_token_account.owner == depositor.key(),
        constraint = depositor_token_account.mint == escrow_vault.mint
    )]
    pub depositor_token_account: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
}

#[account]
pub struct EscrowAccount {
    pub escrow_id: u64,
    pub depositor: Pubkey,
    pub beneficiary: Pubkey,
    pub mint: Pubkey,
    pub amount: u64,
    pub expiry_time: i64,
    pub is_completed: bool,
    pub bump: u8,
}

impl EscrowAccount {
    pub const LEN: usize = 8 + // discriminator
        8 + // escrow_id
        32 + // depositor
        32 + // beneficiary
        32 + // mint
        8 + // amount
        8 + // expiry_time
        1 + // is_completed
        1; // bump
}

#[event]
pub struct EscrowInitialized {
    pub escrow_id: u64,
    pub depositor: Pubkey,
    pub beneficiary: Pubkey,
    pub amount: u64,
    pub expiry_time: i64,
}

#[event]
pub struct EscrowReleased {
    pub escrow_id: u64,
    pub beneficiary: Pubkey,
    pub amount: u64,
    pub released_by: Pubkey,
}

#[event]
pub struct EscrowCancelled {
    pub escrow_id: u64,
    pub depositor: Pubkey,
    pub amount: u64,
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