use crate::state::*;
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct RegisterDonor<'info> {
    #[account(
        init,
        payer = donor_authority,
        seeds = [b"donor", donor_authority.key().as_ref()],
        bump,
        space = 8 + Donor::SPACE,
    )]
    donor: Account<'info, Donor>,
    #[account(mut)]
    donor_authority: Signer<'info>,
    system_program: Program<'info, System>,
}

pub fn register_donor(ctx: Context<RegisterDonor>) -> Result<()> {
    ctx.accounts.donor.bump = *ctx.bumps.get("donor").unwrap();

    emit!(RegisterDonorEvent {});

    Ok(())
}

#[event]
struct RegisterDonorEvent {}
