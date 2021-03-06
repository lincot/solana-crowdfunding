use crate::{error::*, state::*};
use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};

#[derive(Accounts)]
pub struct StartCampaign<'info> {
    #[account(mut, seeds = [b"platform"], bump = platform.bump)]
    platform: Account<'info, Platform>,
    #[account(seeds = [b"chrt_mint"], bump = platform.bump_chrt_mint)]
    chrt_mint: Account<'info, Mint>,
    #[account(
        init,
        payer = campaign_authority,
        seeds = [b"campaign", platform.campaigns_count.to_le_bytes().as_ref()],
        bump,
        space = 8 + Campaign::SPACE,
    )]
    campaign: Account<'info, Campaign>,
    #[account(mut)]
    campaign_authority: Signer<'info>,
    #[account(
        init,
        payer = campaign_authority,
        seeds = [b"donations", campaign.key().as_ref()],
        bump,
        space = 8 + Donations::SPACE,
    )]
    total_donations_to_campaign: Account<'info, Donations>,
    #[account(
        init,
        payer = campaign_authority,
        seeds = [b"fee_exemption_vault", platform.campaigns_count.to_le_bytes().as_ref()],
        bump,
        token::authority = platform,
        token::mint = chrt_mint,
    )]
    fee_exemption_vault: Account<'info, TokenAccount>,
    #[account(
        init,
        payer = campaign_authority,
        seeds = [b"liquidation_vault", platform.campaigns_count.to_le_bytes().as_ref()],
        bump,
        token::authority = platform,
        token::mint = chrt_mint,
    )]
    liquidation_vault: Account<'info, TokenAccount>,
    rent: Sysvar<'info, Rent>,
    token_program: Program<'info, Token>,
    system_program: Program<'info, System>,
}

pub fn start_campaign(ctx: Context<StartCampaign>) -> Result<()> {
    if ctx.accounts.platform.active_campaigns_capacity
        <= ctx.accounts.platform.active_campaigns.len() as _
    {
        return err!(CrowdfundingError::ActiveCampaignsLimit);
    }
    let id = ctx.accounts.platform.campaigns_count;
    ctx.accounts.platform.active_campaigns.push(CampaignRecord {
        id,
        ..Default::default()
    });
    ctx.accounts.platform.campaigns_count += 1;

    ctx.accounts.campaign.bump = *ctx.bumps.get("campaign").unwrap();
    ctx.accounts.campaign.bump_fee_exemption_vault = *ctx.bumps.get("fee_exemption_vault").unwrap();
    ctx.accounts.campaign.bump_liquidation_vault = *ctx.bumps.get("liquidation_vault").unwrap();
    ctx.accounts.campaign.authority = ctx.accounts.campaign_authority.key();
    ctx.accounts.campaign.id = id;

    ctx.accounts.total_donations_to_campaign.bump =
        *ctx.bumps.get("total_donations_to_campaign").unwrap();

    emit!(StartCampaignEvent {});

    Ok(())
}

#[event]
struct StartCampaignEvent {}
