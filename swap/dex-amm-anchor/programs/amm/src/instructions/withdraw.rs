use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{burn, transfer, Burn, Mint, Token, TokenAccount, Transfer},
};

use crate::{
    error::{AmmDexError, PoolConfigError},
    state::PoolConfig,
};

#[derive(Accounts)]
pub struct Withdraw<'info> {
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
        mut,
        associated_token::mint = lp_token,
        associated_token::authority = user,
    )]
    pub user_lp_token_ac: Account<'info, TokenAccount>,
}

impl<'info> Withdraw<'info> {
    pub fn handle_withdraw(
        &mut self,
        amount: u64,
        min_token_x: u64,
        min_token_y: u64,
    ) -> Result<()> {
        // Amount Checks & other checks
        if self.pool_config.owner.is_none() {
            return Err(PoolConfigError::PoolNotInitialized.into());
        }
        if amount == 0 {
            return Err(PoolConfigError::InvalidAmount.into());
        }

        let (withdraw_x, withdraw_y) = self.calculate_withdraw_amounts(amount)?;
        if withdraw_x < min_token_x || withdraw_y < min_token_y {
            return Err(AmmDexError::SlippageToleranceExceeded.into());
        }

        // Withdraw tokens to user
        self.transfer_from_vault(&self.token_x_vault, &self.user_x_token, withdraw_x)?;
        self.transfer_from_vault(&self.token_y_vault, &self.user_y_token, withdraw_y)?;

        // Burn LP tokens from user
        self.burn_lp_tokens(amount)?;

        msg!(
            "Withdraw complete: X = {}, Y = {}, LP burned = {}",
            withdraw_x,
            withdraw_y,
            amount
        );

        Ok(())
    }

    fn calculate_withdraw_amounts(&self, lp_amount: u64) -> Result<(u64, u64)> {
        let supply = self.lp_token.supply;
        if supply == 0 {
            return Err(PoolConfigError::InvalidAmount.into());
        }

        let withdraw_x = (self.token_x_vault.amount as u128)
            .checked_mul(lp_amount as u128)
            .unwrap()
            / supply as u128;

        let withdraw_y = (self.token_y_vault.amount as u128)
            .checked_mul(lp_amount as u128)
            .unwrap()
            / supply as u128;

        Ok((withdraw_x as u64, withdraw_y as u64))
    }

    fn transfer_from_vault(
        &self,
        from: &Account<'info, TokenAccount>,
        to: &Account<'info, TokenAccount>,
        amount: u64,
    ) -> Result<()> {
        let cpi_accounts = Transfer {
            from: from.to_account_info(),
            to: to.to_account_info(),
            authority: self.pool_config.to_account_info(),
        };

        let seeds = &[
            b"pool-config",
            self.pool_config.owner.as_ref().unwrap().as_ref(),
            &[self.pool_config.pool_config_bump],
        ];
        let signer_seeds = &[&seeds[..]];

        let cpi_ctx =
            CpiContext::new_with_signer(self.token_program.to_account_info(), cpi_accounts, signer_seeds);
        transfer(cpi_ctx, amount)?;
        Ok(())
    }

    fn burn_lp_tokens(&self, amount: u64) -> Result<()> {
        let cpi_accounts = Burn {
            mint: self.lp_token.to_account_info(),
            from: self.user_lp_token_ac.to_account_info(),
            authority: self.user.to_account_info(),
        };

        let cpi_ctx = CpiContext::new(self.token_program.to_account_info(), cpi_accounts);
        burn(cpi_ctx, amount)?;
        Ok(())
    }
}
