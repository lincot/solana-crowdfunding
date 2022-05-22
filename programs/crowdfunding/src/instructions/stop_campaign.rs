use crate::state::*;
use anchor_lang::{
    prelude::*,
    solana_program::{program::invoke_signed, system_instruction},
};
use anchor_spl::token::{self, Burn, CloseAccount, Mint, Token, TokenAccount};

#[derive(Accounts)]
pub struct StopCampaign<'info> {
    #[account(mut, seeds = [b"platform"], bump = platform.bump)]
    platform: Account<'info, Platform>,
    /// CHECK:
    #[account(mut, seeds = [b"sol_vault"], bump = platform.bump_sol_vault)]
    sol_vault: UncheckedAccount<'info>,
    #[account(mut, seeds = [b"chrt_mint"], bump = platform.bump_chrt_mint)]
    chrt_mint: Account<'info, Mint>,
    #[account(
        mut,
        close = campaign_authority,
        seeds = [b"campaign", campaign.id.to_le_bytes().as_ref()],
        bump = campaign.bump,
    )]
    campaign: Account<'info, Campaign>,
    #[account(mut, address = campaign.authority)]
    campaign_authority: Signer<'info>,
    #[account(
        mut,
        seeds = [b"fee_exemption_vault", campaign.id.to_le_bytes().as_ref()],
        bump = campaign.bump_fee_exemption_vault,
    )]
    fee_exemption_vault: Account<'info, TokenAccount>,
    #[account(
        mut,
        seeds = [b"liquidation_vault", campaign.id.to_le_bytes().as_ref()],
        bump = campaign.bump_liquidation_vault,
    )]
    liquidation_vault: Account<'info, TokenAccount>,
    token_program: Program<'info, Token>,
    system_program: Program<'info, System>,
}

fn close_chrt_vaults(ctx: &Context<StopCampaign>) -> Result<()> {
    let signer: &[&[&[u8]]] = &[&[b"platform", &[ctx.accounts.platform.bump]]];

    for vault in [
        &ctx.accounts.fee_exemption_vault,
        &ctx.accounts.liquidation_vault,
    ] {
        let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            Burn {
                mint: ctx.accounts.chrt_mint.to_account_info(),
                from: vault.to_account_info(),
                authority: ctx.accounts.platform.to_account_info(),
            },
            signer,
        );
        token::burn(cpi_ctx, vault.amount)?;

        let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            CloseAccount {
                account: vault.to_account_info(),
                destination: ctx.accounts.campaign_authority.to_account_info(),
                authority: ctx.accounts.platform.to_account_info(),
            },
            signer,
        );
        token::close_account(cpi_ctx)?;
    }

    Ok(())
}

fn withdraw_lamports(ctx: &Context<StopCampaign>, lamports: u64) -> Result<()> {
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

pub fn stop_campaign(ctx: Context<StopCampaign>) -> Result<()> {
    close_chrt_vaults(&ctx)?;

    let i = (ctx.accounts.platform.active_campaigns)
        .binary_search_by_key(&ctx.accounts.campaign.id, |c| c.id)
        .unwrap();
    let campaign = ctx.accounts.platform.active_campaigns.remove(i);

    withdraw_lamports(&ctx, campaign.donations_sum - campaign.withdrawn_sum)?;

    ctx.accounts.platform.sum_of_active_campaign_donations -= campaign.donations_sum;

    emit!(StopCampaignEvent {});

    Ok(())
}

#[event]
struct StopCampaignEvent {}
