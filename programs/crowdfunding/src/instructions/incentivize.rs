use crate::{error::*, state::*};
use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, MintTo, Token, TokenAccount};

#[derive(Accounts)]
pub struct Incentivize<'info> {
    #[account(mut, seeds = [b"platform"], bump = platform.bump)]
    platform: Account<'info, Platform>,
    #[account(address = platform.authority)]
    platform_authority: Signer<'info>,
    #[account(mut, seeds = [b"chrt_mint"], bump = platform.bump_chrt_mint)]
    chrt_mint: Account<'info, Mint>,
    token_program: Program<'info, Token>,
}

fn mint_chrt_to_top_donor<'info>(
    ctx: &Context<'_, '_, '_, 'info, Incentivize<'info>>,
    donor_chrt: &Account<'info, TokenAccount>,
) -> Result<()> {
    let signer: &[&[&[u8]]] = &[&[b"platform", &[ctx.accounts.platform.bump]]];
    let cpi_ctx = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        MintTo {
            mint: ctx.accounts.chrt_mint.to_account_info(),
            to: donor_chrt.to_account_info(),
            authority: ctx.accounts.platform.to_account_info(),
        },
        signer,
    );
    token::mint_to(cpi_ctx, ctx.accounts.platform.incentive_amount)
}

pub fn incentivize<'info>(ctx: Context<'_, '_, '_, 'info, Incentivize<'info>>) -> Result<()> {
    let now = Clock::get()?.unix_timestamp as _;

    if now - ctx.accounts.platform.last_incentive_ts < ctx.accounts.platform.incentive_cooldown {
        return err!(CrowdfundingError::IncentiveCooldown);
    }
    ctx.accounts.platform.last_incentive_ts = now;

    let mut accs = ctx.remaining_accounts.iter();

    for d in &ctx.accounts.platform.platform_top {
        let donor_chrt = Account::<TokenAccount>::try_from(
            accs.next().ok_or(CrowdfundingError::CHRTNotProvided)?,
        )?;
        if donor_chrt.owner != d.donor {
            return err!(ConstraintTokenOwner);
        }
        mint_chrt_to_top_donor(&ctx, &donor_chrt)?;
    }

    ctx.accounts.platform.seasonal_top.clear();

    emit!(IncentivizeEvent {});

    Ok(())
}

#[event]
struct IncentivizeEvent {}
