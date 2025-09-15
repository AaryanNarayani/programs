#![allow(deprecated)]
#![allow(unexpected_cfgs)]
use anchor_lang::prelude::*;

declare_id!("8FTigEfLnLyrqNsg2U589WcpaGt5e8mnzNjkxQ52hbhX");

/*
    Simple token program demonstration:
    - Create token mint accounts
    - Create token accounts for mints
    - Delegation of token authority and user can also delegate their token accounts
    - Transfer tokens between accounts
    - Freeze/thaw token accounts
    
    Note: This is a simplified educational example, not for production use.
*/

#[program]
pub mod simple_token_anchor {
    use super::*;

    pub fn initialize(ctx: Context<InitializeMint>, decimals: u8, supply: u64) -> Result<()> {
        let mint_account = &mut ctx.accounts.mint_account;
        let owner = &ctx.accounts.owner;
        mint_account.decimals = decimals;
        mint_account.mint_authority = owner.key();
        mint_account.freeze_authority = owner.key();
        mint_account.supply = supply;
        mint_account.is_initialized = true;
        mint_account.is_frozen = false;
        msg!("Mint initialized with {} decimals and supply {}", decimals, supply);
        Ok(())
    }

    pub fn create_ata(ctx: Context<CreateAta>) -> Result<()> {
        let ata_account = &mut ctx.accounts.ata_account;
        ata_account.owner = ctx.accounts.payer.key();
        ata_account.mint = ctx.accounts.mint.key();
        ata_account.amount = 0;
        ata_account.is_initialized = true;
        ata_account.delegate = None;
        ata_account.delegate_amount = 0;
        Ok(())
    }

    pub fn mint(ctx: Context<MintTokens>, amount: u64) -> Result<()> {
        let mint_account = &mut ctx.accounts.mint_account;
        let ata_account = &mut ctx.accounts.ata_account;

        require!(mint_account.is_initialized, MintError::MintNotInitialized);
        require_keys_eq!(mint_account.mint_authority, ctx.accounts.owner.key(), MintError::UnauthorizedMint);
        require!(!mint_account.is_frozen, MintError::MintFrozen);
        require!(amount > 0, MintError::InvalidAmount);

        mint_account.supply = mint_account.supply.checked_add(amount).ok_or(MintError::SupplyOverflow)?;
        ata_account.amount = ata_account.amount.checked_add(amount).ok_or(MintError::AmountOverflow)?;
        Ok(())
    }

    pub fn delegation(ctx: Context<Delegate>, amount: u64) -> Result<()> {
        let ata_account = &mut ctx.accounts.ata_account;
        require!(ata_account.delegate.is_none(), DelegateError::DelegateAlreadySet);
        require!(amount > 0 && amount <= ata_account.amount, DelegateError::InvalidAmount);

        ata_account.delegate = Some(ctx.accounts.delegate.key());
        ata_account.delegate_amount = amount;
        Ok(())
    }

    pub fn revoke_delegation(ctx: Context<RevokeDelegation>) -> Result<()> {
        let ata_account = &mut ctx.accounts.ata_account;
        ata_account.delegate = None;
        ata_account.delegate_amount = 0;
        Ok(())
    }

    pub fn transfer(ctx: Context<Transfer>, amount: u64) -> Result<()> {
        let from = &mut ctx.accounts.from;
        let to = &mut ctx.accounts.to;
        let signer = ctx.accounts.signer.key();

        require!(from.is_initialized, DelegateError::ATANotInitialized);
        require!(amount > 0, DelegateError::InvalidAmount);
        require!(from.amount >= amount, DelegateError::AmountOverflow);

        // Check if signer is owner or delegate with enough allowance
        if signer != from.owner {
            match from.delegate {
                Some(d) if d == signer => {
                    require!(from.delegate_amount >= amount, DelegateError::InvalidAmount);
                    from.delegate_amount -= amount;
                }
                _ => return Err(DelegateError::UnauthorizedOwner.into()),
            }
        }

        from.amount = from.amount.checked_sub(amount).ok_or(DelegateError::AmountOverflow)?;
        to.amount = to.amount.checked_add(amount).ok_or(DelegateError::AmountOverflow)?;
        Ok(())
    }

    pub fn freeze(ctx: Context<Freeze>) -> Result<()> {
        let mint_account = &mut ctx.accounts.mint_account;
        require!(!mint_account.is_frozen, MintError::MintFrozen);
        mint_account.is_frozen = true;
        Ok(())
    }

    pub fn thaw(ctx: Context<Thaw>) -> Result<()> {
        let mint_account = &mut ctx.accounts.mint_account;
        require!(mint_account.is_frozen, MintError::MintNotFrozen);
        mint_account.is_frozen = false;
        Ok(())
    }
}

#[error_code]
pub enum MintError {
    #[msg("Mint account is not initialized")]
    MintNotInitialized,
    #[msg("You are not the mint authority")]
    UnauthorizedMint,
    #[msg("The mint is frozen")]
    MintFrozen,
    #[msg("Mint supply overflow")]
    SupplyOverflow,
    #[msg("ATA amount overflow")]
    AmountOverflow,
    #[msg("Invalid amount, must be greater than zero")]
    InvalidAmount,
    #[msg("Mint is not frozen")]
    MintNotFrozen,
}

#[error_code]
pub enum DelegateError {
    #[msg("ATA account is not initialized")]
    ATANotInitialized,
    #[msg("You are not the ATA owner or delegate")]
    UnauthorizedOwner,
    #[msg("The delegate is already set")]
    DelegateAlreadySet,
    #[msg("ATA amount overflow")]
    AmountOverflow,
    #[msg("Invalid amount")]
    InvalidAmount,
}

#[account]
pub struct Mint {
    pub decimals: u8,
    pub mint_authority: Pubkey,
    pub freeze_authority: Pubkey,
    pub supply: u64,
    pub is_initialized: bool,
    pub is_frozen: bool,
}

#[account]
pub struct ATA {
    pub owner: Pubkey,
    pub mint: Pubkey,
    pub amount: u64,
    pub is_initialized: bool,
    pub delegate: Option<Pubkey>,
    pub delegate_amount: u64,
}

#[derive(Accounts)]
pub struct InitializeMint<'info> {
    #[account(init, payer = payer, space = 88)]
    pub mint_account: Account<'info, Mint>,
    #[account(mut)]
    pub payer: Signer<'info>,
    /// CHECK: mint authority key
    pub owner: AccountInfo<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct CreateAta<'info> {
    #[account(
        init,
        payer = payer,
        seeds = [b"ata", payer.key().as_ref(), mint.key().as_ref()],
        bump,
        space = 128
    )]
    pub ata_account: Account<'info, ATA>,
    #[account(mut)]
    pub payer: Signer<'info>,
    pub mint: Account<'info, Mint>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct MintTokens<'info> {
    #[account(mut)]
    pub mint_account: Account<'info, Mint>,
    #[account(mut)]
    pub ata_account: Account<'info, ATA>,
    pub owner: Signer<'info>,
}

#[derive(Accounts)]
pub struct Delegate<'info> {
    /// CHECK: delegate key
    pub delegate: UncheckedAccount<'info>,
    #[account(mut, has_one = owner @ DelegateError::UnauthorizedOwner)]
    pub ata_account: Account<'info, ATA>,
    pub owner: Signer<'info>,
}

#[derive(Accounts)]
pub struct RevokeDelegation<'info> {
    #[account(mut, has_one = owner @ DelegateError::UnauthorizedOwner)]
    pub ata_account: Account<'info, ATA>,
    pub owner: Signer<'info>,
}

#[derive(Accounts)]
pub struct Transfer<'info> {
    #[account(mut)]
    pub from: Account<'info, ATA>,
    #[account(mut)]
    pub to: Account<'info, ATA>,
    /// CHECK: Can be either owner or delegate
    pub signer: Signer<'info>,
}

#[derive(Accounts)]
pub struct Freeze<'info> {
    #[account(mut)]
    pub mint_account: Account<'info, Mint>,
    pub owner: Signer<'info>,
}

#[derive(Accounts)]
pub struct Thaw<'info> {
    #[account(mut)]
    pub mint_account: Account<'info, Mint>,
    pub owner: Signer<'info>,
}
