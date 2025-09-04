#![allow(unexpected_cfgs)]
#[warn(deprecated)]
use anchor_lang::prelude::*;
pub mod instructions;
declare_id!("85krVjvbktge3QdRPU5dRYSaaSXi2CgB7cqhreoABi36");
pub use instructions::*;
pub mod state;
pub mod constants;
pub mod error;


#[program]
pub mod amm {
    use super::*;

    pub fn initialize(ctx: Context<Init>, seeds:u64, fee: u16, owner: Option<Pubkey>) -> Result<()> {
        ctx.accounts.handle_initialize(seeds, &ctx.bumps , fee, owner)?;
        Ok(())
    }

    pub fn deposit(ctx: Context<Deposit>, amount: u64, max_token_x: u64, max_token_y: u64) -> Result<()> {
        ctx.accounts.handle_deposit(amount, max_token_x, max_token_y)?;
        Ok(())
    }

    pub fn withdraw(ctx: Context<Withdraw>, amount: u64, min_token_x: u64, min_token_y: u64) -> Result<()> {
        ctx.accounts.handle_withdraw(amount, min_token_x, min_token_y)?;
        Ok(())
    }

    pub fn swap(ctx: Context<Swap>, lp_pair_x: bool, amount: u64, min_swap_amount: u64) -> Result<()> {
        ctx.accounts.handle_swap(lp_pair_x, amount, min_swap_amount)?;
        Ok(())
    }

    pub fn update(ctx: Context<Update>) -> Result<()> {
        ctx.accounts.handle_update()?;
        Ok(())
    }
}

