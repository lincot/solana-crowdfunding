use crate::instructions::*;
use anchor_lang::prelude::*;

pub mod config;
pub mod error;
mod instructions;
pub mod state;
mod utils;

declare_id!("Gf3bXGS7iA2EUxzXs1xS6qwZBPGS8idyqMNitQ5NKDSA");

#[program]
pub mod crowdfunding {
    use super::*;

    pub fn initialize(
        ctx: Context<Initialize>,
        incentive_cooldown: u32,
        incentive_amount: u64,
        fee_basis_points: u16,
        fee_exemption_limit: u64,
        liquidation_limit: u64,
    ) -> Result<()> {
        instructions::initialize(
            ctx,
            incentive_cooldown,
            incentive_amount,
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

    pub fn incentivize<'info>(ctx: Context<'_, '_, '_, 'info, Incentivize<'info>>) -> Result<()> {
        instructions::incentivize(ctx)
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
