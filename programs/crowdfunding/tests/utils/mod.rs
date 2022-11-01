use crate::ctx::*;
use anchor_lang::prelude::*;
use anchor_spl::token::spl_token::instruction::transfer;
use core::result::Result;
use solana_program_test::*;
use solana_sdk::{signature::Keypair, signer::Signer, transaction::Transaction};

pub async fn get_ptc_and_ctx() -> (ProgramTestContext, Ctx) {
    let pt = ProgramTest::new("crowdfunding", crowdfunding::ID, None);
    let mut ptc = pt.start_with_context().await;
    let ctx = Ctx::new();
    ctx.airdrop(&mut ptc).await;
    (ptc, ctx)
}

pub async fn transfer_tokens(
    ptc: &mut ProgramTestContext,
    from: &Pubkey,
    to: &Pubkey,
    amount: u64,
    signer: &Keypair,
) -> Result<(), BanksClientError> {
    ptc.banks_client
        .process_transaction(Transaction::new_signed_with_payer(
            &[transfer(
                &anchor_spl::token::ID,
                from,
                to,
                &signer.pubkey(),
                &[&signer.pubkey()],
                amount,
            )
            .unwrap()],
            Some(&signer.pubkey()),
            &[signer],
            ptc.last_blockhash,
        ))
        .await
}
