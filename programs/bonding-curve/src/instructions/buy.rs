use anchor_lang::prelude::*;
use anchor_spl::token::{self,MintTo,Mint,Token,TokenAccount};


#[derive(Accounts)]
pub struct BuyToken<'info>{
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
        associated_tokem::mint=token_mint,
        associated_token::authority=buyer
    )]
    pub user_token_ata:Account<'info,TokenAccount>,

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

pub fn buy_token(ctx:BuyToken,sol_amount:u64,min_tokens_out:u64)->Result<()>{
    
    let fees=calculate_fees(sol_amount)?;
    bonding_curve.generated_fees+=fees;

    let tokens_out=calculate_tokens_out(sol_amount);

    **ctx.accounts.buyer.to_account_info().try_borrow_mut_lamports()?-=sol_amount;
    **ctx.accounts.reserve_vault.to_account_info().try_borrow_mut_lamports()?+=sol_amount;

    require(tokens_out>=min_tokens_out,CustomError::SlippageTooHigh);
    let bonding_curve=&mut ctx.accounts.bonding_curve;

    let cpi_accounts=MintTo{
        mint:ctx.accounts.token_mint.to_account_info(),
        to:ctx.accounts.user_token_ata.to_account_info(),
        authority:ctx.accounts.bonding_curve.to_account_info()
    };
    let signer_seeds=&[b"bonding-curve".as_ref(),ctx.accounts.token_mint.key().as_ref(),&[bonding_curve.bump]];

    let cpi_program=ctx.accounts.token_program.to_account_info();
    let cpi_ctx=CpiContext::new_with_signer(cpi_program,cpi_accounts,signer_seeds);
    token::transfer(cpi_ctx,tokens_out)?;
    bonding_curve.virtual_sol_reserves+=sol_amount;
    bonding_curve.virtual_token_reserves-=tokens_out;
    
    Ok(())
}