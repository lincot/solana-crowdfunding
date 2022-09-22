use crate::ctx::*;
use anchor_lang::{prelude::*, InstructionData};
use core::{mem::size_of, result::Result};
use crowdfunding::{config::*, state::*};
use solana_program::{instruction::Instruction, system_program, sysvar};
use solana_program_test::{BanksClientError, ProgramTestContext};
use solana_sdk::{signature::Keypair, signer::Signer, transaction::Transaction};
use spl_associated_token_account::get_associated_token_address;

pub async fn fetch<T: AccountDeserialize>(
    ptc: &mut ProgramTestContext,
    address: Pubkey,
) -> Result<T, BanksClientError> {
    T::try_deserialize(
        &mut &*ptc
            .banks_client
            .get_account(address)
            .await?
            .ok_or(BanksClientError::ClientError("Account not present"))?
            .data,
    )
    .map_err(|_| BanksClientError::ClientError("Failed to deserialize account"))
}

pub async fn fetch_active_campaigns(
    ptc: &mut ProgramTestContext,
    ctx: &Ctx,
) -> Result<([CampaignRecord; ACTIVE_CAMPAIGNS_CAPACITY], usize), BanksClientError> {
    let platform: Platform = fetch(ptc, ctx.platform).await?;
    Ok((
        platform.active_campaigns,
        platform.active_campaigns_count as _,
    ))
}

pub async fn fetch_platform_top(
    ptc: &mut ProgramTestContext,
    ctx: &Ctx,
) -> Result<([DonorRecord; PLATFORM_TOP_CAPACITY], usize), BanksClientError> {
    let platform: Platform = fetch(ptc, ctx.platform).await?;
    Ok((
        platform.top,
        platform
            .top
            .iter()
            .position(|d| d.donor.to_bytes() == [0; 32])
            .unwrap_or(platform.top.len()),
    ))
}

pub async fn fetch_seasonal_top(
    ptc: &mut ProgramTestContext,
    ctx: &Ctx,
) -> Result<([DonorRecord; SEASONAL_TOP_CAPACITY], usize), BanksClientError> {
    let platform: Platform = fetch(ptc, ctx.platform).await?;
    Ok((
        platform.seasonal_top,
        platform
            .seasonal_top
            .iter()
            .position(|d| d.donor.to_bytes() == [0; 32])
            .unwrap_or(platform.top.len()),
    ))
}

pub async fn fetch_campaign_top(
    ptc: &mut ProgramTestContext,
    campaign_id: u16,
) -> Result<([DonorRecord; CAMPAIGN_TOP_CAPACITY], usize), BanksClientError> {
    let campaign: Campaign = fetch(ptc, find_campaign(campaign_id)).await?;
    Ok((
        campaign.top,
        campaign
            .top
            .iter()
            .position(|d| d.donor.to_bytes() == [0; 32])
            .unwrap_or(campaign.top.len()),
    ))
}

async fn get_balance_without_rent<T>(
    ptc: &mut ProgramTestContext,
    address: Pubkey,
) -> Result<u64, BanksClientError> {
    Ok(ptc.banks_client.get_balance(address).await?
        - ptc
            .banks_client
            .get_rent()
            .await?
            .minimum_balance(8 + size_of::<T>()))
}

pub async fn get_sol_vault_balance(
    ptc: &mut ProgramTestContext,
    ctx: &Ctx,
) -> Result<u64, BanksClientError> {
    get_balance_without_rent::<Vault>(ptc, ctx.sol_vault).await
}

pub async fn get_fee_vault_balance(
    ptc: &mut ProgramTestContext,
    ctx: &Ctx,
) -> Result<u64, BanksClientError> {
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
    reward_cooldown: u32,
    reward_amount: u64,
    fee_basis_points: u16,
    fee_exemption_limit: u64,
    liquidation_limit: u64,
) -> Result<(), BanksClientError> {
    ptc.banks_client
        .process_transaction(Transaction::new_signed_with_payer(
            &[Instruction {
                program_id: crowdfunding::ID,
                data: crowdfunding::instruction::Initialize {
                    reward_cooldown,
                    reward_amount,
                    fee_basis_points,
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
    ctx: &Ctx,
    donor_authority: &Keypair,
) -> Result<(), BanksClientError> {
    ptc.banks_client
        .process_transaction(Transaction::new_signed_with_payer(
            &[Instruction {
                program_id: crowdfunding::ID,
                data: crowdfunding::instruction::RegisterDonor {}.data(),
                accounts: crowdfunding::accounts::RegisterDonor {
                    platform: ctx.platform,
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
    let platform: Platform = fetch(ptc, ctx.platform).await?;
    let id = platform.campaigns_count;

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

pub async fn record_donors(
    ptc: &mut ProgramTestContext,
    ctx: &Ctx,
) -> Result<(), BanksClientError> {
    let clock: Clock = ptc.banks_client.get_sysvar().await.unwrap();
    ptc.warp_to_slot(clock.slot + 1).unwrap();

    let mut accounts = crowdfunding::accounts::RecordDonors {
        platform: ctx.platform,
    }
    .to_account_metas(None);

    for donor in &ctx.donors {
        accounts.push(AccountMeta {
            pubkey: find_donor(donor.pubkey()),
            is_signer: false,
            is_writable: true,
        });
    }

    ptc.banks_client
        .process_transaction(Transaction::new_signed_with_payer(
            &[Instruction {
                program_id: crowdfunding::ID,
                data: crowdfunding::instruction::RecordDonors {}.data(),
                accounts,
            }],
            Some(&ctx.platform_authority.pubkey()),
            &[&ctx.platform_authority],
            ptc.last_blockhash,
        ))
        .await
}

pub async fn drop_rewards(ptc: &mut ProgramTestContext, ctx: &Ctx) -> Result<(), BanksClientError> {
    let mut accounts = crowdfunding::accounts::DropRewards {
        platform: ctx.platform,
        chrt_mint: ctx.chrt_mint,
        token_program: anchor_spl::token::ID,
    }
    .to_account_metas(None);

    let platform: Platform = fetch(ptc, ctx.platform).await?;
    let seasonal_top = platform.seasonal_top;

    for donor in seasonal_top {
        accounts.push(AccountMeta {
            pubkey: find_donor(donor.donor),
            is_signer: false,
            is_writable: true,
        });
        accounts.push(AccountMeta {
            pubkey: get_associated_token_address(&donor.donor, &ctx.chrt_mint),
            is_signer: false,
            is_writable: true,
        });
    }

    ptc.banks_client
        .process_transaction(Transaction::new_signed_with_payer(
            &[Instruction {
                program_id: crowdfunding::ID,
                data: crowdfunding::instruction::DropRewards {}.data(),
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
    let clock: Clock = ptc.banks_client.get_sysvar().await.unwrap();
    ptc.warp_to_slot(clock.slot + 1).unwrap();

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
