use crate::state::*;
use anchor_lang::prelude::*;
use core::mem::size_of;

#[derive(Accounts)]
pub struct RegisterDonor<'info> {
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
    let mut donor = ctx.accounts.donor.load_init()?;

    donor.bump = *ctx.bumps.get("donor").unwrap();
    donor.authority = ctx.accounts.donor_authority.key();

    emit!(RegisterDonorEvent {});

    Ok(())
}

#[event]
struct RegisterDonorEvent {}
