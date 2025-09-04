use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};

use crate::state::PoolConfig;
use crate::constants::PROTOCOL_FEE;

#[derive(Accounts)]
pub struct Init<'info> {
    #[account(mut)]
    owner: Signer<'info>,
    #[account(
        init,
        payer = owner,
        seeds = [b"lp-token", owner.key().as_ref()],
        bump,
        mint::decimals = 6,
        mint::authority = pool_config,
    )]
    lp_token: Account<'info, Mint>,
    token_x_mint: Account<'info, Mint>,
    token_y_mint: Account<'info, Mint>,
    
    #[account(
        init,
        payer = owner,
        space = 8 + std::mem::size_of::<PoolConfig>(),
        seeds = [b"pool-config", owner.key().as_ref()],
        bump
    )]
    pool_config: Account<'info, PoolConfig>,

    #[account(
        init,
        payer = pool_config,
        token::mint = token_x_mint,
        token::authority = pool_config,
    )]
    token_x_vault: Account<'info, TokenAccount>,

    #[account(
        init,
        payer = pool_config,
        token::mint = token_y_mint,
        token::authority = pool_config,
    )]
    token_y_vault: Account<'info, TokenAccount>,

    system_program: Program<'info, System>,
    token_program: Program<'info, Token>,
}

impl<'info> Init<'info> {
    pub fn handle_initialize(
        &mut self,
        seeds: u64,
        bump: &InitBumps,
        fee: u16,
        owner: Option<Pubkey>,
    ) -> Result<()> {
        let pool_config = &mut self.pool_config.set_inner(
            PoolConfig {
                seeds,
                lp_fee: fee,
                protocol_fee: PROTOCOL_FEE,
                lp_bump: bump.lp_token,
                pool_config_bump: bump.pool_config,
                lp_token_mint: self.lp_token.key(),
                token_x_mint: self.token_x_mint.key(),
                token_y_mint: self.token_y_mint.key(),
                is_locked: false,
                owner,
            }
        );
        msg!("Pool Config initialized: {:?}", pool_config);
        Ok(())
    }
}