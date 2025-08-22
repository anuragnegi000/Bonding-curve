use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount, Transfer,transfer};
use anchor_spl::associated_token::AssociatedToken;

use crate::states::bonding_curve::BondingCurve;



#[derive(Accounts)]
pub struct InitializeBondingCurve<'info>{
    #[account(mut)]
    pub signer:Signer<'info>,

    #[account(mut)]
    pub token_mint:Account<'info,Mint>,

    #[account(
        init,
        space=8+32,
        payer=signer,
        seeds=[b"vault".as_ref(),token_mint.key().as_ref()],
        bump
    )]
    pub reserve_vault:Account<'info,TokenAccount>,

    pub collateral_mint:Account<'info,Mint>,

    #[account(
        mut,
        associated_token::mint=token_mint,
        associated_token::authority=signer
    )]
    pub user_collateral_ata:Account<'info,TokenAccount>,

    #[account(
        init,
        payer=signer,
        mint::decimals=9,
        mint::authority=bonding_curve,
    )]
    pub bonding_curve_mint:Account<'info,Mint>,

    #[account(
        init,
        space=8,
        payer=signer,
        seeds=[b"fee-vault".as_ref(),token_mint.key().as_ref()],
        bump
    )]
    pub fee_vault:Account<'info,TokenAccount>,

    #[account(
        mut,
        associated_token::mint=token_mint,
        associated_token::authority=signer
    )]
    pub user_token_ata:Account<'info,TokenAccount>,

    #[account(
        init,
        payer=signer,
        space=8+32+32+32+8+8+8+1,
        seeds=[b"bonding-curve".as_ref(),token_mint.key().as_ref()],
        bump
    )]
    pub bonding_curve:Account<'info,BondingCurve>,
    
    pub associated_token_program:Program<'info,AssociatedToken>,
    pub token_program:Program<'info,Token>,
    pub system_program:Program<'info,System>,
    

}


pub fn initialize_bonding_curve(ctx:Context<InitializeBondingCurve>,fee_percentage:u64,solAmount:u64)->Result<()>{

    let bonding_curve=&mut ctx.accounts.bonding_curve;

    bonding_curve.authority=ctx.accounts.signer.key();
    bonding_curve.token_mint=ctx.accounts.token_mint.key();
    bonding_curve.reserve_vault=ctx.accounts.reserve_vault.key();
    bonding_curve.generated_fees=0;
    bonding_curve.fee_percentage=fee_percentage;
    bonding_curve.migrated=false;

    transfer(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer{
                from:ctx.accounts.user_collateral_ata.to_account_info(),
                to:ctx.accounts.reserve_vault.to_account_info(),
                authority:ctx.accounts.signer.to_account_info()
            },
        ),
        solAmount
    )?;

    Ok(())
}