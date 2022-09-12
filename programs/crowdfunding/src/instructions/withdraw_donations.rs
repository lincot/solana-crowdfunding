use crate::{state::*, utils::*};
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct WithdrawDonations<'info> {
    #[account(mut, seeds = [b"platform"], bump = platform.load()?.bump)]
    platform: AccountLoader<'info, Platform>,
    #[account(mut, seeds = [b"sol_vault"], bump = sol_vault.load()?.bump)]
    sol_vault: AccountLoader<'info, Vault>,
    #[account(
        seeds = [b"campaign", campaign.load()?.id.to_le_bytes().as_ref()],
        bump = campaign.load()?.bump,
    )]
    campaign: AccountLoader<'info, Campaign>,
    #[account(mut, address = campaign.load()?.authority)]
    campaign_authority: Signer<'info>,
}

pub fn withdraw_donations(ctx: Context<WithdrawDonations>) -> Result<()> {
    let platform = &mut ctx.accounts.platform.load_mut()?;

    let id = ctx.accounts.campaign.load()?.id;
    let i = platform.active_campaigns[..platform.active_campaigns_count as usize]
        .binary_search_by_key(&id, |c| c.id)
        .unwrap();
    let lamports = {
        let mut campaign = platform.active_campaigns.get_mut(i).unwrap();
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
