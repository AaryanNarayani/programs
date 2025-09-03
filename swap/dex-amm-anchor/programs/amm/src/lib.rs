#![allow(unexpected_cfgs)]
use anchor_lang::prelude::*;
pub mod instructions;
declare_id!("85krVjvbktge3QdRPU5dRYSaaSXi2CgB7cqhreoABi36");
pub use instructions::*;
pub mod state;
pub mod constants;


#[program]
pub mod amm {
    use super::*;

    pub fn initialize(ctx: Context<Init>, seeds:u64, fee: u64, owner: Option<Pubkey>) -> Result<()> {
        ctx.accounts.handle_initialize(seeds, &ctx.bumps , fee, owner)?;
        Ok(())
    }

    // pub fn deposit(ctx: Context<Deposit>) -> Result<()> {
    //     msg!("Deposit Instruction done");
    //     Ok(())
    // }


    // pub fn withdraw(ctx: Context<Withdraw>) -> Result<()> {
    //     msg!("Withdraw Instruction done");
    //     Ok(())
    // }


    // pub fn swap(ctx: Context<Swap>) -> Result<()> {
    //     msg!("Swap Instruction done");
    //     Ok(())
    // }

    // pub fn update(ctx: Context<Update>) -> Result<()> {
    //     msg!("Update Instruction done");
    //     Ok(())
    // }
}

