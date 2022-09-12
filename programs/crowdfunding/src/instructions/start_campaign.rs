use crate::{config::*, error::*, state::*};
use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};
use core::mem::size_of;

#[derive(Accounts)]
pub struct StartCampaign<'info> {
    #[account(mut, seeds = [b"platform"], bump = platform.load()?.bump)]
    platform: AccountLoader<'info, Platform>,
    #[account(seeds = [b"chrt_mint"], bump = platform.load()?.bump_chrt_mint)]
    chrt_mint: Account<'info, Mint>,
    #[account(
        init,
        payer = campaign_authority,
        seeds = [b"campaign", platform.load()?.campaigns_count.to_le_bytes().as_ref()],
        bump,
        space = 8 + size_of::<Campaign>(),
    )]
    campaign: AccountLoader<'info, Campaign>,
    #[account(mut)]
    campaign_authority: Signer<'info>,
    #[account(
        init,
        payer = campaign_authority,
        seeds = [b"donations", campaign.key().as_ref()],
        bump,
        space = 8 + size_of::<Donations>(),
    )]
    total_donations_to_campaign: AccountLoader<'info, Donations>,
    #[account(
        init,
        payer = campaign_authority,
        seeds = [b"fee_exemption_vault", platform.load()?.campaigns_count.to_le_bytes().as_ref()],
        bump,
        token::authority = platform,
        token::mint = chrt_mint,
    )]
    fee_exemption_vault: Account<'info, TokenAccount>,
    #[account(
        init,
        payer = campaign_authority,
        seeds = [b"liquidation_vault", platform.load()?.campaigns_count.to_le_bytes().as_ref()],
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
    let platform = &mut ctx.accounts.platform.load_mut()?;
    if ACTIVE_CAMPAIGNS_CAPACITY as u16 <= platform.active_campaigns_count {
        return err!(CrowdfundingError::ActiveCampaignsLimit);
    }
    let id = platform.campaigns_count;
    let len = platform.active_campaigns_count as usize;
    platform.active_campaigns[len] = CampaignRecord {
        id,
        ..Default::default()
    };
    platform.campaigns_count += 1;
    platform.active_campaigns_count += 1;

    let campaign = &mut ctx.accounts.campaign.load_init()?;
    campaign.bump = *ctx.bumps.get("campaign").unwrap();
    campaign.bump_fee_exemption_vault = *ctx.bumps.get("fee_exemption_vault").unwrap();
    campaign.bump_liquidation_vault = *ctx.bumps.get("liquidation_vault").unwrap();
    campaign.authority = ctx.accounts.campaign_authority.key();
    campaign.id = id;

    ctx.accounts.total_donations_to_campaign.load_init()?.bump =
        *ctx.bumps.get("total_donations_to_campaign").unwrap();

    emit!(StartCampaignEvent {});

    Ok(())
}

#[event]
struct StartCampaignEvent {}
