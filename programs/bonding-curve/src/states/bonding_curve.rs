use anchor_lang::prelude::*;

#[account]
pub struct BondingCurve {
    pub authority: Pubkey,
    pub token_mint: Pubkey,
    pub reserve_vault: Pubkey, // this will store all the solana which will later be migrated to the target pool
    pub generated_fees: u64,
    pub fee_percentage: u64,
    pub total_supply: u64,  
    pub migrated: bool,
}