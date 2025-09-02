use anchor_lang::prelude::*;
use anchor_spl::token::{Transfer,Token,TokenAccount};


#[derive(Accounts)]
pub struct SellToken<'info>{
    #[account]
    pub seller:Signer<'info>,

    #[account(
        mut,
        seeds=[b"bonding-curve",token_mint.key().as_ref()],
        bump=bonding_curve.bump
    )]
    pub bonding_curve:Account<'info,BondingCurve>, 

   #[account(mut)]
   pub token_mint:Account<'info,Mint>,

   #[account(
    mut,
    seeds=[b"vault",token_mint.key().as_ref()],
    bump=bonding_curve.vault_bump
   )]
   pub reserve_vault:Account<'info,TokenAccount>,

   #[account(
    mut,
    payer=seller,
    associated_token::mint=token_mint,
    associated_token::authority=seller
   )]
   pub user_token_ata:Account<'info,TokenAccount>,

   pub token_account:Program<'info,Token>,
   pub associated_token_program:Program<'info,AssociatedToken>,
   pub system_program:Program<'info,System>
}

pub fn sell_token(ctx:SellToken,min_sol_out:u64,tokens_to_sell:u64)->Result<()>{
    let mut bonding_curve=&mut ctx.accounts.bonding_curve;
    let sol_out=calculate_sol_out(tokens_to_sell)?;
    require(sol_out>=mint_sol_out,CustomError::SlippageTooHigh);
    Ok(())
}