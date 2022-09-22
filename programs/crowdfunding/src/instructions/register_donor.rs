use crate::state::*;
use anchor_lang::prelude::*;
use core::mem::size_of;

#[derive(Accounts)]
pub struct RegisterDonor<'info> {
    #[account(mut, seeds = [b"platform"], bump)]
    platform: AccountLoader<'info, Platform>,
    #[account(
        init,
        payer = donor_authority,
        seeds = [b"donor", donor_authority.key().as_ref()],
        bump,
        space = 8 + size_of::<Donor>(),
    )]
    donor: AccountLoader<'info, Donor>,
    #[account(mut)]
    donor_authority: Signer<'info>,
    system_program: Program<'info, System>,
}

pub fn register_donor(ctx: Context<RegisterDonor>) -> Result<()> {
    let platform = &mut ctx.accounts.platform.load_mut()?;
    platform.donors_count += 1;

    let donor = &mut ctx.accounts.donor.load_init()?;
    donor.authority = ctx.accounts.donor_authority.key();

    Ok(())
}
