use anchor_lang::prelude::*;

#[error_code]
pub enum CrowdfundingError {
    /// 6000 0x1770
    #[msg("Campaign's vault does not have enough CHRT for operation")]
    NotEnoughCHRTInVault,
    /// 6001 0x1771
    #[msg("Platform's limit of active campaigns is reached")]
    ActiveCampaignsLimit,
    /// 6002 0x1772
    #[msg("Referring yourself is not allowed")]
    CannotReferYourself,
    /// 6003 0x1773
    #[msg("Incentive cooldown time has not passed")]
    IncentiveCooldown,
    /// 6004 0x1774
    #[msg("Donor is not eligible for incentive")]
    NotEligibleForIncentive,
    /// 6005 0x1775
    #[msg("Seasonal top contains duplicates")]
    DuplicateInTop,
}
