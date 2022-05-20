use crate::{state::*, utils::*};
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct WithdrawDonations<'info> {
    #[account(
        seeds = [b"campaign", campaign.id.to_le_bytes().as_ref()],
        bump = campaign.bump,
    )]
    campaign: Account<'info, Campaign>,
    #[account(mut, address = campaign.authority)]
    campaign_authority: Signer<'info>,
    /// CHECK:
    #[account(
        mut,
        seeds = [b"sol_vault", campaign.id.to_le_bytes().as_ref()],
        bump = campaign.bump_sol_vault,
    )]
    sol_vault: UncheckedAccount<'info>,
}

pub fn withdraw_donations(ctx: Context<WithdrawDonations>) -> Result<()> {
    transfer_all_lamports(
        &ctx.accounts.sol_vault.to_account_info(),
        &ctx.accounts.campaign_authority.to_account_info(),
    )?;

    Ok(())
}
