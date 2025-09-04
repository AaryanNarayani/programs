use anchor_lang::prelude::*;
use anchor_spl::{associated_token::AssociatedToken, token::{transfer , mint_to , MintTo ,Mint, Token, TokenAccount, Transfer}};
use constant_product_curve::ConstantProduct;

use crate::{error::{AmmDexError, PoolConfigError}, state::PoolConfig};
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
        // Lock Checks & other checks
        if self.pool_config.owner.is_none() {
            return Err(PoolConfigError::PoolNotInitialized.into());
        }
        if self.pool_config.is_locked {
            return Err(PoolConfigError::PoolLocked.into());
        }
        if amount == 0 || max_token_x == 0 || max_token_y == 0 {
            return Err(PoolConfigError::InvalidAmount.into());
        }
        // Get x and y deposit amounts
        let mut deposit_x = 0;
        let mut deposit_y = 0;

        if self.lp_token.supply == 0 && self.token_x_vault.amount == 0 && self.token_y_vault.amount == 0 {
            // first time deposit
            deposit_x = max_token_x;
            deposit_y = max_token_y;
        } else {
            let deposit_amount = ConstantProduct::xy_deposit_amounts_from_l(
                self.token_x_vault.amount,
                self.token_y_vault.amount,
                self.lp_token.supply,
                amount,
                6
            ).unwrap();
            deposit_x = deposit_amount.x;
            deposit_y = deposit_amount.y;
        }
        if deposit_x <= max_token_x && deposit_y <= max_token_y {
            return Err(AmmDexError::SlippageToleranceExceeded.into());
        }
        // deposit liquidity
        // Transfer token x from user to vault
        let cpi_program_token_x = self.token_program.to_account_info();
        let cpi_accounts_x = Transfer {
            from: self.user_x_token.to_account_info(),
            to: self.token_x_vault.to_account_info(),
            authority: self.user.to_account_info(),
        };
        let cpi_context_x = CpiContext::new(cpi_program_token_x, cpi_accounts_x);
        transfer(cpi_context_x, deposit_x);
        msg!("Deposited X Amount: {}, Token", deposit_x);

        // Transfer token y from user to vault
        let cpi_program_token_y = self.token_program.to_account_info();
        let cpi_accounts_y = Transfer {
            from: self.user_y_token.to_account_info(),
            to: self.token_y_vault.to_account_info(),
            authority: self.user.to_account_info(),
        };
        let cpi_context_y = CpiContext::new(cpi_program_token_y, cpi_accounts_y);
        transfer(cpi_context_y, deposit_y);

        msg!("Deposited Y Amount: {}, Token", deposit_y);

        // Mint LP tokens to user
        let cpi_program_lp = self.token_program.to_account_info();
        let cpi_accounts_lp = MintTo {
            mint: self.lp_token.to_account_info(),
            to: self.user_lp_token_ac.to_account_info(),
            authority: self.pool_config.to_account_info(),
        };
        let seeds = &[
            b"pool_config",
            &self.pool_config.seeds.to_be_bytes()[..],
            &[self.pool_config.pool_config_bump],
        ];

        let signer_seeds = &[&seeds[..]];
        let cpi_context_lp = CpiContext::new_with_signer(cpi_program_lp, cpi_accounts_lp, signer_seeds);
        mint_to(cpi_context_lp, amount)?;
        msg!("Minted LP Amount: {}, Token", amount);
        Ok(())
    }
}