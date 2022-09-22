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
    reward_cooldown: u32,
    reward_amount: u64,
    fee_basis_points: u16,
    fee_exemption_limit: u64,
    liquidation_limit: u64,
) -> Result<()> {
    let platform = &mut ctx.accounts.platform.load_init()?;
    platform.authority = ctx.accounts.platform_authority.key();
    platform.reward_cooldown = reward_cooldown;
    platform.reward_amount = reward_amount;
    platform.fee_basis_points = fee_basis_points;
    platform.fee_exemption_limit = fee_exemption_limit;
    platform.liquidation_limit = liquidation_limit;

    Ok(())
}
