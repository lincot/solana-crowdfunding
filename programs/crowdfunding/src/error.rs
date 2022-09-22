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
    #[msg("Reward cooldown time has not passed")]
    RewardCooldown,
    /// 6004 0x1774
    #[msg("Donors can only be recorded for reward once")]
    CannotRecordTwice,
    /// 6005 0x1775
    #[msg("Campaign is not active")]
    CampaignInactive,
    /// 6006 0x1776
    #[msg("All the donors must be recorded before reward drop")]
    NotAllDonorsRecorded,
    /// 6007 0x1777
    #[msg("Donors passed for reward must match those in seasonal top")]
    IncorrectSeasonalTop,
    /// 6008 0x1778
    #[msg("Cannot donate during reward procedure")]
    RewardProcedureInProcess,
}
