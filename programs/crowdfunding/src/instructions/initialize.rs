use crate::{state::*, CHRT_DECIMALS};
use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token};

#[derive(Accounts)]
#[instruction(campaigns_capacity: u16)]
pub struct Initialize<'info> {
    #[account(
        init,
        payer = platform_authority,
        seeds = [b"platform"],
        bump,
        space = 8 + Platform::space(campaigns_capacity),
    )]
    platform: Account<'info, Platform>,
    #[account(mut)]
    platform_authority: Signer<'info>,
    /// CHECK:
    #[account(
        init,
        payer = platform_authority,
        seeds = [b"fee_vault"],
        bump,
        space = 0,
        owner = system_program.key(),
    )]
    fee_vault: UncheckedAccount<'info>,
    /// CHECK:
    #[account(
        init,
        payer = platform_authority,
        seeds = [b"sol_vault"],
        bump,
        space = 0,
        owner = system_program.key(),
    )]
    sol_vault: UncheckedAccount<'info>,
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

#[allow(clippy::too_many_arguments)]
pub fn initialize(
    ctx: Context<Initialize>,
    active_campaigns_capacity: u16,
    incentive_cooldown: u32,
    incentive_amount: u64,
    platform_fee_num: u64,
    platform_fee_denom: u64,
    fee_exemption_limit: u64,
    liquidation_limit: u64,
) -> Result<()> {
    ctx.accounts.platform.bump = *ctx.bumps.get("platform").unwrap();
    ctx.accounts.platform.bump_fee_vault = *ctx.bumps.get("fee_vault").unwrap();
    ctx.accounts.platform.bump_sol_vault = *ctx.bumps.get("sol_vault").unwrap();
    ctx.accounts.platform.bump_chrt_mint = *ctx.bumps.get("chrt_mint").unwrap();
    ctx.accounts.platform.authority = ctx.accounts.platform_authority.key();
    ctx.accounts.platform.active_campaigns_capacity = active_campaigns_capacity;
    ctx.accounts.platform.incentive_cooldown = incentive_cooldown;
    ctx.accounts.platform.incentive_amount = incentive_amount;
    ctx.accounts.platform.platform_fee_num = platform_fee_num;
    ctx.accounts.platform.platform_fee_denom = platform_fee_denom;
    ctx.accounts.platform.fee_exemption_limit = fee_exemption_limit;
    ctx.accounts.platform.liquidation_limit = liquidation_limit;

    emit!(InitializeEvent {});

    Ok(())
}

#[event]
struct InitializeEvent {}
