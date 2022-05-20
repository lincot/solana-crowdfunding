use crate::{state::*, utils::*};
use anchor_lang::prelude::*;

pub fn claim_liquidated_sol(
    platform: &Account<Platform>,
    liquidated_sol_vault: &UncheckedAccount,
    campaign: &mut Account<Campaign>,
    sol_vault: &UncheckedAccount,
) -> Result<()> {
    for liquidation in &platform.liquidations_history {
        if liquidation.timestamp > campaign.last_claim_ts {
            transfer_lamports(
                &liquidated_sol_vault.to_account_info(),
                &sol_vault.to_account_info(),
                liquidation.liquidated_amount
                    * (campaign.donations_sum)
                        .checked_div(liquidation.sum_of_active_campaign_donations)
                        .unwrap_or(0),
            )?;
        }
    }

    campaign.last_claim_ts = Clock::get()?.unix_timestamp as _;

    Ok(())
}
