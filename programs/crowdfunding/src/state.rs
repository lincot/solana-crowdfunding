use crate::config::*;
use anchor_lang::prelude::*;

#[derive(AnchorDeserialize, Copy, Clone, Debug, Default, PartialEq, Eq)]
#[repr(packed)]
pub struct DonorRecord {
    pub donor: Pubkey,
    pub donations_sum: u64,
}

#[derive(AnchorDeserialize, Copy, Clone, Debug, Default, PartialEq, Eq)]
#[repr(packed)]
pub struct CampaignRecord {
    pub id: u16,
    pub donations_sum: u64,
    pub withdrawn_sum: u64,
}

#[account(zero_copy)]
#[repr(packed)]
pub struct Platform {
    pub authority: Pubkey,
    pub campaigns_count: u16,
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
    pub top: [DonorRecord; PLATFORM_TOP_CAPACITY],
    pub active_campaigns: [CampaignRecord; ACTIVE_CAMPAIGNS_CAPACITY],
    pub active_campaigns_count: u16,
}

#[account(zero_copy)]
#[derive(Debug)]
#[repr(packed)]
pub struct Campaign {
    pub authority: Pubkey,
    pub id: u16,
    pub top: [DonorRecord; CAMPAIGN_TOP_CAPACITY],
}

#[account(zero_copy)]
#[derive(Debug)]
pub struct Donor {
    pub authority: Pubkey,
    pub donations_sum: u64,
    pub incentivized_donations_sum: u64,
}

#[account(zero_copy)]
#[derive(Debug)]
pub struct Donations {
    pub donations_sum: u64,
}

#[account(zero_copy)]
#[derive(Debug)]
pub struct Vault {}
