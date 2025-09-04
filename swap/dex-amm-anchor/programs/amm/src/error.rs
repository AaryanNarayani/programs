use anchor_lang::prelude::*;

#[error_code]
pub enum PoolConfigError{
    #[msg("Pool is not initialized")]
    PoolNotInitialized,
    #[msg("Invalid Owner")]
    InvalidOwner,
    #[msg("Invalid Amount")]
    InvalidAmount,
    #[msg("Pool is locked")]
    PoolLocked,
}

#[error_code]
pub enum AmmDexError{
    #[msg("Invalid Swap")]
    InvalidSwap,
    #[msg("Slippage Tolerance Exceeded")]
    SlippageToleranceExceeded,
    #[msg("Invalid Authority")]
    InvalidAuthority,

}