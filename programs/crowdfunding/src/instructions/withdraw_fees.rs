use crate::{state::*, utils::*};
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct WithdrawFees<'info> {
    #[account(seeds = [b"platform"], bump)]
    platform: AccountLoader<'info, Platform>,
    #[account(mut, address = platform.load()?.authority)]
    platform_authority: Signer<'info>,
    #[account(mut, seeds = [b"fee_vault"], bump)]
    fee_vault: AccountLoader<'info, Vault>,
}

pub fn withdraw_fees(ctx: Context<WithdrawFees>) -> Result<()> {
    transfer_all_but_rent(
        &ctx.accounts.fee_vault.to_account_info(),
        &ctx.accounts.platform_authority.to_account_info(),
    )?;

    Ok(())
}
