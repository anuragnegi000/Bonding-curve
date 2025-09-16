use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount, TransferChecked,transfer_checked,transfer, MintTo, mint_to};
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::{self,SyncNative};
use anchor_lang::system_program::{transfer as system_transfer,Transfer as SystemTransfer};

use crate::states::bonding_curve::BondingCurve;
use crate::constant::*;

use crate::{calculate_tokens_out,calculate_sol_out,error::BondingCurveError};



#[derive(Accounts)]
pub struct InitializeBondingCurve<'info>{
    #[account(mut)]
    pub signer:Signer<'info>,

    #[account(mut)]
    pub token_mint:Account<'info,Mint>,

    #[account(address = anchor_spl::token::spl_token::native_mint::id())]
    pub wsol_mint: Account<'info, Mint>,
    
    #[account(
        mut,
        seeds=[b"vault".as_ref(),token_mint.key().as_ref()],
        bump
    )]
    pub reserve_vault:SystemAccount<'info>,

    #[account(
        init,
        payer=signer,
        associated_token::mint=token_mint,
        associated_token::authority=bonding_curve
    )]
    pub bonding_curve_ata:Account<'info,TokenAccount>,

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
        space=8+32+32+32+8+8+8+8+8+8+8+1+1+1+1, // Updated space for all fields
        seeds=[b"bonding-curve".as_ref(),token_mint.key().as_ref()],
        bump
    )]
    pub bonding_curve:Account<'info,BondingCurve>,
    
    pub associated_token_program:Program<'info,AssociatedToken>,
    pub token_program:Program<'info,Token>,
    pub system_program:Program<'info,System>,
    

}


pub fn initialize_bonding_curve(ctx:Context<InitializeBondingCurve>,fee_percentage:u64,solAmount:u64,min_tokens_out:u64,bump:u8,vault_bump:u8)->Result<()>{

    let bonding_curve=&mut ctx.accounts.bonding_curve;

    bonding_curve.authority=ctx.accounts.signer.key();
    bonding_curve.token_mint=ctx.accounts.token_mint.key();
    bonding_curve.reserve_vault=ctx.accounts.reserve_vault.key();
    bonding_curve.generated_fees=0;
    bonding_curve.fee_percentage=fee_percentage;
    bonding_curve.migrated=false;
    bonding_curve.bump=bump;
    bonding_curve.vault_bump=vault_bump;
    bonding_curve.virtual_sol_reserves=VIRTUAL_SOL;
    bonding_curve.virtual_token_reserves=VIRTUAL_TOKEN_RESERVE;
    bonding_curve.real_sol_reserves=solAmount;

    let cpi_accounts_for_rv=SystemTransfer{
        from:ctx.accounts.signer.to_account_info(),
        to:ctx.accounts.reserve_vault.to_account_info()
    };

    let system_cpi_ctx=CpiContext::new(ctx.accounts.system_program.to_account_info(),cpi_accounts_for_rv);

    system_transfer(system_cpi_ctx, solAmount)?;
    

    let token_mint=ctx.accounts.token_mint.to_account_info().key();

    let seeds = &[
        b"bonding-curve".as_ref(),
        token_mint.as_ref(),
        &[ctx.bumps.bonding_curve],
    ];
    let signer_seeds = &[&seeds[..]];

    let mint_to_bonding_curve = MintTo {
        mint: ctx.accounts.token_mint.to_account_info(),
        to: ctx.accounts.bonding_curve_ata.to_account_info(),
        authority: bonding_curve.to_account_info(),
    };
    let cpi_ctx_bonding = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        mint_to_bonding_curve,
        signer_seeds
    );
    mint_to(cpi_ctx_bonding, TOKEN_FOR_SALE)?; 

    let tokens_out=crate::utils::calculate_tokens_out(solAmount,bonding_curve.virtual_sol_reserves,bonding_curve.virtual_token_reserves)?;
    require!(tokens_out>=min_tokens_out,BondingCurveError::SlippageTooHigh);
    let transfer_to_user = TransferChecked {
        from: ctx.accounts.bonding_curve_ata.to_account_info(),
        mint:ctx.accounts.token_mint.to_account_info(),
        to: ctx.accounts.user_token_ata.to_account_info(),
        authority: bonding_curve.to_account_info()
    };
    let cpi_program=ctx.accounts.token_program.to_account_info();
    let cpi_ctx=CpiContext::new_with_signer(cpi_program, transfer_to_user, signer_seeds);
    transfer_checked(cpi_ctx, tokens_out,6)?;
    bonding_curve.virtual_sol_reserves+=solAmount;
    bonding_curve.virtual_token_reserves-=tokens_out;
    bonding_curve.total_supply = TOKEN_FOR_SALE; 

    bonding_curve.real_token_reserves=TOKEN_FOR_SALE - tokens_out;
    
    Ok(())
}