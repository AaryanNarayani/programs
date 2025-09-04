use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct Swap<'info> {
    user: Signer<'info>,
}

impl <'info> Swap<'info> {
    pub fn handle_swap(&mut self) -> Result<()> {
        Ok(())
    }
}
