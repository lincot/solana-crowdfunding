use crate::state::*;
use anchor_lang::{
    prelude::*,
    solana_program::{program::invoke_signed, system_instruction},
};

#[derive(Accounts)]
pub struct WithdrawFees<'info> {
    #[account(seeds = [b"platform"], bump = platform.bump)]
    platform: Account<'info, Platform>,
    #[account(mut, address = platform.authority)]
    platform_authority: Signer<'info>,
    /// CHECK:
    #[account(mut, seeds = [b"fee_vault"], bump = platform.bump_fee_vault)]
    fee_vault: UncheckedAccount<'info>,
    system_program: Program<'info, System>,
}

pub fn withdraw_fees(ctx: Context<WithdrawFees>) -> Result<()> {
    invoke_signed(
        &system_instruction::transfer(
            ctx.accounts.fee_vault.key,
            ctx.accounts.platform_authority.key,
            ctx.accounts.fee_vault.lamports() - Rent::get()?.minimum_balance(0),
        ),
        &[
            ctx.accounts.fee_vault.to_account_info(),
            ctx.accounts.platform_authority.to_account_info(),
        ],
        &[&[b"fee_vault", &[ctx.accounts.platform.bump_fee_vault]]],
    )?;

    emit!(WithdrawFeesEvent {});

    Ok(())
}

#[event]
struct WithdrawFeesEvent {}
