use super::claim_liquidated_sol::*;
use crate::{state::*, utils::*};
use anchor_lang::prelude::*;
use anchor_spl::token::{self, Burn, CloseAccount, Mint, Token, TokenAccount};

#[derive(Accounts)]
pub struct StopCampaign<'info> {
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
    #[account(mut, address = campaign.authority)]
    campaign_authority: Signer<'info>,
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

pub fn stop_campaign(ctx: Context<StopCampaign>) -> Result<()> {
    claim_liquidated_sol(
        &ctx.accounts.platform,
        &ctx.accounts.liquidated_sol_vault,
        &mut ctx.accounts.campaign,
        &ctx.accounts.sol_vault,
    )?;

    ctx.accounts.platform.sum_of_active_campaign_donations -= ctx.accounts.campaign.donations_sum;

    close_chrt_vaults(&ctx)?;
    transfer_all_lamports(
        &ctx.accounts.sol_vault.to_account_info(),
        &ctx.accounts.campaign_authority.to_account_info(),
    )?;

    emit!(StopCampaignEvent {});

    Ok(())
}

#[event]
struct StopCampaignEvent {}
