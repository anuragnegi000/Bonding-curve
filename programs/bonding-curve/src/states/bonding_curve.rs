use anchor_lang::prelude::*;

#[account]
pub struct BondingCurve {
    pub authority: Pubkey,
    pub token_mint: Pubkey,
    pub reserve_vault:Pubkey,
    pub virtual_token_reserves:u64,
    pub virtual_sol_reserves:u64,
    pub real_token_reserves:u64,
    pub real_sol_reserves:u64,
    pub generated_fees: u64,
    pub fee_percentage: u64,
    pub total_supply: u64,  
    pub migrated: bool,
    pub bump: u8,
    pub vault_bump:u8,
    pub fee_bump:u8,
}