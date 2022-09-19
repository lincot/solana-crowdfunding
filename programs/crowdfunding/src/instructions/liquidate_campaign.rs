
use crate::{error::*, state::*, utils::*};
use anchor_lang::prelude::*;
use anchor_spl::token::{self, Burn, CloseAccount, Mint, Token, TokenAccount};

#[derive(Accounts)]
pub struct LiquidateCampaign<'info> {
    #[account(mut, seeds = [b"platform"], bump = platform.load()?.bump)]
    platform: AccountLoader<'info, Platform>,
    #[account(mut, seeds = [b"fee_vault"], bump = fee_vault.load()?.bump)]
    fee_vault: AccountLoader<'info, Vault>,
    #[account(mut, seeds = [b"sol_vault"], bump = sol_vault.load()?.bump)]
    sol_vault: AccountLoader<'info, Vault>,
    #[account(mut, seeds = [b"chrt_mint"], bump = platform.load()?.bump_chrt_mint)]
    chrt_mint: Account<'info, Mint>,
    #[account(
        mut,
        close = campaign_authority,
        seeds = [b"campaign", campaign.load()?.id.to_le_bytes().as_ref()],
        bump = campaign.load()?.bump,
    )]
    campaign: AccountLoader<'info, Campaign>,
    #[account(mut, address = campaign.load()?.authority)]
    campaign_authority: UncheckedAccount<'info>,
    #[account(
        mut,
        seeds = [b"fee_exemption_vault", campaign.load()?.id.to_le_bytes().as_ref()],
        bump = campaign.load()?.bump_fee_exemption_vault,
    )]
    fee_exemption_vault: Account<'info, TokenAccount>,
    #[account(
        mut,
        seeds = [b"liquidation_vault", campaign.load()?.id.to_le_bytes().as_ref()],
        bump = campaign.load()?.bump_liquidation_vault,
    )]
    liquidation_vault: Account<'info, TokenAccount>,
    token_program: Program<'info, Token>,
    system_program: Program<'info, System>,
}

fn close_chrt_vaults(ctx: &Context<LiquidateCampaign>) -> Result<()> {
    let signer: &[&[&[u8]]] = &[&[b"platform", &[ctx.accounts.platform.load()?.bump]]];

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
    if ctx.accounts.liquidation_vault.amount < ctx.accounts.platform.load()?.liquidation_limit {
        return err!(CrowdfundingError::NotEnoughCHRTInVault);
    }
    close_chrt_vaults(&ctx)?;

    let platform = &mut ctx.accounts.platform.load_mut()?;
    let len = platform.active_campaigns_count as usize;
    let id = ctx.accounts.campaign.load()?.id;
    let i = platform.active_campaigns[..len]
        .binary_search_by_key(&id, |c| c.id)
        .map_err(|_| CrowdfundingError::CampaignInactive)?;
    let campaign = platform.active_campaigns[i];
    platform.active_campaigns[i] = Default::default();
    platform.active_campaigns[i..len].rotate_left(1);
    platform.active_campaigns_count -= 1;
    let liquidation_amount = campaign.donations_sum - campaign.withdrawn_sum;
    platform.liquidations_sum += liquidation_amount;

    let remaining_sum = platform.sum_of_active_campaign_donations - campaign.donations_sum;
    let mut distributed_sum = 0;
    for active_campaign in &mut platform.active_campaigns {
        let share = liquidation_amount * active_campaign.donations_sum / remaining_sum;
        active_campaign.donations_sum += share;
        distributed_sum += share;
    }

    let not_distributed = liquidation_amount - distributed_sum;
    platform.sum_of_active_campaign_donations -= not_distributed;
    transfer(
        &ctx.accounts.sol_vault.to_account_info(),
        &ctx.accounts.fee_vault.to_account_info(),
        not_distributed,
    )?;

    emit!(LiquidateCampaignEvent {});

    Ok(())
}

#[event]
struct LiquidateCampaignEvent {}
