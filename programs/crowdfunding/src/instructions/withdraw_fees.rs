use crate::{state::*, utils::*};
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct WithdrawFees<'info> {
    #[account(seeds = [b"platform"], bump = platform.bump)]
    platform: Account<'info, Platform>,
    #[account(mut, address = platform.authority)]
    platform_authority: Signer<'info>,
    /// CHECK:
    #[account(mut, seeds = [b"fee_vault"], bump = platform.bump_fee_vault)]
    fee_vault: UncheckedAccount<'info>,
}

pub fn withdraw_fees(ctx: Context<WithdrawFees>) -> Result<()> {
    transfer_all_lamports(
        &ctx.accounts.fee_vault.to_account_info(),
        &ctx.accounts.platform_authority.to_account_info(),
    )?;

    Ok(())
}
