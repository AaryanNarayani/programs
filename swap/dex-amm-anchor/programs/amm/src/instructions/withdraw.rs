use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(mut)]
    user: Signer<'info>,
}

impl<'info> Withdraw<'info> {
    pub fn handle_withdraw(&mut self, amount: u64, min_token_x: u64, min_token_y: u64) -> Result<()> {
        msg!("Withdraw Instruction Init");
        Ok(())
    }
}