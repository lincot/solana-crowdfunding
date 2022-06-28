use anchor_lang::prelude::*;

pub fn transfer(from: &AccountInfo, to: &AccountInfo, lamports: u64) -> Result<()> {
    **from.try_borrow_mut_lamports()? = (from.lamports())
        .checked_sub(lamports)
        .ok_or(ProgramError::InsufficientFunds)?;
    **to.try_borrow_mut_lamports()? += lamports;
    Ok(())
}

pub fn transfer_all_but_rent(from: &AccountInfo, to: &AccountInfo) -> Result<()> {
    let rent = Rent::get()?.minimum_balance(from.data_len());
    **to.try_borrow_mut_lamports()? += (from.lamports())
        .checked_sub(rent)
        .ok_or(ProgramError::InsufficientFunds)?;
    **from.try_borrow_mut_lamports()? = rent;
    Ok(())
}
