use anchor_lang::prelude::*;

#[error_code]
pub enum CrowdfundingError {
    /// 6000 0x1770
    #[msg("Campaign's vault does not have enough CHRT for operation")]
    NotEnoughCHRTInVault,
    /// 6001 0x1771
    #[msg("CHRT token account should be provided for every top donor")]
    CHRTNotProvided,
    /// 6002 0x1772
    #[msg("Platform's limit of liquidations is exceeded")]
    LiquidationsLimit,
    /// 6003 0x1773
    #[msg("Referring yourself is not allowed")]
    CannotReferYourself,
}
