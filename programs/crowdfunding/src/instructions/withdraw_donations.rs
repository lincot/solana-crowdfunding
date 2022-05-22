use crate::state::*;
use anchor_lang::{
    prelude::*,
    solana_program::{program::invoke_signed, system_instruction},
};

#[derive(Accounts)]
pub struct WithdrawDonations<'info> {
    #[account(mut, seeds = [b"platform"], bump = platform.bump)]
    platform: Account<'info, Platform>,
    /// CHECK:
    #[account(mut, seeds = [b"sol_vault"], bump = platform.bump_sol_vault)]
    sol_vault: UncheckedAccount<'info>,
    #[account(
        seeds = [b"campaign", campaign.id.to_le_bytes().as_ref()],
        bump = campaign.bump,
    )]
    campaign: Account<'info, Campaign>,
    #[account(mut, address = campaign.authority)]
    campaign_authority: Signer<'info>,
    system_program: Program<'info, System>,
}

fn withdraw_lamports(ctx: &Context<WithdrawDonations>, lamports: u64) -> Result<()> {
    invoke_signed(
        &system_instruction::transfer(
            ctx.accounts.sol_vault.key,
            ctx.accounts.campaign_authority.key,
            lamports,
        ),
        &[
            ctx.accounts.sol_vault.to_account_info(),
            ctx.accounts.campaign_authority.to_account_info(),
        ],
        &[&[b"sol_vault", &[ctx.accounts.platform.bump_sol_vault]]],
    )?;
    Ok(())
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

    withdraw_lamports(&ctx, lamports)?;

    emit!(WithdrawDonationsEvent {});

    Ok(())
}

#[event]
struct WithdrawDonationsEvent {}
