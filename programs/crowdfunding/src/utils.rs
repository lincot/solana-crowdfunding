use anchor_lang::{__private::CLOSED_ACCOUNT_DISCRIMINATOR, prelude::*};
use std::io::{Cursor, Write};

pub fn close(account: &AccountInfo, destination: &AccountInfo) -> Result<()> {
    **destination.try_borrow_mut_lamports()? += account.lamports();
    **account.try_borrow_mut_lamports()? = 0;

    let data: &mut [u8] = &mut account.try_borrow_mut_data()?;
    Cursor::new(data).write_all(&CLOSED_ACCOUNT_DISCRIMINATOR)?;

    Ok(())
}

pub fn save<'info, T: AccountSerialize + AccountDeserialize + Owner + Clone>(
    acc: &Account<'info, T>,
    info: &AccountInfo<'info>,
) -> Result<()> {
    acc.try_serialize(&mut *info.try_borrow_mut_data()?)
}

pub fn transfer_all_but_rent<'info>(
    from: &AccountInfo<'info>,
    to: &AccountInfo<'info>,
    space: usize,
) -> Result<()> {
    let rent = Rent::get()?.minimum_balance(space);
    **to.try_borrow_mut_lamports()? += from.lamports() - rent;
    **from.try_borrow_mut_lamports()? = rent;
    Ok(())
}

pub fn transfer<'info>(
    from: &AccountInfo<'info>,
    to: &AccountInfo<'info>,
    lamports: u64,
) -> Result<()> {
    **from.try_borrow_mut_lamports()? -= lamports;
    **to.try_borrow_mut_lamports()? += lamports;
    Ok(())
}
