use anchor_lang::prelude::*;

declare_id!("8Nz3qiKNPeP4E6qMNH8g1xTRW2PPo8h1qGNY3BWyvYDs");

mod instructions;
mod states;
mod constant;
mod utils;
mod error;

pub use utils::*;
pub use error::*;

use instructions::*;
use states::*;


#[program]
pub mod bonding_curve {
    use anchor_spl::token;

    use super::*;

    pub fn initialize_bonding_curve(ctx: Context<InitializeBondingCurve>, fee_percentage: u64, sol_amount: u64,bump:u8,min_tokens_out:u64,vault_bump:u8) -> Result<()> {
        instructions::initialize_bonding_curve(ctx,fee_percentage,sol_amount,min_tokens_out,bump,vault_bump)?;
        Ok(())
    }
    pub fn buy_token(ctx:Context<BuyToken>,sol_amount: u64,min_tokens_out:u64,fee_bump:u8)->Result<()>{
        instructions::buy_token(ctx,sol_amount,min_tokens_out,fee_bump)?;
        Ok(())
    }
    pub fn sell_token(ctx:Context<SellToken>,min_sol_out:u64,tokens_to_sell:u64)->Result<()>{
        instructions::sell_token(ctx,min_sol_out,tokens_to_sell)?;
        Ok(())
    }
    pub fn withdraw_fees(ctx:Context<WithdrawFees>)->Result<()>{
        instructions::withdraw_fees(ctx)?;
        Ok(())
    }
    
    pub fn migrate_to_raydium(ctx: Context<MigrateToRaydium>) -> Result<()> {
        instructions::migrate_to_raydium(ctx)?;
        Ok(())
    }
}

#[event]
pub struct MigrationReadyEvent {
    pub token_mint: Pubkey,
    pub sol_reserves: u64,
    pub token_reserves: u64,
}
