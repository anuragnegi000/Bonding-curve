use anchor_lang::prelude::*;
use anchor_spl::{associated_token::AssociatedToken, token::{Mint, Token, TokenAccount,transfer, Transfer}};
use crate::states::bonding_curve::BondingCurve;
use crate::{calculate_sol_out,error::BondingCurveError};


#[derive(Accounts)]
pub struct SellToken<'info>{
    #[account(mut)]
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
    associated_token::mint=token_mint,
    associated_token::authority=bonding_curve
   )]
   pub bonding_curve_ata:Account<'info,TokenAccount>,

   #[account(
    init,
    payer=seller,
    associated_token::mint=token_mint,
    associated_token::authority=seller
   )]
   pub user_token_ata:Account<'info,TokenAccount>,

   pub token_program:Program<'info,Token>,
   pub associated_token_program:Program<'info,AssociatedToken>,
   pub system_program:Program<'info,System>
}

pub fn sell_token(ctx:Context<SellToken>,min_sol_out:u64,tokens_to_sell:u64)->Result<()>{
    let mut bonding_curve=&mut ctx.accounts.bonding_curve;
    let sol_out=calculate_sol_out(tokens_to_sell,bonding_curve.virtual_sol_reserves,bonding_curve.virtual_token_reserves)?;
    require!(sol_out>=min_sol_out,BondingCurveError::SlippageTooHigh);
    let cpi_accounts=Transfer{
        from:ctx.accounts.user_token_ata.to_account_info(),
        to:ctx.accounts.bonding_curve_ata.to_account_info(),
        authority:ctx.accounts.seller.to_account_info()
    };
    let cpi_program=ctx.accounts.token_program.to_account_info();
    let cpi_ctx=CpiContext::new(cpi_program,cpi_accounts);
    transfer(cpi_ctx,tokens_to_sell)?;

    **ctx.accounts.reserve_vault.to_account_info().try_borrow_mut_lamports()?-=sol_out;
    **ctx.accounts.seller.to_account_info().try_borrow_mut_lamports()?+=sol_out;
    

    Ok(())
}