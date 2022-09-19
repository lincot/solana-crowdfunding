use crate::{config::*, state::*};
use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token};
use core::mem::size_of;

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init,
        payer = platform_authority,
        seeds = [b"platform"],
        bump,
        space = 8 + size_of::<Platform>(),
    )]
    platform: AccountLoader<'info, Platform>,
    #[account(mut)]
    platform_authority: Signer<'info>,
    #[account(
        init,
        payer = platform_authority,
        seeds = [b"fee_vault"],
        bump,
        space = 8 + size_of::<Vault>(),
    )]
    fee_vault: AccountLoader<'info, Vault>,
    #[account(
        init,
        payer = platform_authority,
        seeds = [b"sol_vault"],
        bump,
        space = 8 + size_of::<Vault>(),
    )]
    sol_vault: AccountLoader<'info, Vault>,
    #[account(
        init,
        payer = platform_authority,
        seeds = [b"chrt_mint"],
        bump,
        mint::authority = platform,
        mint::decimals = CHRT_DECIMALS,
    )]
    chrt_mint: Account<'info, Mint>,
    rent: Sysvar<'info, Rent>,
    token_program: Program<'info, Token>,
    system_program: Program<'info, System>,
}

pub fn initialize(
    ctx: Context<Initialize>,
    incentive_cooldown: u32,
    incentive_amount: u64,
    platform_fee_num: u64,
    platform_fee_denom: u64,
    fee_exemption_limit: u64,
    liquidation_limit: u64,
) -> Result<()> {
    if cfg!(production) {
        require_eq!(platform_fee_num, PLATFORM_FEE_NUM);
        require_eq!(platform_fee_denom, PLATFORM_FEE_DENOM);
    }

    let platform = &mut ctx.accounts.platform.load_init()?;
    platform.bump = *ctx.bumps.get("platform").unwrap();
    platform.bump_chrt_mint = *ctx.bumps.get("chrt_mint").unwrap();
    platform.authority = ctx.accounts.platform_authority.key();
    platform.incentive_cooldown = incentive_cooldown;
    platform.incentive_amount = incentive_amount;
    platform.platform_fee_num = platform_fee_num;
    platform.platform_fee_denom = platform_fee_denom;
    platform.fee_exemption_limit = fee_exemption_limit;
    platform.liquidation_limit = liquidation_limit;

    ctx.accounts.fee_vault.load_init()?.bump = *ctx.bumps.get("fee_vault").unwrap();
    ctx.accounts.sol_vault.load_init()?.bump = *ctx.bumps.get("sol_vault").unwrap();

    emit!(InitializeEvent {});

    Ok(())
}

#[event]
struct InitializeEvent {}
