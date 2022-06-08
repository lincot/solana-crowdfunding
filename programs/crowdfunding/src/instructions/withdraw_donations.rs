use crate::{state::*, utils::*};
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct WithdrawDonations<'info> {
    #[account(mut, seeds = [b"platform"], bump = platform.bump)]
    platform: Account<'info, Platform>,
    #[account(mut, seeds = [b"sol_vault"], bump = sol_vault.bump)]
    sol_vault: Account<'info, Vault>,
    #[account(
        seeds = [b"campaign", campaign.id.to_le_bytes().as_ref()],
        bump = campaign.bump,
    )]
    campaign: Account<'info, Campaign>,
    #[account(mut, address = campaign.authority)]
    campaign_authority: Signer<'info>,
}

pub fn withdraw_donations(ctx: Context<WithdrawDonations>) -> Result<()> {
    let i = (ctx.accounts.platform.active_campaigns)
        .binary_search_by_key(&ctx.accounts.campaign.id, |c| c.id)
        .unwrap();
    let lamports = {
        let mut campaign = ctx.accounts.platform.active_campaigns.get_mut(i).unwrap();
        let lamports = campaign.donations_sum - campaign.withdrawn_sum;
        campaign.withdrawn_sum = campaign.donations_sum;
        lamports
    };

    transfer(
        &ctx.accounts.sol_vault.to_account_info(),
        &ctx.accounts.campaign_authority.to_account_info(),
        lamports,
    )?;

    emit!(WithdrawDonationsEvent {});

    Ok(())
}

#[event]
struct WithdrawDonationsEvent {}
