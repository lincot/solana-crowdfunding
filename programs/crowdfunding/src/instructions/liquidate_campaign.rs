use super::claim_liquidated_sol::*;
use crate::{error::*, state::*, utils::*};
use anchor_lang::prelude::*;
use anchor_spl::token::{self, Burn, CloseAccount, Mint, Token, TokenAccount};

#[derive(Accounts)]
pub struct LiquidateCampaign<'info> {
    #[account(mut, seeds = [b"platform"], bump = platform.bump)]
    platform: Account<'info, Platform>,
    /// CHECK:
    #[account(mut, seeds = [b"liquidated_sol_vault"], bump = platform.bump_liquidated_sol_vault)]
    liquidated_sol_vault: UncheckedAccount<'info>,
    #[account(mut, seeds = [b"chrt_mint"], bump = platform.bump_chrt_mint)]
    chrt_mint: Account<'info, Mint>,
    #[account(
        seeds = [b"campaign", campaign.id.to_le_bytes().as_ref()],
        bump = campaign.bump,
    )]
    campaign: Account<'info, Campaign>,
    /// CHECK:
    #[account(mut, address = campaign.authority)]
    campaign_authority: UncheckedAccount<'info>,
    /// CHECK:
    #[account(
        mut,
        seeds = [b"sol_vault", campaign.id.to_le_bytes().as_ref()],
        bump = campaign.bump_sol_vault,
    )]
    sol_vault: UncheckedAccount<'info>,
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
}

fn close_chrt_vaults(ctx: &Context<LiquidateCampaign>) -> Result<()> {
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

pub fn liquidate_campaign(ctx: Context<LiquidateCampaign>) -> Result<()> {
    if ctx.accounts.liquidation_vault.amount < ctx.accounts.platform.liquidation_limit {
        return err!(CrowdfundingError::NotEnoughCHRTInVault);
    }

    claim_liquidated_sol(
        &ctx.accounts.platform,
        &ctx.accounts.liquidated_sol_vault,
        &mut ctx.accounts.campaign,
        &ctx.accounts.sol_vault,
    )?;

    if ctx.accounts.platform.liquidations_history.len()
        >= ctx.accounts.platform.max_liquidations as _
    {
        return err!(CrowdfundingError::LiquidationsLimit);
    }
    let sum_of_active_campaign_donations = ctx.accounts.platform.sum_of_active_campaign_donations;
    (ctx.accounts.platform.liquidations_history).push(LiquidationRecord {
        timestamp: Clock::get()?.unix_timestamp as _,
        liquidated_amount: ctx.accounts.sol_vault.lamports(),
        sum_of_active_campaign_donations,
    });

    ctx.accounts.platform.sum_of_active_campaign_donations -= ctx.accounts.campaign.donations_sum;
    ctx.accounts.platform.liquidations_sum += ctx.accounts.sol_vault.lamports();

    close_chrt_vaults(&ctx)?;
    transfer_all_lamports(
        &ctx.accounts.sol_vault.to_account_info(),
        &ctx.accounts.liquidated_sol_vault.to_account_info(),
    )?;

    emit!(LiquidateCampaignEvent {});

    Ok(())
}

#[event]
struct LiquidateCampaignEvent {}
