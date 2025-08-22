use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount, Mint};

use crate::states::bonding_curve::BondingCurve;


pub struct InitializeBondingCurve<'info>{
    pub signer:Signer<'info>,

    pub token_mint:Account<'info,Mint>,

    #[account(
        init,
        payer=signer,
        seeds=[b"vault".as_ref(),token_mint.key().as_ref()],
        bump
    )]
    pub reserve_vault:Account<'info,TokenAccount>,

    #[account(
        init,
        payer=signer,
        seeds=[b"fee-vault".as_ref().token_mint.key().as_ref()]
        bump
    )]
    pub fee_vault:Account<'info,TokenAccount>,
    #[account(
        init,
        payer=signer,
        seeds=[b"bonding-curve".as_ref(),token_mint.key().as_ref()],
        bump
    )]
    pub bonding_curve:Account<'info,BondingCurve>,
    
    pub token_program:Program<'info,System>,
    pub system_program:Program<'info,System>,
    

}


pub fn initialize_bonding_curve<InitializeBondingCurve>()->Result<()>{
    

    
    Ok(())
}