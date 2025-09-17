use anchor_lang::prelude::*;
use anchor_spl::{associated_token::AssociatedToken, token::{mint_to, transfer_checked, Mint, MintTo, Token, TokenAccount, TransferChecked}};
use raydium_cpmm_cpi::{
    cpi,
    program::RaydiumCpmm,
    states::{AmmConfig,OBSERVATION_SEED,POOL_LP_MINT_SEED, POOL_SEED, POOL_VAULT_SEED}
};
use anchor_lang::system_program::{transfer as system_transfer, Transfer as SystemTransfer};
use crate::states::bonding_curve::BondingCurve;
use crate::error::BondingCurveError;

#[derive(Accounts)]
pub struct MigrateToRaydium<'info>{
    pub cp_swap_program: Program<'info, RaydiumCpmm>,

    #[account(mut)]
    pub creator: Signer<'info>,

    // Add bonding curve account to access real reserves
    #[account(
        mut,
        seeds = [b"bonding-curve".as_ref(), token_mint.key().as_ref()],
        bump = bonding_curve.bump,
        constraint = bonding_curve.real_sol_reserves >= 40_000_000_000 @ BondingCurveError::InsufficientSolInBondingCurve,
        constraint = !bonding_curve.migrated @ BondingCurveError::AlreadyMigrated,
    )]
    pub bonding_curve: Account<'info, BondingCurve>,

    // Original token mint (your project token)
    #[account(mut)]
    pub token_mint: Account<'info, Mint>,

    // wSOL mint
    #[account(address = anchor_spl::token::spl_token::native_mint::id())]
    pub wsol_mint: Account<'info, Mint>,

    // Bonding curve's token account (holds remaining tokens)
    #[account(
        mut,
        associated_token::mint = token_mint,
        associated_token::authority = bonding_curve,
    )]
    pub bonding_curve_ata: Account<'info, TokenAccount>,

    // Bonding curve's SOL vault (holds collected SOL)
    #[account(
        mut,
        seeds = [b"vault".as_ref(), token_mint.key().as_ref()],
        bump = bonding_curve.vault_bump,
    )]
    pub reserve_vault: SystemAccount<'info>,

    pub amm_config: Box<Account<'info, AmmConfig>>,

    /// CHECK: Authority signer for the migration
    #[account(
        seeds = [raydium_cpmm_cpi::AUTH_SEED.as_bytes()],
        seeds::program = cp_swap_program,
        bump,
    )]
    pub authority: UncheckedAccount<'info>,

    /// CHECK: Pool state for the migration
    #[account(
        mut,
        seeds = [
            POOL_SEED.as_bytes(),
            amm_config.key().as_ref(),
            token_0_mint.key().as_ref(),
            token_1_mint.key().as_ref(),
        ],
        seeds::program = cp_swap_program,
        bump,
    )]
    pub pool_state: UncheckedAccount<'info>,

    // For Raydium, we need to determine which is token_0 and token_1
    // wSOL vs your token (sorted by pubkey)
    #[account(
        constraint = token_0_mint.key() < token_1_mint.key(),
    )]
    pub token_0_mint: Account<'info, Mint>,

    pub token_1_mint: Account<'info, Mint>,

    /// CHECK: lp_mint for the migration
     #[account(
        mut,
        seeds = [
            POOL_LP_MINT_SEED.as_bytes(),
            pool_state.key().as_ref(),
        ],
        seeds::program = cp_swap_program,
        bump,
    )]
    pub lp_mint: UncheckedAccount<'info>,

    #[account(
        mut,
        token::mint = token_0_mint,
        token::authority = creator,
    )]
    pub creator_token_0: Account<'info, TokenAccount>,

    #[account(
        mut,
        token::mint = token_1_mint,
        token::authority = creator,
    )]
    pub creator_token_1: Account<'info, TokenAccount>,

    /// CHECK:  creator_lp_token for the migration
    #[account(mut)]
    pub creator_lp_token: UncheckedAccount<'info>,  

    /// CHECK:  token_0_vault for the migration
    #[account(
        mut,
        seeds = [
            POOL_VAULT_SEED.as_bytes(),
            pool_state.key().as_ref(),
            token_0_mint.key().as_ref()
        ],
        seeds::program = cp_swap_program,
        bump,
    )]
    pub token_0_vault: UncheckedAccount<'info>,



    /// CHECK: token_1_vault for the migration
    #[account(
        mut,
        seeds = [
            POOL_VAULT_SEED.as_bytes(),
            pool_state.key().as_ref(),
            token_1_mint.key().as_ref()
        ],
        seeds::program = cp_swap_program,
        bump,
    )]
    pub token_1_vault: UncheckedAccount<'info>,

    #[account(
        mut,
        address= raydium_cpmm_cpi::create_pool_fee_reveiver::ID,
    )]
    pub create_pool_fee: Account<'info, TokenAccount>,

    /// CHECK:  observation state for the migration
    #[account(
        mut,
        seeds = [
            OBSERVATION_SEED.as_bytes(),
            pool_state.key().as_ref(),
        ],
        seeds::program = cp_swap_program,
        bump,
    )]
    pub observation_state: UncheckedAccount<'info>,

    /// Program to create mint account and mint tokens
    pub token_program: Program<'info, Token>,
    /// Spl token program or token program 2022
    pub token_0_program: Program<'info, Token>,
    /// Spl token program or token program 2022
    pub token_1_program: Program<'info, Token>,
    /// Program to create an ATA for receiving position NFT
    pub associated_token_program: Program<'info, AssociatedToken>,
    /// To create a new program account
    pub system_program: Program<'info, System>,
    /// Sysvar for program account
    pub rent: Sysvar<'info, Rent>,
}

pub fn migrate(ctx: Context<MigrateToRaydium>) -> Result<()> {
    let bonding_curve = &mut ctx.accounts.bonding_curve;
    
    // 1. Validate migration conditions
    require!(bonding_curve.real_sol_reserves >= 40_000_000_000, BondingCurveError::InsufficientSolInBondingCurve);
    require!(!bonding_curve.migrated, BondingCurveError::AlreadyMigrated);
    
    // 2. Calculate migration amounts
    let sol_amount = bonding_curve.real_sol_reserves; // Actual SOL collected
    let existing_tokens = bonding_curve.real_token_reserves; // Remaining tokens in bonding curve
    let dex_tokens = 627_000_000; // Additional tokens for DEX (1B - 373M)
    let total_token_amount = existing_tokens + dex_tokens;
    
    msg!("Migrating to Raydium:");
    msg!("SOL amount: {}", sol_amount);
    msg!("Existing tokens: {}", existing_tokens);
    msg!("DEX tokens: {}", dex_tokens);
    msg!("Total token amount: {}", total_token_amount);
    
    // 3. Mint additional 627M tokens for DEX liquidity
    let token_mint = ctx.accounts.token_mint.to_account_info().key();
    let seeds = &[
        b"bonding-curve".as_ref(),
        token_mint.as_ref(),
        &[bonding_curve.bump],
    ];
    let signer_seeds = &[&seeds[..]];
    
    // Mint additional DEX tokens to bonding curve ATA
    let mint_to_bonding_curve = MintTo {
        mint: ctx.accounts.token_mint.to_account_info(),
        to: ctx.accounts.bonding_curve_ata.to_account_info(),
        authority: bonding_curve.to_account_info(),
    };
    let cpi_ctx_mint = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        mint_to_bonding_curve,
        signer_seeds
    );
    mint_to(cpi_ctx_mint, dex_tokens)?;
    
    // 4. Transfer all tokens to creator for pool creation
    let transfer_tokens = TransferChecked {
        from: ctx.accounts.bonding_curve_ata.to_account_info(),
        mint: ctx.accounts.token_mint.to_account_info(),
        to: ctx.accounts.creator_token_1.to_account_info(), // creator's token account
        authority: bonding_curve.to_account_info(),
    };
    let cpi_ctx_transfer = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        transfer_tokens,
        signer_seeds
    );
    transfer_checked(cpi_ctx_transfer, total_token_amount, 6)?;
    
    // 5. Transfer SOL from vault to creator's wSOL account
    // First transfer SOL from vault to creator
    let sol_transfer = SystemTransfer {
        from: ctx.accounts.reserve_vault.to_account_info(),
        to: ctx.accounts.creator.to_account_info(),
    };
    let vault_seeds = &[
        b"vault".as_ref(),
        token_mint.as_ref(),
        &[bonding_curve.vault_bump],
    ];
    let vault_signer_seeds = &[&vault_seeds[..]];
    let cpi_ctx_sol = CpiContext::new_with_signer(
        ctx.accounts.system_program.to_account_info(),
        sol_transfer,
        vault_signer_seeds
    );
    system_transfer(cpi_ctx_sol, sol_amount)?;
    
    // 6. Create Raydium pool with the migrated liquidity
    // Determine token order for Raydium (must be sorted by pubkey)
    let (token_0_amount, token_1_amount) = if ctx.accounts.wsol_mint.key() < ctx.accounts.token_mint.key() {
        // wSOL is token_0, project token is token_1
        (sol_amount, total_token_amount)
    } else {
        // project token is token_0, wSOL is token_1
        (total_token_amount, sol_amount)
    };
    
    let cpi_accounts = cpi::accounts::Initialize {
        creator: ctx.accounts.creator.to_account_info(),
        amm_config: ctx.accounts.amm_config.to_account_info(),
        authority: ctx.accounts.authority.to_account_info(),
        pool_state: ctx.accounts.pool_state.to_account_info(),
        token_0_mint: ctx.accounts.token_0_mint.to_account_info(),
        token_1_mint: ctx.accounts.token_1_mint.to_account_info(),
        lp_mint: ctx.accounts.lp_mint.to_account_info(),
        creator_token_0: ctx.accounts.creator_token_0.to_account_info(),
        creator_token_1: ctx.accounts.creator_token_1.to_account_info(),
        creator_lp_token: ctx.accounts.creator_lp_token.to_account_info(),
        token_0_vault: ctx.accounts.token_0_vault.to_account_info(),
        token_1_vault: ctx.accounts.token_1_vault.to_account_info(),
        create_pool_fee: ctx.accounts.create_pool_fee.to_account_info(),
        observation_state: ctx.accounts.observation_state.to_account_info(),
        token_program: ctx.accounts.token_program.to_account_info(),
        token_0_program: ctx.accounts.token_0_program.to_account_info(),
        token_1_program: ctx.accounts.token_1_program.to_account_info(),
        associated_token_program: ctx.accounts.associated_token_program.to_account_info(),
        system_program: ctx.accounts.system_program.to_account_info(),
        rent: ctx.accounts.rent.to_account_info(),
    };
    let cpi_context = CpiContext::new(ctx.accounts.cp_swap_program.to_account_info(), cpi_accounts);
    cpi::initialize(cpi_context, token_0_amount, token_1_amount, 0)?; // open_time = 0 for immediate
    
    // 7. Mark as migrated and reset reserves
    bonding_curve.migrated = true;
    bonding_curve.real_sol_reserves = 0; // All SOL transferred
    bonding_curve.real_token_reserves = 0; 
    
    msg!("Migration to Raydium completed successfully!");
    
    Ok(())
}