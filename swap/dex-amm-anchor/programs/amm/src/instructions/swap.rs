use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Mint, Token, TokenAccount,transfer},
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
        self.deposit_to_vault(
            if lp_pair_x { &self.user_x_token } else { &self.user_y_token },
            if lp_pair_x { &self.token_x_vault } else { &self.token_y_vault },
            swap_result.deposit,
        )?;
        self.withdraw_from_vault(
            if lp_pair_x { &self.token_y_vault } else { &self.token_x_vault },
            if lp_pair_x { &self.user_y_token } else { &self.user_x_token },
            swap_result.withdraw,
        )?;
        Ok(())
    }
    fn deposit_to_vault(
        &self,
        from: &Account<'info, TokenAccount>,
        to: &Account<'info, TokenAccount>,
        amount: u64,
    ) -> Result<()> {
        let cpi_accounts = anchor_spl::token::Transfer {
            from: from.to_account_info(),
            to: to.to_account_info(),
            authority: self.user.to_account_info(),
        };
        let cpi_program = self.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        transfer(cpi_ctx, amount)?;
        Ok(())
    }
    fn withdraw_from_vault(
        &self,
        from: &Account<'info, TokenAccount>,
        to: &Account<'info, TokenAccount>,
        amount: u64,
    ) -> Result<()> {
        let seeds = &[
            b"pool-config",
            self.pool_config.owner.as_ref().unwrap().as_ref(),
            &[self.pool_config.pool_config_bump],
        ];
        let signer = &[&seeds[..]];
        let cpi_accounts = anchor_spl::token::Transfer {
            from: from.to_account_info(),
            to: to.to_account_info(),
            authority: self.pool_config.to_account_info(),
        };
        let cpi_program = self.token_program.to_account_info();
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
        transfer(cpi_ctx, amount)?;
        Ok(())
    }
}
