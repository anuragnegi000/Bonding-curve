use anchor_lang::prelude::*;
use anchor_spl::{associated_token::AssociatedToken, token::{self, transfer, Mint, SyncNative, Token, TokenAccount, TransferChecked,transfer_checked}};
use crate::{calculate_fees, states::bonding_curve::BondingCurve};
use crate::{calculate_sol_out,error::BondingCurveError};
use anchor_lang::system_program::{transfer as system_transfer,Transfer as SystemTransfer};


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

    #[account(address = anchor_spl::token::spl_token::native_mint::id())]
    pub wsol_mint: Account<'info, Mint>,

   #[account(mut)]
   pub token_mint:Account<'info,Mint>,

   #[account(
    mut,
    token::mint=wsol_mint,
    token::authority=bonding_curve,
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
    let cpi_accounts=TransferChecked{
        from:ctx.accounts.user_token_ata.to_account_info(),
        mint:ctx.accounts.token_mint.to_account_info(),
        to:ctx.accounts.bonding_curve_ata.to_account_info(),
        authority:ctx.accounts.seller.to_account_info()
    };
    let cpi_program=ctx.accounts.token_program.to_account_info();
    let cpi_ctx=CpiContext::new(cpi_program,cpi_accounts);
    transfer_checked(cpi_ctx,tokens_to_sell,6)?;
    let fees=calculate_fees(sol_out)?;
    bonding_curve.generated_fees+=fees;

    let cpi_accounts_for_rv=SystemTransfer{
        from:ctx.accounts.reserve_vault.to_account_info(),
        to:ctx.accounts.seller.to_account_info()
    };

    let system_cpi_ctx=CpiContext::new(ctx.accounts.system_program.to_account_info(),cpi_accounts_for_rv);
    system_transfer(system_cpi_ctx, sol_out)?;

    bonding_curve.virtual_sol_reserves-=sol_out;
    bonding_curve.virtual_token_reserves+=tokens_to_sell;

    Ok(())
}