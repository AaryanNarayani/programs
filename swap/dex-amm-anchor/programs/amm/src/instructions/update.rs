use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct Update<'info> {
    user: Signer<'info>,
}

impl <'info> Update<'info> {
    pub fn handle_update(&mut self) -> Result<()> {
        Ok(())
    }
}