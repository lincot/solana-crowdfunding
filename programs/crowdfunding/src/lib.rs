use crate::instructions::*;
use anchor_lang::prelude::*;

pub mod config;
pub mod error;
mod instructions;
pub mod state;
mod utils;

declare_id!("BkBYehfNc7WBa6MmmFz3mMzwBduQLBTzboA3e6JaBGYR");

#[program]
pub mod crowdfunding {
    use super::*;

    pub fn initialize(
        ctx: Context<Initialize>,
        reward_cooldown: u32,
        reward_amount: u64,
        fee_basis_points: u16,
        fee_exemption_limit: u64,
        liquidation_limit: u64,
    ) -> Result<()> {
        instructions::initialize(
            ctx,
            reward_cooldown,
            reward_amount,
            fee_basis_points,
            fee_exemption_limit,
            liquidation_limit,
        )
    }

    pub fn register_donor(ctx: Context<RegisterDonor>) -> Result<()> {
        instructions::register_donor(ctx)
    }

    pub fn start_campaign(ctx: Context<StartCampaign>) -> Result<()> {
        instructions::start_campaign(ctx)
    }

    pub fn donate(ctx: Context<Donate>, amount: u64) -> Result<()> {
        instructions::donate(ctx, amount)
    }

    pub fn donate_with_referer(ctx: Context<DonateWithReferer>, amount: u64) -> Result<()> {
        instructions::donate_with_referer(ctx, amount)
    }

    pub fn record_donors(ctx: Context<RecordDonors>) -> Result<()> {
        instructions::record_donors(ctx)
    }

    pub fn drop_rewards<'info>(ctx: Context<'_, '_, '_, 'info, DropRewards<'info>>) -> Result<()> {
        instructions::drop_rewards(ctx)
    }

    pub fn withdraw_donations(ctx: Context<WithdrawDonations>) -> Result<()> {
        instructions::withdraw_donations(ctx)
    }

    pub fn stop_campaign(ctx: Context<StopCampaign>) -> Result<()> {
        instructions::stop_campaign(ctx)
    }

    pub fn liquidate_campaign(ctx: Context<LiquidateCampaign>) -> Result<()> {
        instructions::liquidate_campaign(ctx)
    }

    pub fn withdraw_fees(ctx: Context<WithdrawFees>) -> Result<()> {
        instructions::withdraw_fees(ctx)
    }
}
