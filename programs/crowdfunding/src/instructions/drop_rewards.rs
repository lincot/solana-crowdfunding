use crate::{config::*, error::*, state::*};
use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, MintTo, Token, TokenAccount};

#[derive(Accounts)]
pub struct DropRewards<'info> {
    #[account(mut, seeds = [b"platform"], bump)]
    platform: AccountLoader<'info, Platform>,
    #[account(mut, seeds = [b"chrt_mint"], bump)]
    chrt_mint: Account<'info, Mint>,
    token_program: Program<'info, Token>,
}

fn mint_chrt<'info>(
    ctx: &Context<'_, '_, '_, 'info, DropRewards<'info>>,
    donor_chrt: &Account<'info, TokenAccount>,
    reward_amount: u64,
) -> Result<()> {
    token::mint_to(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            MintTo {
                mint: ctx.accounts.chrt_mint.to_account_info(),
                to: donor_chrt.to_account_info(),
                authority: ctx.accounts.platform.to_account_info(),
            },
            &[&[b"platform", &[*ctx.bumps.get("platform").unwrap()]]],
        ),
        reward_amount,
    )
}

pub fn drop_rewards<'info>(ctx: Context<'_, '_, '_, 'info, DropRewards<'info>>) -> Result<()> {
    let (reward_amount, seasonal_top, seasonal_top_len) = {
        let platform = &mut ctx.accounts.platform.load_mut()?;

        if !platform.reward_procedure_is_in_process
            || platform.donors_recorded != platform.donors_count
        {
            return err!(CrowdfundingError::NotAllDonorsRecorded);
        }

        if ctx.remaining_accounts.len() != 2 * platform.seasonal_top.len() {
            return err!(CrowdfundingError::IncorrectSeasonalTop);
        }

        let reward_amount = platform.reward_amount;
        let seasonal_top = platform.seasonal_top;
        let seasonal_top_len = seasonal_top
            .iter()
            .position(|d| d.donations_sum == 0)
            .unwrap_or(seasonal_top.len());

        platform.reward_procedure_is_in_process = false;
        platform.donors_recorded = 0;
        platform.seasonal_top = [DonorRecord {
            ..Default::default()
        }; SEASONAL_TOP_CAPACITY];

        (reward_amount, seasonal_top, seasonal_top_len)
    };
    let seasonal_top = &seasonal_top[..seasonal_top_len];

    for (pair, seasonal_top_donor) in ctx.remaining_accounts.chunks_exact(2).zip(seasonal_top) {
        let donor = AccountLoader::<Donor>::try_from(&pair[0])?;
        let donor = &mut donor.load_mut()?;
        if donor.authority != seasonal_top_donor.donor {
            return err!(CrowdfundingError::IncorrectSeasonalTop);
        }
        donor.rewarded_donations_sum = donor.donations_sum;

        let donor_chrt = Account::<TokenAccount>::try_from(&pair[1])?;
        if donor.authority != donor_chrt.owner {
            return err!(ConstraintTokenOwner);
        }
        mint_chrt(&ctx, &donor_chrt, reward_amount)?;
    }

    Ok(())
}
