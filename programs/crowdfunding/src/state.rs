use crate::config::*;
use anchor_lang::prelude::*;

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
    pub id: u16,
    pub donations_sum: u64,
    pub withdrawn_sum: u64,
}
impl CampaignRecord {
    pub const SPACE: usize = 2 + 8 + 8;
}

#[account]
pub struct Platform {
    pub bump: u8,
    pub bump_chrt_mint: u8,
    pub authority: Pubkey,
    pub campaigns_count: u16,
    pub active_campaigns_capacity: u16,
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
    pub top: Vec<DonorRecord>,
    pub active_campaigns: Vec<CampaignRecord>,
}
impl Platform {
    pub const fn space(active_campaigns_capacity: u16) -> usize {
        (1 + 1 + 32 + 2 + 2 + 4 + 8 + 8 + 8 + 8 + 8 + 4 + 8 + 8 + 8 + 8)
            + (4 + PLATFORM_TOP_CAPACITY * DonorRecord::SPACE)
            + (4 + active_campaigns_capacity as usize * CampaignRecord::SPACE)
    }
}

#[account]
pub struct Campaign {
    pub bump: u8,
    pub bump_fee_exemption_vault: u8,
    pub bump_liquidation_vault: u8,
    pub authority: Pubkey,
    pub id: u16,
    pub top: Vec<DonorRecord>,
}
impl Campaign {
    pub const SPACE: usize = 1 + 1 + 1 + 32 + 2 + (4 + CAMPAIGN_TOP_CAPACITY * DonorRecord::SPACE);
}

#[account(zero_copy)]
#[repr(packed)]
pub struct Donor {
    pub bump: u8,
    pub authority: Pubkey,
    pub donations_sum: u64,
    pub incentivized_donations_sum: u64,
}

#[account]
pub struct Donations {
    pub bump: u8,
    pub donations_sum: u64,
}
impl Donations {
    pub const SPACE: usize = 1 + 8;
}

#[account]
pub struct Vault {
    pub bump: u8,
}
impl Vault {
    pub const SPACE: usize = 1;
}
