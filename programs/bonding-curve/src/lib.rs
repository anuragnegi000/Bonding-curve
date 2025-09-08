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
    use super::*;

    pub fn initialize_bonding_curve(ctx: Context<InitializeBondingCurve>, fee_percentage: u64, sol_amount: u64,bump:u8,min_tokens_out:u64,vault_bump:u8) -> Result<()> {
        instructions::initialize_bonding_curve(ctx,fee_percentage,sol_amount,min_tokens_out,bump,vault_bump)?;
        Ok(())
    }
    // pub fn 
}
