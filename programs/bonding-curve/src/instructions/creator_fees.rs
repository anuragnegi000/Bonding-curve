use anchor_spl::{associated_token::AssociatedToken, token::{transfer, Mint, Token, TokenAccount, Transfer}};
use anchor_lang::prelude::*;
use crate::error::BondingCurveError;
use crate::states::bonding_curve::BondingCurve;

#[derive(Accounts)]
pub struct WithdrawFees<'info>{
    #[account(mut)]
    pub authority:Signer<'info>,

    #[account(mut,)]
    pub token_mint:Account<'info,Mint>,

    #[account(
        mut,
        seeds=[b"bonding-curve".as_ref(),token_mint.key().as_ref()],
        bump=bonding_curve.bump,
    )]
    pub bonding_curve:Account<'info,BondingCurve>,

    #[account(
        mut,
        seeds=[b"fee-vault".as_ref(),token_mint.key().as_ref()],
        bump=bonding_curve.fee_bump,
    )]
    pub fee_vault:Account<'info,TokenAccount>,
     
    pub token_program:Program<'info,Token>,
    pub associated_token_program:Program<'info,AssociatedToken>,
    pub system_program:Program<'info,System>
}
pub fn withdraw_fees(ctx:Context<WithdrawFees>)->Result<()>{
    let bonding_curve=&mut ctx.accounts.bonding_curve;
    let fees=bonding_curve.generated_fees;
    require!(fees>0,BondingCurveError::NoFeesToWithdraw);
    **ctx.accounts.fee_vault.to_account_info().try_borrow_mut_lamports()?-=fees;
    **ctx.accounts.authority.to_account_info().try_borrow_mut_lamports()?+=fees;
    bonding_curve.generated_fees=0;
    Ok(())
}