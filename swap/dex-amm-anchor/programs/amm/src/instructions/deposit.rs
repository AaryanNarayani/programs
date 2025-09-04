use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{transfer, mint_to, MintTo, Mint, Token, TokenAccount, Transfer},
};
use constant_product_curve::ConstantProduct;

use crate::{
    error::{AmmDexError, PoolConfigError},
    state::PoolConfig,
};

#[derive(Accounts)]
pub struct Deposit<'info> {
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

impl<'info> Deposit<'info> {
    pub fn handle_deposit(
        &mut self,
        amount: u64,
        max_token_x: u64,
        max_token_y: u64,
    ) -> Result<()> {
        // Amount Checks & other checks
        if self.pool_config.owner.is_none() {
            return Err(PoolConfigError::PoolNotInitialized.into());
        }
        if self.pool_config.is_locked {
            return Err(PoolConfigError::PoolLocked.into());
        }
        if amount == 0 || max_token_x == 0 || max_token_y == 0 {
            return Err(PoolConfigError::InvalidAmount.into());
        }

        // Calculate required deposit amounts
        let (deposit_x, deposit_y) = if self.lp_token.supply == 0
            && self.token_x_vault.amount == 0
            && self.token_y_vault.amount == 0
        {
            // first deposit
            (max_token_x, max_token_y)
        } else {
            let dep = ConstantProduct::xy_deposit_amounts_from_l(
                self.token_x_vault.amount,
                self.token_y_vault.amount,
                self.lp_token.supply,
                amount,
                6,
            )
            .map_err(|_| PoolConfigError::InvalidAmount)?;
            (dep.x, dep.y)
        };

        if deposit_x > max_token_x || deposit_y > max_token_y {
            return Err(AmmDexError::SlippageToleranceExceeded.into());
        }

        // Transfer tokens to vault
        self.transfer_tokens(&self.user_x_token, &self.token_x_vault, deposit_x)?;
        self.transfer_tokens(&self.user_y_token, &self.token_y_vault, deposit_y)?;

        // Mint LP tokens to user
        self.mint_lp_tokens(amount)?;

        msg!(
            "Deposit complete: X = {}, Y = {}, LP = {}",
            deposit_x,
            deposit_y,
            amount
        );

        Ok(())
    }

    fn transfer_tokens(
        &self,
        from: &Account<'info, TokenAccount>,
        to: &Account<'info, TokenAccount>,
        amount: u64,
    ) -> Result<()> {
        let cpi_accounts = Transfer {
            from: from.to_account_info(),
            to: to.to_account_info(),
            authority: self.user.to_account_info(),
        };
        let cpi_ctx = CpiContext::new(self.token_program.to_account_info(), cpi_accounts);
        transfer(cpi_ctx, amount)?;
        Ok(())
    }

    fn mint_lp_tokens(&self, amount: u64) -> Result<()> {
        let cpi_accounts = MintTo {
            mint: self.lp_token.to_account_info(),
            to: self.user_lp_token_ac.to_account_info(),
            authority: self.pool_config.to_account_info(),
        };

        let seeds = &[
            b"pool-config",
            self.pool_config.owner.as_ref().unwrap().as_ref(),
            &[self.pool_config.pool_config_bump],
        ];
        let signer_seeds = &[&seeds[..]];

        let cpi_ctx = CpiContext::new_with_signer(
            self.token_program.to_account_info(),
            cpi_accounts,
            signer_seeds,
        );

        mint_to(cpi_ctx, amount)?;
        Ok(())
    }
}
