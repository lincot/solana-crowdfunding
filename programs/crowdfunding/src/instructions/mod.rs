pub use crate::instructions::{
    donate::*, drop_rewards::*, initialize::*, liquidate_campaign::*, record_donors::*,
    register_donor::*, start_campaign::*, stop_campaign::*, withdraw_donations::*,
    withdraw_fees::*,
};

pub mod donate;
pub mod drop_rewards;
pub mod initialize;
pub mod liquidate_campaign;
pub mod record_donors;
pub mod register_donor;
pub mod start_campaign;
pub mod stop_campaign;
pub mod withdraw_donations;
pub mod withdraw_fees;
