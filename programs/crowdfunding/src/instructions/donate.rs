use crate::{config::*, error::*, state::*, utils::*};
use anchor_lang::{
    prelude::*,
    solana_program::{program::invoke, system_instruction},
};
use anchor_spl::token::{
    self, spl_token::native_mint::DECIMALS, Mint, MintTo, Token, TokenAccount,
};
use core::{mem::size_of, ops::Deref};

#[derive(Accounts)]
pub struct Donate<'info> {
    #[account(mut, seeds = [b"platform"], bump)]
    platform: AccountLoader<'info, Platform>,
    #[account(mut, seeds = [b"fee_vault"], bump)]
    fee_vault: AccountLoader<'info, Vault>,
    #[account(mut, seeds = [b"sol_vault"], bump)]
    sol_vault: AccountLoader<'info, Vault>,
    #[account(
        mut,
        seeds = [b"campaign", campaign.load()?.id.to_le_bytes().as_ref()],
        bump,
    )]
    campaign: AccountLoader<'info, Campaign>,
    #[account(
        mut,
        seeds = [b"donations", campaign.load()?.id.to_le_bytes().as_ref()],
        bump,
    )]
    total_donations_to_campaign: AccountLoader<'info, Donations>,
    #[account(
        seeds = [b"fee_exemption_vault", campaign.load()?.id.to_le_bytes().as_ref()],
        bump,
    )]
    fee_exemption_vault: Account<'info, TokenAccount>,
    #[account(mut, seeds = [b"donor", donor_authority.key().as_ref()], bump)]
    donor: AccountLoader<'info, Donor>,
    #[account(mut)]
    donor_authority: Signer<'info>,
    #[account(
        init_if_needed,
        payer = donor_authority,
        seeds = [b"donations", donor_authority.key().as_ref(), campaign.load()?.id.to_le_bytes().as_ref()],
        bump,
        space = 8 + size_of::<Donations>(),
    )]
    donor_donations_to_campaign: AccountLoader<'info, Donations>,
    system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct DonateWithReferer<'info> {
    donate: Donate<'info>,
    #[account(mut, seeds = [b"chrt_mint"], bump)]
    chrt_mint: Account<'info, Mint>,
    #[account(
        seeds = [b"donor", referer_authority.key().as_ref()],
        bump,
        constraint = referer.key() != donate.donor.key() @ CrowdfundingError::CannotReferYourself,
    )]
    referer: AccountLoader<'info, Donor>,
    referer_authority: UncheckedAccount<'info>,
    #[account(mut, token::authority = referer_authority)]
    referer_chrt: Account<'info, TokenAccount>,
    token_program: Program<'info, Token>,
}

fn transfer_to_campaign(accounts: &mut Donate, lamports: u64) -> Result<()> {
    let platform = &mut accounts.platform.load_mut()?;
    let id = accounts.campaign.load()?.id;
    let i = platform.active_campaigns[..platform.active_campaigns_count as usize]
        .binary_search_by_key(&id, |c| c.id)
        .unwrap();
    platform.active_campaigns[i].donations_sum += lamports;
    platform.sum_of_all_donations += lamports;
    platform.sum_of_active_campaign_donations += lamports;
    accounts.donor.load_mut()?.donations_sum += lamports;
    (accounts.total_donations_to_campaign.load_mut()?).donations_sum += lamports;
    let donor_donations_to_campaign = &mut if accounts
        .donor_donations_to_campaign
        .to_account_info()
        .try_borrow_data()?
        .starts_with(&[0; 8])
    {
        accounts.donor_donations_to_campaign.load_init()?
    } else {
        accounts.donor_donations_to_campaign.load_mut()?
    };
    donor_donations_to_campaign.donations_sum += lamports;

    invoke(
        &system_instruction::transfer(
            &accounts.donor_authority.key(),
            &accounts.sol_vault.key(),
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
            &accounts.donor_authority.key(),
            &accounts.fee_vault.key(),
            lamports,
        ),
        &[
            accounts.donor_authority.to_account_info(),
            accounts.fee_vault.to_account_info(),
        ],
    )?;
    Ok(())
}

fn donate_common(accounts: &mut Donate, lamports: u64) -> Result<()> {
    let &Platform {
        reward_procedure_is_in_process,
        fee_basis_points,
        fee_exemption_limit,
        ..
    } = accounts.platform.load()?.deref();

    if reward_procedure_is_in_process {
        return err!(CrowdfundingError::RewardProcedureInProcess);
    }

    let fee = lamports * fee_basis_points as u64 / 10000;
    if accounts.fee_exemption_vault.amount < fee_exemption_limit {
        transfer_to_campaign(accounts, lamports - fee)?;
        transfer_to_platform(accounts, fee)?;
    } else {
        transfer_to_campaign(accounts, lamports)?;
        accounts.platform.load_mut()?.avoided_fees_sum += fee;
    }

    add_to_top(
        &mut accounts.platform.load_mut()?.top,
        DonorRecord {
            donor: accounts.donor_authority.key(),
            donations_sum: accounts.donor.load()?.donations_sum,
        },
    );

    let donations_sum = if accounts
        .donor_donations_to_campaign
        .to_account_info()
        .try_borrow_data()?
        .starts_with(&[0; 8])
    {
        accounts
            .donor_donations_to_campaign
            .load_init()?
            .donations_sum
    } else {
        accounts.donor_donations_to_campaign.load()?.donations_sum
    };
    add_to_top(
        &mut accounts.campaign.load_mut()?.top,
        DonorRecord {
            donor: accounts.donor_authority.key(),
            donations_sum,
        },
    );

    Ok(())
}

pub fn donate(ctx: Context<Donate>, lamports: u64) -> Result<()> {
    donate_common(ctx.accounts, lamports)?;

    Ok(())
}

fn mint_chrt_to_referer(ctx: Context<DonateWithReferer>, amount: u64) -> Result<()> {
    let signer: &[&[&[u8]]] = &[&[b"platform", &[*ctx.bumps.get("platform").unwrap()]]];
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

    Ok(())
}
