use super::claim_liquidated_sol::*;
use crate::{error::*, state::*, CHRT_DECIMALS};
use anchor_lang::{
    prelude::*,
    solana_program::{program::invoke, system_instruction},
};
use anchor_spl::token::{
    self, spl_token::native_mint::DECIMALS, Mint, MintTo, Token, TokenAccount,
};

#[derive(Accounts)]
pub struct Donate<'info> {
    #[account(mut, seeds = [b"platform"], bump = platform.bump)]
    platform: Box<Account<'info, Platform>>,
    /// CHECK:
    #[account(mut, seeds = [b"fee_vault"], bump = platform.bump_fee_vault)]
    fee_vault: UncheckedAccount<'info>,
    /// CHECK:
    #[account(mut, seeds = [b"liquidated_sol_vault"], bump = platform.bump_liquidated_sol_vault)]
    liquidated_sol_vault: UncheckedAccount<'info>,
    #[account(
        mut,
        seeds = [b"campaign", campaign.id.to_le_bytes().as_ref()],
        bump = campaign.bump,
    )]
    campaign: Account<'info, Campaign>,
    /// CHECK:
    #[account(
        mut,
        seeds = [b"sol_vault", campaign.id.to_le_bytes().as_ref()],
        bump = campaign.bump_sol_vault,
    )]
    sol_vault: UncheckedAccount<'info>,
    #[account(
        seeds = [b"fee_exemption_vault", campaign.id.to_le_bytes().as_ref()],
        bump = campaign.bump_fee_exemption_vault,
    )]
    fee_exemption_vault: Account<'info, TokenAccount>,
    #[account(mut, seeds = [b"donor", donor_authority.key().as_ref()], bump = donor.bump)]
    donor: Account<'info, Donor>,
    #[account(mut)]
    donor_authority: Signer<'info>,
    #[account(
        init_if_needed,
        payer = donor_authority,
        seeds = [b"donations", donor_authority.key().as_ref(), campaign.key().as_ref()],
        bump,
        space = 8 + Donations::SPACE,
    )]
    donations: Account<'info, Donations>,
    system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct DonateWithReferer<'info> {
    donate: Donate<'info>,
    #[account(mut, seeds = [b"chrt_mint"], bump = donate.platform.bump_chrt_mint)]
    chrt_mint: Account<'info, Mint>,
    #[account(
        seeds = [b"donor", referer_authority.key().as_ref()],
        bump = referer.bump,
        constraint = referer.key() != donate.donor.key() @ CrowdfundingError::CannotReferYourself,
    )]
    referer: Account<'info, Donor>,
    /// CHECK:
    referer_authority: UncheckedAccount<'info>,
    #[account(mut, token::authority = referer_authority)]
    referer_chrt: Account<'info, TokenAccount>,
    token_program: Program<'info, Token>,
}

fn transfer_to_campaign(accounts: &mut Donate, lamports: u64) -> Result<()> {
    accounts.campaign.donations_sum += lamports;
    accounts.platform.sum_of_all_donations += lamports;
    accounts.platform.sum_of_active_campaign_donations += lamports;
    accounts.donor.donations_sum += lamports;
    if accounts.donor.last_donation_ts < accounts.platform.last_incentive_ts {
        accounts.donor.seasonal_donations_sum = 0;
    }
    accounts.donor.last_donation_ts = Clock::get()?.unix_timestamp as _;
    accounts.donor.seasonal_donations_sum += lamports;
    accounts.donations.sum += lamports;

    invoke(
        &system_instruction::transfer(
            accounts.donor_authority.key,
            accounts.sol_vault.key,
            lamports,
        ),
        &[
            accounts.donor_authority.to_account_info(),
            accounts.sol_vault.to_account_info(),
        ],
    )?;
    Ok(())
}

fn transfer_to_platform(accounts: &Donate, lamports: u64) -> Result<()> {
    invoke(
        &system_instruction::transfer(
            accounts.donor_authority.key,
            accounts.fee_vault.key,
            lamports,
        ),
        &[
            accounts.donor_authority.to_account_info(),
            accounts.fee_vault.to_account_info(),
        ],
    )?;
    Ok(())
}

fn add_to_top(top: &mut Vec<DonorRecord>, donor_record: DonorRecord, capacity: usize) {
    let (cur_i, assigned) =
        if let Some(cur_i) = top.iter().position(|d| d.donor == donor_record.donor) {
            // assign new sum
            top[cur_i] = donor_record;
            (cur_i, true)
        } else if top.len() < capacity {
            // push new donor
            top.push(donor_record);
            (top.len() - 1, true)
        } else {
            // no space to push, so pretend it's the last
            (top.len() - 1, false)
        };

    if let Some(new_i) = top[..cur_i].iter().position(|d| d.sum < donor_record.sum) {
        // sort donor
        top[new_i..=cur_i].rotate_right(1);

        if !assigned {
            // previously last donor gets replaced by new one
            top[new_i] = donor_record;
        }
    }
}

fn donate_common(accounts: &mut Donate, lamports: u64) -> Result<()> {
    claim_liquidated_sol(
        &accounts.platform,
        &accounts.liquidated_sol_vault,
        &mut accounts.campaign,
        &accounts.sol_vault,
    )?;

    let fee = lamports * accounts.platform.platform_fee_num / accounts.platform.platform_fee_denom;
    if accounts.fee_exemption_vault.amount < accounts.platform.fee_exemption_limit {
        transfer_to_campaign(accounts, lamports - fee)?;
        transfer_to_platform(accounts, fee)?;
    } else {
        transfer_to_campaign(accounts, lamports)?;
        accounts.platform.avoided_fees_sum += fee;
    }

    add_to_top(
        &mut accounts.platform.platform_top,
        DonorRecord {
            donor: accounts.donor_authority.key(),
            sum: accounts.donor.donations_sum,
        },
        PLATFORM_TOP_CAPACITY,
    );
    add_to_top(
        &mut accounts.platform.seasonal_top,
        DonorRecord {
            donor: accounts.donor_authority.key(),
            sum: accounts.donor.seasonal_donations_sum,
        },
        SEASONAL_TOP_CAPACITY,
    );
    add_to_top(
        &mut accounts.campaign.campaign_top,
        DonorRecord {
            donor: accounts.donor_authority.key(),
            sum: accounts.donations.sum,
        },
        CAMPAIGN_TOP_CAPACITY,
    );

    Ok(())
}

pub fn donate(ctx: Context<Donate>, lamports: u64) -> Result<()> {
    donate_common(ctx.accounts, lamports)?;

    emit!(DonateEvent {});

    Ok(())
}

#[event]
struct DonateEvent {}

fn mint_chrt_to_referer(ctx: Context<DonateWithReferer>, amount: u64) -> Result<()> {
    let signer: &[&[&[u8]]] = &[&[b"platform", &[ctx.accounts.donate.platform.bump]]];
    let cpi_ctx = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        MintTo {
            mint: ctx.accounts.chrt_mint.to_account_info(),
            to: ctx.accounts.referer_chrt.to_account_info(),
            authority: ctx.accounts.donate.platform.to_account_info(),
        },
        signer,
    );
    token::mint_to(cpi_ctx, amount)
}

pub fn donate_with_referer(ctx: Context<DonateWithReferer>, lamports: u64) -> Result<()> {
    donate_common(&mut ctx.accounts.donate, lamports)?;

    mint_chrt_to_referer(
        ctx,
        101 * lamports / 10u64.pow((DECIMALS - CHRT_DECIMALS) as _),
    )?;

    emit!(DonateWithRefererEvent {});

    Ok(())
}

#[event]
struct DonateWithRefererEvent {}
