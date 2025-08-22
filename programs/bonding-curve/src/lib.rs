use anchor_lang::prelude::*;

declare_id!("8Nz3qiKNPeP4E6qMNH8g1xTRW2PPo8h1qGNY3BWyvYDs");

mod instructions;
mod states;

use instructions::*;
use states::*;

#[program]
pub mod bonding_curve {
    use super::*;

    pub fn initialize_bonding_curve(ctx: Context<InitializeBondingCurve>, fee_percentage: u64, sol_amount: u64) -> Result<()> {
        instructions::initialize_bonding_curve(ctx, fee_percentage, sol_amount)?;
        Ok(())
    }
}
