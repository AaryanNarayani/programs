use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Mint, Token, TokenAccount},
};
use constant_product_curve::{ConstantProduct, LiquidityPair};

use crate::{error::PoolConfigError, state::PoolConfig};

#[derive(Accounts)]
pub struct Swap<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    pub token_x_mint: Account<'info, Mint>,
    pub token_y_mint: Account<'info, Mint>,

    #[account(associated_token::mint = token_x_mint, associated_token::authority = user)]
    pub user_x_token: Account<'info, TokenAccount>,

    #[account(associated_token::mint = token_y_mint, associated_token::authority = user)]
    pub user_y_token: Account<'info, TokenAccount>,

    #[account(associated_token::mint = token_x_mint, associated_token::authority = pool_config)]
    pub token_x_vault: Account<'info, TokenAccount>,

    #[account(associated_token::mint = token_y_mint, associated_token::authority = pool_config)]
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

impl<'info> Swap<'info> {
    pub fn handle_swap(&mut self, lp_pair_x: bool, amount: u64, min_swap_amount: u64) -> Result<()> {
        // Amount Checks & other checks
        if self.pool_config.owner.is_none() {
            return Err(PoolConfigError::PoolNotInitialized.into());
        }
        if amount == 0 || min_swap_amount == 0 {
            return Err(PoolConfigError::InvalidAmount.into());
        }
        // Set Liquidity Pair Direction
        let direction = match lp_pair_x {
            true => LiquidityPair::X,
            false => LiquidityPair::Y,
        };
        // Initialize Curve
        let mut curve = ConstantProduct::init(
            self.token_x_vault.amount,
            self.token_y_vault.amount,
            self.lp_token.supply,
            self.pool_config.lp_fee,
            None,
        ).unwrap();
        // Calculate Swap Amounts
        let swap_result = curve.swap(direction, amount, min_swap_amount).unwrap();
        msg!("Swap Result Deposit: {:?}", swap_result.deposit );
        msg!("Swap Result Withdrawal: {:?}", swap_result.withdraw);
        Ok(())
    }
}
