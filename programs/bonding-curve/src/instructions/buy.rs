use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::{self, transfer, Mint, MintTo, Token, TokenAccount, Transfer};
use crate::error::BondingCurveError;
use crate::states::bonding_curve::BondingCurve;
use crate::{calculate_tokens_out,calculate_fees};

#[derive(Accounts)]
pub struct BuyToken<'info>{
    #[account(mut)]
    pub buyer:Signer<'info>,

    #[account(
        mut,
        seeds=[b"bonding-curve".as_ref(),token_mint.key().as_ref()],
        bump=bonding_curve.bump
    )]
    pub bonding_curve:Account<'info,BondingCurve>,

    #[account(mut)]
    pub token_mint:Account<'info,Mint>,

    #[account(
        init,
        payer=buyer,
        associated_token::mint=token_mint,
        associated_token::authority=buyer
    )]
    pub user_token_ata:Account<'info,TokenAccount>,

    #[account(
        mut,
        seeds=[b"fee-vault".as_ref(),token_mint.key().as_ref()],
        bump
    )]
    pub fee_vault:Account<'info,TokenAccount>,

    #[account(
        mut,
        seeds=[b"vault".as_ref(),token_mint.key().as_ref()],
        bump=bonding_curve.vault_bump
    )]
    pub reserve_vault:Account<'info,TokenAccount>,

    #[account(
        mut,
        associated_token::mint=token_mint,
        associated_token::authority=bonding_curve
    )]
    pub bonding_curve_ata:Account<'info,TokenAccount>,

    pub token_program:Program<'info,Token>,
    pub associated_token_program:Program<'info,AssociatedToken>,
    pub system_program:Program<'info,System>
}

pub fn buy_token(ctx:Context<BuyToken>,sol_amount:u64,min_tokens_out:u64,fee_bump:u8)->Result<()>{
    
    let bonding_curve=&mut ctx.accounts.bonding_curve;
    bonding_curve.fee_bump=fee_bump;
    
    let tokens_out=calculate_tokens_out(sol_amount,bonding_curve.virtual_sol_reserves,bonding_curve.virtual_token_reserves)?;

    **ctx.accounts.buyer.to_account_info().try_borrow_mut_lamports()?-=sol_amount;
    **ctx.accounts.reserve_vault.to_account_info().try_borrow_mut_lamports()?+=sol_amount;

    require!(tokens_out>=min_tokens_out,BondingCurveError::SlippageTooHigh);
    let bonding_curve=&mut ctx.accounts.bonding_curve;

    let fees=calculate_fees(sol_amount)?;
    bonding_curve.generated_fees+=fees;


    let cpi_accounts=Transfer{
        from:ctx.accounts.reserve_vault.to_account_info(),
        to:ctx.accounts.user_token_ata.to_account_info(),
        authority:bonding_curve.to_account_info()
    };
    let token_mint=&ctx.accounts.token_mint.to_account_info().key();
    let seeds: &[&[u8]] = &[
        b"bonding-curve".as_ref(),
        token_mint.as_ref(),
        &[bonding_curve.bump],
    ];
    let signer_seeds: &[&[&[u8]]] = &[seeds];
    let cpi_program = ctx.accounts.token_program.to_account_info();
    let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer_seeds);
    transfer(cpi_ctx,tokens_out)?;
    bonding_curve.virtual_sol_reserves+=sol_amount;
    bonding_curve.virtual_token_reserves-=tokens_out;
    bonding_curve.generated_fees+=fees;
    Ok(())
}