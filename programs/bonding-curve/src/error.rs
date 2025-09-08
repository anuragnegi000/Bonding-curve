use anchor_lang::prelude::*;
#[error_code]
pub enum BondingCurveError{
    #[msg("Insufficient sol in bonding curve")]
    InsufficientSolInBondingCurve,
    #[msg("High Slippage")]
    SlippageTooHigh,
    #[msg("Already Migrated")]
    AlreadyMigrated,
    #[msg("Invalid fee percentage")]
    InvalidFeePercentage
}
