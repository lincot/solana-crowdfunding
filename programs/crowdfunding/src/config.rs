use anchor_lang::prelude::*;

#[constant]
pub const CHRT_DECIMALS: u8 = 3;
#[constant]
pub const PLATFORM_FEE_NUM: u64 = 3;
#[constant]
pub const PLATFORM_FEE_DENOM: u64 = 100;
#[constant]
pub const SEASONAL_TOP_CAPACITY: usize = 10;
#[constant]
pub const PLATFORM_TOP_CAPACITY: usize = 128;
#[constant]
pub const CAMPAIGN_TOP_CAPACITY: usize = 10;
#[constant]
pub const ACTIVE_CAMPAIGNS_CAPACITY: usize = 256;
