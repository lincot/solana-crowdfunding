use anchor_lang::prelude::*;

pub const PLATFORM_TOP_CAPACITY: usize = 100;
pub const SEASONAL_TOP_CAPACITY: usize = 10;
pub const CAMPAIGN_TOP_CAPACITY: usize = 10;

#[derive(AnchorSerialize, AnchorDeserialize, Copy, Clone, Debug, Default)]
pub struct DonorRecord {
    pub donor: Pubkey,
    pub donations_sum: u64,
}
impl DonorRecord {
    pub const SPACE: usize = 32 + 8;
}

#[derive(AnchorSerialize, AnchorDeserialize, Copy, Clone, Debug, Default)]
pub struct CampaignRecord {
    pub donations_sum: u64,
    pub withdrawn_sum: u64,
    pub is_closed: bool,
}
impl CampaignRecord {
    pub const SPACE: usize = 8 + 8 + 1;
}

#[account]
pub struct Platform {
    pub bump: u8,
    pub bump_fee_vault: u8,
    pub bump_sol_vault: u8,
    pub bump_chrt_mint: u8,
    pub authority: Pubkey,
    pub campaigns_capacity: u16,
    pub incentive_cooldown: u32,
    pub incentive_amount: u64,
    pub platform_fee_num: u64,
    pub platform_fee_denom: u64,
    pub fee_exemption_limit: u64,
    pub liquidation_limit: u64,
    pub last_incentive_ts: u32,
    pub sum_of_all_donations: u64,
    pub sum_of_active_campaign_donations: u64,
    pub avoided_fees_sum: u64,
    pub liquidations_sum: u64,
    pub platform_top: Vec<DonorRecord>,
    pub seasonal_top: Vec<DonorRecord>,
    pub campaigns: Vec<CampaignRecord>,
}
impl Platform {
    pub const fn space(campaigns_capacity: u16) -> usize {
        (1 + 1 + 1 + 1 + 32 + 2 + 4 + 8 + 8 + 8 + 8 + 8 + 4 + 8 + 8 + 8 + 8)
            + (4 + PLATFORM_TOP_CAPACITY * DonorRecord::SPACE)
            + (4 + SEASONAL_TOP_CAPACITY * DonorRecord::SPACE)
            + (4 + campaigns_capacity as usize * CampaignRecord::SPACE)
    }
}

#[account]
pub struct Campaign {
    pub bump: u8,
    pub bump_fee_exemption_vault: u8,
    pub bump_liquidation_vault: u8,
    pub authority: Pubkey,
    pub id: u16,
    pub campaign_top: Vec<DonorRecord>,
}
impl Campaign {
    pub const SPACE: usize = 1 + 1 + 1 + 32 + 2 + (4 + CAMPAIGN_TOP_CAPACITY * DonorRecord::SPACE);
}

#[account]
pub struct Donor {
    pub bump: u8,
    pub donations_sum: u64,
    pub seasonal_donations_sum: u64,
    pub last_donation_ts: u32,
}
impl Donor {
    pub const SPACE: usize = 1 + 8 + 8 + 4;
}

#[account]
pub struct Donations {
    pub donations_sum: u64,
}
impl Donations {
    pub const SPACE: usize = 8;
}
