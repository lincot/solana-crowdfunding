use crate::{error::*, state::*, utils::*};
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct RecordDonors<'info> {
    #[account(mut, seeds = [b"platform"], bump)]
    platform: AccountLoader<'info, Platform>,
}

pub fn record_donors(ctx: Context<RecordDonors>) -> Result<()> {
    let now = Clock::get()?.unix_timestamp as _;
    let platform = &mut ctx.accounts.platform.load_mut()?;
    if !platform.reward_procedure_is_in_process {
        if now - platform.last_reward_procedure_ts < platform.reward_cooldown {
            return err!(CrowdfundingError::RewardCooldown);
        }
        platform.reward_procedure_is_in_process = true;
        platform.last_reward_procedure_ts = now;
    }

    for i in 0..ctx.remaining_accounts.len() {
        for j in i + 1..ctx.remaining_accounts.len() {
            if ctx.remaining_accounts[i].key() == ctx.remaining_accounts[j].key() {
                return err!(CrowdfundingError::CannotRecordTwice);
            }
        }
    }

    for donor in ctx.remaining_accounts {
        let donor = AccountLoader::<Donor>::try_from(&donor)?;
        let donor = &mut donor.load_mut()?;
        if donor.last_record_ts >= platform.last_reward_procedure_ts {
            return err!(CrowdfundingError::CannotRecordTwice);
        }
        donor.last_record_ts = now;

        if donor.donations_sum != donor.rewarded_donations_sum {
            add_to_top(
                &mut platform.seasonal_top,
                DonorRecord {
                    donor: donor.authority,
                    donations_sum: donor.donations_sum,
                },
            );
        }
    }

    platform.donors_recorded += ctx.remaining_accounts.len() as u32;

    Ok(())
}
