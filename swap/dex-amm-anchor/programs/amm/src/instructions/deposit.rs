use anchor_lang::prelude::*;
use anchor_spl::{associated_token::AssociatedToken, token::{Mint, Token, TokenAccount}};

use crate::state::PoolConfig;
#[derive(Accounts)]
pub struct Deposit<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    pub token_x_mint: Account<'info, Mint>,
    pub token_y_mint: Account<'info, Mint>,
    
    #[account(
        associated_token::mint = token_x_mint,
        associated_token::authority = user,
    )]
    pub user_x_token: Account<'info, TokenAccount>,
    
    #[account(
        associated_token::mint = token_y_mint,
        associated_token::authority = user,
    )]
    pub user_y_token: Account<'info, TokenAccount>,

    #[account(
        associated_token::mint = token_x_mint,
        associated_token::authority = pool_config,
    )]
    pub token_x_vault: Account<'info, TokenAccount>,

    #[account(
        associated_token::mint = token_y_mint,
        associated_token::authority = pool_config,
    )]
    pub token_y_vault: Account<'info, TokenAccount>,

    #[account(
        seeds = [b"pool-config", pool_config.owner.as_ref().unwrap().as_ref()],
        bump = pool_config.pool_config_bump,
        has_one = token_x_mint,
        has_one = token_y_mint,
    )]
    pub pool_config: Account<'info, PoolConfig>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,

    pub lp_token: Account<'info, Mint>,
    #[account(
        init_if_needed,
        payer = user,
        associated_token::mint = lp_token,
        associated_token::authority = user,
    )]
    pub user_lp_token_ac: Account<'info, TokenAccount>,
}

impl<'info> Deposit<'info> {
    pub fn handle_deposit(&mut self, amount: u64, max_token_x: u64, max_token_y: u64) -> Result<()> {
        // Lock Checks

        // Get x and y deposit amounts

        // deposit liquidity
        msg!("Deposit Instruction done");
        Ok(())
    }
}