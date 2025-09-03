use anchor_lang::prelude::*;
use anchor_spl::token::{Transfer,Token,TokenAccount};


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
    let cpi_accounts=Transfer{
        from:ctx.accounts.user_token_ata.to_account_info(),
        to:ctx.accounts.bonding_curve_ata.to_account_info(),
        authority:ctx.accounts.seller.to_account_info()
    };
    let cpi_program=ctx.accounts.token_account.to_account_info();
    let cpi_ctx=CpiContext::new(cpi_program,cpi_accounts);
    token::transfer(cpi_ctx,tokens_to_sell)?;

    **ctx.accounts.reserve_vault.to_account_info().try_borrow_mut_lamports()?-=sol_out;
    **ctx.accounts.seller.to_account_info().try_borrow_mut_lamports()?+=sol_out;

    Ok(())
}