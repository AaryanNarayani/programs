use anchor_lang::prelude::*;
use crate::{error::AmmDexError, state::PoolConfig};

#[derive(Accounts)]
pub struct Update<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        mut,
        seeds = [b"pool-config", pool_config.owner.as_ref().unwrap().as_ref()],
        bump = pool_config.pool_config_bump,
    )]
    pub pool_config: Account<'info, PoolConfig>,
}

impl<'info> Update<'info> {
    pub fn handle_update(&mut self, lock: bool) -> Result<()> {
        if self.pool_config.owner.is_none() {
            return Err(crate::error::PoolConfigError::PoolNotInitialized.into());
        }
        require!(
            self.pool_config.owner == Some(self.user.key()),
            AmmDexError::InvalidAuthority
        );
        self.pool_config.is_locked = lock;
        Ok(())
    }
}