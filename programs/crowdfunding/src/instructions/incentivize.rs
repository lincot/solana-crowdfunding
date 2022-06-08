use crate::{config::*, error::*, state::*};
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

fn mint_chrt<'info>(
    ctx: &Context<'_, '_, '_, 'info, Incentivize<'info>>,
    donor_chrt: &Account<'info, TokenAccount>,
    amount: u64,
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
    token::mint_to(cpi_ctx, amount)
}

pub fn incentivize<'info>(ctx: Context<'_, '_, '_, 'info, Incentivize<'info>>) -> Result<()> {
    let now = Clock::get()?.unix_timestamp as _;

    if now - ctx.accounts.platform.last_incentive_ts < ctx.accounts.platform.incentive_cooldown {
        return err!(CrowdfundingError::IncentiveCooldown);
    }
    ctx.accounts.platform.last_incentive_ts = now;

    let mut prev_donors = Vec::with_capacity(SEASONAL_TOP_CAPACITY);

    for pair in (ctx.remaining_accounts)
        .chunks_exact(2)
        .take(SEASONAL_TOP_CAPACITY)
    {
        let donor = AccountLoader::<Donor>::try_from(&pair[0])?;
        let mut donor = donor.load_mut()?;
        if prev_donors.contains(&pair[0].key()) {
            return err!(CrowdfundingError::DuplicateInTop);
        }
        prev_donors.push(pair[0].key());
        if donor.incentivized_donations_sum == donor.donations_sum {
            return err!(CrowdfundingError::NotEligibleForIncentive);
        }
        donor.incentivized_donations_sum = donor.donations_sum;

        let donor_chrt = Account::<TokenAccount>::try_from(&pair[1])?;
        if donor_chrt.owner != donor.authority {
            return err!(ConstraintTokenOwner);
        }

        mint_chrt(&ctx, &donor_chrt, ctx.accounts.platform.incentive_amount)?;
    }

    emit!(IncentivizeEvent {});

    Ok(())
}

#[event]
struct IncentivizeEvent {}
