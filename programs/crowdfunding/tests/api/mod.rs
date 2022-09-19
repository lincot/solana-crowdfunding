use crate::ctx::*;
use anchor_lang::{prelude::*, InstructionData};
use core::{cmp::Reverse, mem::size_of, result::Result};
use crowdfunding::{config::*, state::*};
use solana_program::{instruction::Instruction, system_program, sysvar};
use solana_program_test::{BanksClientError, ProgramTestContext};
use solana_sdk::{signature::Keypair, signer::Signer, transaction::Transaction};
use spl_associated_token_account::get_associated_token_address;

pub async fn fetch<T: AccountDeserialize>(ptc: &mut ProgramTestContext, address: Pubkey) -> T {
    T::try_deserialize(
        &mut &*ptc
            .banks_client
            .get_account(address)
            .await
            .unwrap()
            .unwrap()
            .data,
    )
    .unwrap()
}

pub async fn fetch_active_campaigns(
    ptc: &mut ProgramTestContext,
    ctx: &Ctx,
) -> ([CampaignRecord; ACTIVE_CAMPAIGNS_CAPACITY], usize) {
    let platform_data: Platform = fetch(ptc, ctx.platform).await;
    (
        platform_data.active_campaigns,
        platform_data.active_campaigns_count as _,
    )
}

pub async fn fetch_platform_top(
    ptc: &mut ProgramTestContext,
    ctx: &Ctx,
) -> ([DonorRecord; PLATFORM_TOP_CAPACITY], usize) {
    let platform_data: Platform = fetch(ptc, ctx.platform).await;
    (
        platform_data.top,
        platform_data
            .top
            .iter()
            .position(|d| d.donor.to_bytes() == [0; 32])
            .unwrap_or(platform_data.top.len()),
    )
}

pub async fn fetch_campaign_top(
    ptc: &mut ProgramTestContext,
    campaign_id: u16,
) -> ([DonorRecord; CAMPAIGN_TOP_CAPACITY], usize) {
    let campaign_data: Campaign = fetch(ptc, find_campaign(campaign_id)).await;
    (
        campaign_data.top,
        campaign_data
            .top
            .iter()
            .position(|d| d.donor.to_bytes() == [0; 32])
            .unwrap_or(campaign_data.top.len()),
    )
}

pub async fn get_seasonal_top(
    ptc: &mut ProgramTestContext,
    ctx: &Ctx,
) -> heapless::Vec<Pubkey, DONORS_LEN> {
    let mut donors = heapless::Vec::<_, DONORS_LEN>::new();
    for donor in &ctx.donors {
        let donor: Donor = fetch(ptc, find_donor(donor.pubkey())).await;
        if donor.donations_sum != donor.incentivized_donations_sum {
            donors.push(donor).unwrap();
        }
    }
    donors.sort_by_key(|d| Reverse(d.donations_sum - d.incentivized_donations_sum));

    let mut res = heapless::Vec::new();
    for donor in &donors[..donors.len().min(SEASONAL_TOP_CAPACITY)] {
        res.push(donor.authority).unwrap();
    }
    res
}

async fn get_balance_without_rent<T>(ptc: &mut ProgramTestContext, address: Pubkey) -> u64 {
    ptc.banks_client.get_balance(address).await.unwrap()
        - ptc
            .banks_client
            .get_rent()
            .await
            .unwrap()
            .minimum_balance(8 + size_of::<T>())
}

pub async fn get_sol_vault_balance(ptc: &mut ProgramTestContext, ctx: &Ctx) -> u64 {
    get_balance_without_rent::<Vault>(ptc, ctx.sol_vault).await
}

pub async fn get_fee_vault_balance(ptc: &mut ProgramTestContext, ctx: &Ctx) -> u64 {
    get_balance_without_rent::<Vault>(ptc, ctx.fee_vault).await
}

fn find_pda(seeds: &[&[u8]]) -> Pubkey {
    Pubkey::find_program_address(seeds, &crowdfunding::ID).0
}

pub fn find_donor(donor_authority: Pubkey) -> Pubkey {
    find_pda(&[b"donor", &donor_authority.to_bytes()])
}

pub fn find_donor_donations_to_campaign(donor_authority: Pubkey, campaign_id: u16) -> Pubkey {
    find_pda(&[
        b"donations",
        &donor_authority.to_bytes(),
        &campaign_id.to_le_bytes(),
    ])
}

pub fn find_campaign(campaign_id: u16) -> Pubkey {
    find_pda(&[b"campaign", &campaign_id.to_le_bytes()])
}

pub fn find_total_donations_to_campaign(campaign_id: u16) -> Pubkey {
    find_pda(&[b"donations", &campaign_id.to_le_bytes()])
}

pub fn find_fee_exemption_vault(campaign_id: u16) -> Pubkey {
    find_pda(&[b"fee_exemption_vault", &campaign_id.to_le_bytes()])
}

pub fn find_liquidation_vault(campaign_id: u16) -> Pubkey {
    find_pda(&[b"liquidation_vault", &campaign_id.to_le_bytes()])
}

pub async fn initialize(
    ptc: &mut ProgramTestContext,
    ctx: &Ctx,
    incentive_cooldown: u32,
    incentive_amount: u64,
    platform_fee_num: u64,
    platform_fee_denom: u64,
    fee_exemption_limit: u64,
    liquidation_limit: u64,
) -> Result<(), BanksClientError> {
    ptc.banks_client
        .process_transaction(Transaction::new_signed_with_payer(
            &[Instruction {
                program_id: crowdfunding::ID,
                data: crowdfunding::instruction::Initialize {
                    incentive_cooldown,
                    incentive_amount,
                    platform_fee_num,
                    platform_fee_denom,
                    fee_exemption_limit,
                    liquidation_limit,
                }
                .data(),
                accounts: crowdfunding::accounts::Initialize {
                    platform: ctx.platform,
                    platform_authority: ctx.platform_authority.pubkey(),
                    fee_vault: ctx.fee_vault,
                    sol_vault: ctx.sol_vault,
                    chrt_mint: ctx.chrt_mint,
                    rent: sysvar::rent::id(),
                    token_program: anchor_spl::token::ID,
                    system_program: system_program::ID,
                }
                .to_account_metas(None),
            }],
            Some(&ctx.platform_authority.pubkey()),
            &[&ctx.platform_authority],
            ptc.last_blockhash,
        ))
        .await
}

pub async fn register_donor(
    ptc: &mut ProgramTestContext,
    donor_authority: &Keypair,
) -> Result<(), BanksClientError> {
    ptc.banks_client
        .process_transaction(Transaction::new_signed_with_payer(
            &[Instruction {
                program_id: crowdfunding::ID,
                data: crowdfunding::instruction::RegisterDonor {}.data(),
                accounts: crowdfunding::accounts::RegisterDonor {
                    donor: find_donor(donor_authority.pubkey()),
                    donor_authority: donor_authority.pubkey(),
                    system_program: system_program::ID,
                }
                .to_account_metas(None),
            }],
            Some(&donor_authority.pubkey()),
            &[donor_authority],
            ptc.last_blockhash,
        ))
        .await
}

pub async fn start_campaign(
    ptc: &mut ProgramTestContext,
    ctx: &Ctx,
) -> Result<(), BanksClientError> {
    let platform_data: Platform = fetch(ptc, ctx.platform).await;
    let id = platform_data.campaigns_count;

    ptc.banks_client
        .process_transaction(Transaction::new_signed_with_payer(
            &[Instruction {
                program_id: crowdfunding::ID,
                data: crowdfunding::instruction::StartCampaign {}.data(),
                accounts: crowdfunding::accounts::StartCampaign {
                    platform: ctx.platform,
                    chrt_mint: ctx.chrt_mint,
                    campaign: find_campaign(id),
                    campaign_authority: ctx.campaign_authority.pubkey(),
                    total_donations_to_campaign: find_total_donations_to_campaign(id),
                    fee_exemption_vault: find_fee_exemption_vault(id),
                    liquidation_vault: find_liquidation_vault(id),
                    rent: sysvar::rent::id(),
                    token_program: anchor_spl::token::ID,
                    system_program: system_program::ID,
                }
                .to_account_metas(None),
            }],
            Some(&ctx.campaign_authority.pubkey()),
            &[&ctx.campaign_authority],
            ptc.last_blockhash,
        ))
        .await
}

pub async fn donate(
    ptc: &mut ProgramTestContext,
    ctx: &Ctx,
    donor_authority: &Keypair,
    campaign_id: u16,
    amount: u64,
) -> Result<(), BanksClientError> {
    ptc.banks_client
        .process_transaction(Transaction::new_signed_with_payer(
            &[Instruction {
                program_id: crowdfunding::ID,
                data: crowdfunding::instruction::Donate { amount }.data(),
                accounts: crowdfunding::accounts::Donate {
                    platform: ctx.platform,
                    fee_vault: ctx.fee_vault,
                    sol_vault: ctx.sol_vault,
                    campaign: find_campaign(campaign_id),
                    total_donations_to_campaign: find_total_donations_to_campaign(campaign_id),
                    fee_exemption_vault: find_fee_exemption_vault(campaign_id),
                    donor: find_donor(donor_authority.pubkey()),
                    donor_authority: donor_authority.pubkey(),
                    donor_donations_to_campaign: find_donor_donations_to_campaign(
                        donor_authority.pubkey(),
                        campaign_id,
                    ),
                    system_program: system_program::ID,
                }
                .to_account_metas(None),
            }],
            Some(&donor_authority.pubkey()),
            &[donor_authority],
            ptc.last_blockhash,
        ))
        .await
}

pub async fn donate_with_referer(
    ptc: &mut ProgramTestContext,
    ctx: &Ctx,
    donor_authority: &Keypair,
    campaign_id: u16,
    amount: u64,
    referer_authority: Pubkey,
) -> Result<(), BanksClientError> {
    ptc.banks_client
        .process_transaction(Transaction::new_signed_with_payer(
            &[Instruction {
                program_id: crowdfunding::ID,
                data: crowdfunding::instruction::DonateWithReferer { amount }.data(),
                accounts: crowdfunding::accounts::DonateWithReferer {
                    donate: crowdfunding::accounts::Donate {
                        platform: ctx.platform,
                        fee_vault: ctx.fee_vault,
                        sol_vault: ctx.sol_vault,
                        campaign: find_campaign(campaign_id),
                        total_donations_to_campaign: find_total_donations_to_campaign(campaign_id),
                        fee_exemption_vault: find_fee_exemption_vault(campaign_id),
                        donor: find_donor(donor_authority.pubkey()),
                        donor_authority: donor_authority.pubkey(),
                        donor_donations_to_campaign: find_donor_donations_to_campaign(
                            donor_authority.pubkey(),
                            campaign_id,
                        ),
                        system_program: system_program::ID,
                    },
                    chrt_mint: ctx.chrt_mint,
                    referer: find_donor(referer_authority),
                    referer_authority,
                    referer_chrt: get_associated_token_address(&referer_authority, &ctx.chrt_mint),
                    token_program: anchor_spl::token::ID,
                }
                .to_account_metas(None),
            }],
            Some(&donor_authority.pubkey()),
            &[donor_authority],
            ptc.last_blockhash,
        ))
        .await
}

pub async fn incentivize(ptc: &mut ProgramTestContext, ctx: &Ctx) -> Result<(), BanksClientError> {
    let mut accounts = crowdfunding::accounts::Incentivize {
        platform: ctx.platform,
        platform_authority: ctx.platform_authority.pubkey(),
        chrt_mint: ctx.chrt_mint,
        token_program: anchor_spl::token::ID,
    }
    .to_account_metas(None);

    for donor in get_seasonal_top(ptc, ctx).await {
        accounts.push(AccountMeta {
            pubkey: find_donor(donor),
            is_signer: false,
            is_writable: true,
        });
        accounts.push(AccountMeta {
            pubkey: get_associated_token_address(&donor, &ctx.chrt_mint),
            is_signer: false,
            is_writable: true,
        });
    }

    ptc.banks_client
        .process_transaction(Transaction::new_signed_with_payer(
            &[Instruction {
                program_id: crowdfunding::ID,
                data: crowdfunding::instruction::Incentivize {}.data(),
                accounts,
            }],
            Some(&ctx.platform_authority.pubkey()),
            &[&ctx.platform_authority],
            ptc.last_blockhash,
        ))
        .await
}

pub async fn withdraw_donations(
    ptc: &mut ProgramTestContext,
    ctx: &Ctx,
    campaign_id: u16,
) -> Result<(), BanksClientError> {
    ptc.banks_client
        .process_transaction(Transaction::new_signed_with_payer(
            &[Instruction {
                program_id: crowdfunding::ID,
                data: crowdfunding::instruction::WithdrawDonations {}.data(),
                accounts: crowdfunding::accounts::WithdrawDonations {
                    platform: ctx.platform,
                    sol_vault: ctx.sol_vault,
                    campaign: find_campaign(campaign_id),
                    campaign_authority: ctx.campaign_authority.pubkey(),
                }
                .to_account_metas(None),
            }],
            Some(&ctx.campaign_authority.pubkey()),
            &[&ctx.campaign_authority],
            ptc.last_blockhash,
        ))
        .await
}

pub async fn liquidate_campaign(
    ptc: &mut ProgramTestContext,
    ctx: &Ctx,
    campaign_id: u16,
) -> Result<(), BanksClientError> {
    ptc.banks_client
        .process_transaction(Transaction::new_signed_with_payer(
            &[Instruction {
                program_id: crowdfunding::ID,
                data: crowdfunding::instruction::LiquidateCampaign {}.data(),
                accounts: crowdfunding::accounts::LiquidateCampaign {
                    platform: ctx.platform,
                    fee_vault: ctx.fee_vault,
                    sol_vault: ctx.sol_vault,
                    chrt_mint: ctx.chrt_mint,
                    campaign: find_campaign(campaign_id),
                    campaign_authority: ctx.campaign_authority.pubkey(),
                    fee_exemption_vault: find_fee_exemption_vault(campaign_id),
                    liquidation_vault: find_liquidation_vault(campaign_id),
                    token_program: anchor_spl::token::ID,
                    system_program: system_program::ID,
                }
                .to_account_metas(None),
            }],
            Some(&ptc.payer.pubkey()),
            &[&ptc.payer],
            ptc.last_blockhash,
        ))
        .await
}

pub async fn stop_campaign(
    ptc: &mut ProgramTestContext,
    ctx: &Ctx,
    campaign_id: u16,
) -> Result<(), BanksClientError> {
    ptc.banks_client
        .process_transaction(Transaction::new_signed_with_payer(
            &[Instruction {
                program_id: crowdfunding::ID,
                data: crowdfunding::instruction::StopCampaign {}.data(),
                accounts: crowdfunding::accounts::StopCampaign {
                    platform: ctx.platform,
                    sol_vault: ctx.sol_vault,
                    chrt_mint: ctx.chrt_mint,
                    campaign: find_campaign(campaign_id),
                    campaign_authority: ctx.campaign_authority.pubkey(),
                    fee_exemption_vault: find_fee_exemption_vault(campaign_id),
                    liquidation_vault: find_liquidation_vault(campaign_id),
                    token_program: anchor_spl::token::ID,
                    system_program: system_program::ID,
                }
                .to_account_metas(None),
            }],
            Some(&ctx.campaign_authority.pubkey()),
            &[&ctx.campaign_authority],
            ptc.last_blockhash,
        ))
        .await
}

pub async fn withdraw_fees(
    ptc: &mut ProgramTestContext,
    ctx: &Ctx,
) -> Result<(), BanksClientError> {
    ptc.banks_client
        .process_transaction(Transaction::new_signed_with_payer(
            &[Instruction {
                program_id: crowdfunding::ID,
                data: crowdfunding::instruction::WithdrawFees {}.data(),
                accounts: crowdfunding::accounts::WithdrawFees {
                    platform: ctx.platform,
                    platform_authority: ctx.platform_authority.pubkey(),
                    fee_vault: ctx.fee_vault,
                }
                .to_account_metas(None),
            }],
            Some(&ctx.platform_authority.pubkey()),
            &[&ctx.platform_authority],
            ptc.last_blockhash,
        ))
        .await
}
