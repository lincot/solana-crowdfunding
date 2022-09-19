use anchor_lang::prelude::*;
use solana_program::system_instruction;
use solana_program_test::*;
use solana_sdk::{signature::Keypair, signer::Signer, transaction::Transaction};
use spl_associated_token_account::instruction::create_associated_token_account;

pub const DONORS_LEN: usize = 15;

pub struct Ctx {
    pub platform_authority: Keypair,
    pub campaign_authority: Keypair,
    pub donors: [Keypair; DONORS_LEN],
    pub platform: Pubkey,
    pub fee_vault: Pubkey,
    pub sol_vault: Pubkey,
    pub chrt_mint: Pubkey,
}

impl Ctx {
    pub fn new() -> Self {
        let platform = Pubkey::find_program_address(&[b"platform"], &crowdfunding::ID).0;
        let fee_vault = Pubkey::find_program_address(&[b"fee_vault"], &crowdfunding::ID).0;
        let sol_vault = Pubkey::find_program_address(&[b"sol_vault"], &crowdfunding::ID).0;
        let chrt_mint = Pubkey::find_program_address(&[b"chrt_mint"], &crowdfunding::ID).0;
        Ctx {
            platform_authority: Keypair::new(),
            campaign_authority: Keypair::new(),
            donors: [
                Keypair::new(),
                Keypair::new(),
                Keypair::new(),
                Keypair::new(),
                Keypair::new(),
                Keypair::new(),
                Keypair::new(),
                Keypair::new(),
                Keypair::new(),
                Keypair::new(),
                Keypair::new(),
                Keypair::new(),
                Keypair::new(),
                Keypair::new(),
                Keypair::new(),
            ],
            platform,
            fee_vault,
            sol_vault,
            chrt_mint,
        }
    }

    pub async fn airdrop(&self, ptc: &mut ProgramTestContext) {
        let mut instructions = heapless::Vec::<_, { DONORS_LEN + 2 }>::new();
        for to_pubkey in self.donors.iter().map(|d| d.pubkey()).chain([
            self.platform_authority.pubkey(),
            self.campaign_authority.pubkey(),
        ]) {
            instructions
                .push(system_instruction::transfer(
                    &ptc.payer.pubkey(),
                    &to_pubkey,
                    200_000_000,
                ))
                .unwrap();
        }
        ptc.banks_client
            .process_transaction(Transaction::new_signed_with_payer(
                &instructions,
                Some(&ptc.payer.pubkey()),
                &[&ptc.payer],
                ptc.last_blockhash,
            ))
            .await
            .unwrap();
    }

    pub async fn create_atas(&self, ptc: &mut ProgramTestContext) {
        let mut instructions = heapless::Vec::<_, DONORS_LEN>::new();
        for wallet_address in self.donors.iter().map(|d| d.pubkey()) {
            instructions
                .push(create_associated_token_account(
                    &ptc.payer.pubkey(),
                    &wallet_address,
                    &self.chrt_mint,
                ))
                .unwrap();
        }
        ptc.banks_client
            .process_transaction(Transaction::new_signed_with_payer(
                &instructions,
                Some(&ptc.payer.pubkey()),
                &[&ptc.payer],
                ptc.last_blockhash,
            ))
            .await
            .unwrap();
    }
}
