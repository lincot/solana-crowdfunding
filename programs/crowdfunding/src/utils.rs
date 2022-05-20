use anchor_lang::{__private::CLOSED_ACCOUNT_DISCRIMINATOR, prelude::*};
use std::io::{Cursor, Write};

pub fn transfer_lamports(
    account: &AccountInfo,
    destination: &AccountInfo,
    lamports: u64,
) -> Result<()> {
    account
        .try_borrow_mut_lamports()?
        .checked_sub(lamports)
        .ok_or(ErrorCode::AccountDidNotDeserialize)?;
    **destination.try_borrow_mut_lamports()? += lamports;
    Ok(())
}

pub fn transfer_all_lamports(account: &AccountInfo, destination: &AccountInfo) -> Result<()> {
    **destination.try_borrow_mut_lamports()? += account.lamports();
    **account.try_borrow_mut_lamports()? = 0;
    Ok(())
}

pub fn close(account: &AccountInfo, destination: &AccountInfo) -> Result<()> {
    transfer_all_lamports(account, destination)?;

    let data: &mut [u8] = &mut account.try_borrow_mut_data()?;
    Cursor::new(data).write_all(&CLOSED_ACCOUNT_DISCRIMINATOR)?;

    Ok(())
}
