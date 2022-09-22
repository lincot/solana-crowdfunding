use crate::state::*;
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

pub fn add_to_top(top: &mut [DonorRecord], donor_record: DonorRecord) {
    let top_len = top
        .iter()
        .position(|d| d.donor.to_bytes() == [0; 32])
        .unwrap_or(top.len());

    let cur_i = if let Some(cur_i) = top.iter().position(|d| d.donor == donor_record.donor) {
        // assign new sum
        top[cur_i] = donor_record;
        cur_i
    } else if top_len < top.len() {
        // push new donor
        top[top_len] = donor_record;
        top_len
    } else {
        // no space to push, so replace with last if eligible
        let last = top.last_mut().unwrap();
        if last.donations_sum > donor_record.donations_sum {
            return;
        }
        *last = donor_record;
        top.len() - 1
    };

    // sort donor
    let new_i = top[..cur_i].partition_point(|d| d.donations_sum >= donor_record.donations_sum);
    top[new_i..=cur_i].rotate_right(1);
}
