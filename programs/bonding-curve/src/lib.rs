use anchor_lang::prelude::*;

declare_id!("8Nz3qiKNPeP4E6qMNH8g1xTRW2PPo8h1qGNY3BWyvYDs");

#[program]
pub mod bonding_curve {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
