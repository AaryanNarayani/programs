use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct PoolConfig {
    pub seeds: u64,
    pub lp_fee: u64,
    pub protocol_fee: u64,
    pub lp_bump: u8,
    pub pool_config_bump: u8,
    pub lp_token_mint: Pubkey,
    pub token_x_mint: Pubkey,
    pub token_y_mint: Pubkey,
    pub owner: Option<Pubkey>,
}