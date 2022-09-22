use crate::{api::*, ctx::*, utils::*};
use anchor_lang::prelude::{Clock, ErrorCode};
use anchor_spl::token::TokenAccount;
use core::assert_matches::assert_matches;
use crowdfunding::{error::*, state::*};
use solana_program::instruction::InstructionError;
use solana_program_test::*;
use solana_sdk::{signer::Signer, transaction::TransactionError};
use spl_associated_token_account::get_associated_token_address;

pub async fn test_instructions() {
    let (mut ptc, ctx) = get_ptc_and_ctx().await;
    test_initialize(&mut ptc, &ctx).await;
    ctx.create_atas(&mut ptc).await;
    test_register_donor(&mut ptc, &ctx).await;
    test_start_campaign(&mut ptc, &ctx).await;
    test_donate(&mut ptc, &ctx).await;
    test_donate_with_referer(&mut ptc, &ctx).await;
    test_record_donors(&mut ptc, &ctx).await;
    test_drop_rewards(&mut ptc, &ctx).await;
    test_withdraw_donations(&mut ptc, &ctx).await;
    test_liquidate_campaign(&mut ptc, &ctx).await;
    test_stop_campaign(&mut ptc, &ctx).await;
    test_withdraw_fees(&mut ptc, &ctx).await;
}

async fn test_initialize(ptc: &mut ProgramTestContext, ctx: &Ctx) {
    let reward_cooldown = 10;
    let reward_amount = 1000;
    let fee_basis_points = 300;
    let fee_exemption_limit = 1000;
    let liquidation_limit = 2000;

    initialize(
        ptc,
        ctx,
        reward_cooldown,
        reward_amount,
        fee_basis_points,
        fee_exemption_limit,
        liquidation_limit,
    )
    .await
    .unwrap();

    let platform: Platform = fetch(ptc, ctx.platform).await.unwrap();

    assert_eq!(platform.authority, ctx.platform_authority.pubkey());
    let platform_reward_cooldown = platform.reward_cooldown;
    assert_eq!(platform_reward_cooldown, reward_cooldown);
    let platform_reward_amount = platform.reward_amount;
    assert_eq!(platform_reward_amount, reward_amount);
    let platform_fee_basis_points = platform.fee_basis_points;
    assert_eq!(platform_fee_basis_points, fee_basis_points);
    let platform_fee_exemption_limit = platform.fee_exemption_limit;
    assert_eq!(platform_fee_exemption_limit, fee_exemption_limit);
    let platform_liquidation_limit = platform.liquidation_limit;
    assert_eq!(platform_liquidation_limit, liquidation_limit);
}

async fn test_register_donor(ptc: &mut ProgramTestContext, ctx: &Ctx) {
    for donor in &ctx.donors {
        register_donor(ptc, ctx, donor).await.unwrap();
    }

    let donor: Donor = fetch(ptc, find_donor(ctx.donors[0].pubkey()))
        .await
        .unwrap();
    assert_eq!(donor.authority, ctx.donors[0].pubkey());
}

async fn test_start_campaign(ptc: &mut ProgramTestContext, ctx: &Ctx) {
    start_campaign(ptc, ctx).await.unwrap();

    let campaign: Campaign = fetch(ptc, find_campaign(0)).await.unwrap();
    assert_eq!(campaign.authority, ctx.campaign_authority.pubkey());
    let campaign_id = campaign.id;
    assert_eq!(campaign_id, 0);

    let (active_campaigns, len) = fetch_active_campaigns(ptc, ctx).await.unwrap();
    assert_eq!(
        active_campaigns[..len],
        [CampaignRecord {
            id: 0,
            donations_sum: 0,
            withdrawn_sum: 0,
        }]
    );

    start_campaign(ptc, ctx).await.unwrap();

    let (active_campaigns, len) = fetch_active_campaigns(ptc, ctx).await.unwrap();
    assert_eq!(
        active_campaigns[..len],
        [
            CampaignRecord {
                id: 0,
                donations_sum: 0,
                withdrawn_sum: 0,
            },
            CampaignRecord {
                id: 1,
                donations_sum: 0,
                withdrawn_sum: 0,
            }
        ]
    );
}

async fn test_donate(ptc: &mut ProgramTestContext, ctx: &Ctx) {
    donate(ptc, ctx, &ctx.donors[0], 0, 100).await.unwrap();

    let platform: Platform = fetch(ptc, ctx.platform).await.unwrap();
    let sum_of_all_donations = platform.sum_of_all_donations;
    assert_eq!(sum_of_all_donations, 97);
    let sum_of_active_campaign_donations = platform.sum_of_active_campaign_donations;
    assert_eq!(sum_of_active_campaign_donations, 97);

    let donor: Donor = fetch(ptc, find_donor(ctx.donors[0].pubkey()))
        .await
        .unwrap();
    assert_eq!(donor.donations_sum, 97);

    let donor_donations_to_campaign: Donations = fetch(
        ptc,
        find_donor_donations_to_campaign(ctx.donors[0].pubkey(), 0),
    )
    .await
    .unwrap();
    assert_eq!(donor_donations_to_campaign.donations_sum, 97);

    let total_donations_to_campaign: Donations = fetch(ptc, find_total_donations_to_campaign(0))
        .await
        .unwrap();
    assert_eq!(total_donations_to_campaign.donations_sum, 97);

    assert_eq!(get_sol_vault_balance(ptc, ctx).await.unwrap(), 97);
    assert_eq!(get_fee_vault_balance(ptc, ctx).await.unwrap(), 3);

    let (active_campaigns, len) = fetch_active_campaigns(ptc, ctx).await.unwrap();
    assert_eq!(
        active_campaigns[..len],
        [
            CampaignRecord {
                id: 0,
                donations_sum: 97,
                withdrawn_sum: 0,
            },
            CampaignRecord {
                id: 1,
                donations_sum: 0,
                withdrawn_sum: 0,
            }
        ]
    );

    let (platform_top, len) = fetch_platform_top(ptc, ctx).await.unwrap();
    assert_eq!(
        platform_top[..len],
        [DonorRecord {
            donor: ctx.donors[0].pubkey(),
            donations_sum: 97,
        }]
    );

    let (campaign_top, len) = fetch_campaign_top(ptc, 0).await.unwrap();
    assert_eq!(
        campaign_top[..len],
        [DonorRecord {
            donor: ctx.donors[0].pubkey(),
            donations_sum: 97,
        }]
    );
}

async fn test_donate_with_referer(ptc: &mut ProgramTestContext, ctx: &Ctx) {
    donate_with_referer(ptc, ctx, &ctx.donors[1], 0, 10000, ctx.donors[0].pubkey())
        .await
        .unwrap();

    let donor_chrt: TokenAccount = fetch(
        ptc,
        get_associated_token_address(&ctx.donors[0].pubkey(), &ctx.chrt_mint),
    )
    .await
    .unwrap();
    assert_eq!(donor_chrt.amount, 1);

    let platform: Platform = fetch(ptc, ctx.platform).await.unwrap();
    let sum_of_all_donations = platform.sum_of_all_donations;
    assert_eq!(sum_of_all_donations, 97 + 9700);
    let sum_of_active_campaign_donations = platform.sum_of_active_campaign_donations;
    assert_eq!(sum_of_active_campaign_donations, 97 + 9700);

    let donor: Donor = fetch(ptc, find_donor(ctx.donors[1].pubkey()))
        .await
        .unwrap();
    assert_eq!(donor.donations_sum, 9700);

    let donor_donations_to_campaign: Donations = fetch(
        ptc,
        find_donor_donations_to_campaign(ctx.donors[1].pubkey(), 0),
    )
    .await
    .unwrap();
    assert_eq!(donor_donations_to_campaign.donations_sum, 9700);

    let total_donations_to_campaign: Donations = fetch(ptc, find_total_donations_to_campaign(0))
        .await
        .unwrap();
    assert_eq!(total_donations_to_campaign.donations_sum, 97 + 9700);

    assert_eq!(get_sol_vault_balance(ptc, ctx).await.unwrap(), 97 + 9700);
    assert_eq!(get_fee_vault_balance(ptc, ctx).await.unwrap(), 3 + 300);

    let (active_campaigns, len) = fetch_active_campaigns(ptc, ctx).await.unwrap();
    assert_eq!(
        active_campaigns[..len],
        [
            CampaignRecord {
                id: 0,
                donations_sum: 97 + 9700,
                withdrawn_sum: 0,
            },
            CampaignRecord {
                id: 1,
                donations_sum: 0,
                withdrawn_sum: 0,
            }
        ]
    );

    let (platform_top, len) = fetch_platform_top(ptc, ctx).await.unwrap();
    assert_eq!(
        platform_top[..len],
        [
            DonorRecord {
                donor: ctx.donors[1].pubkey(),
                donations_sum: 9700,
            },
            DonorRecord {
                donor: ctx.donors[0].pubkey(),
                donations_sum: 97,
            }
        ]
    );

    let (campaign_top, len) = fetch_campaign_top(ptc, 0).await.unwrap();
    assert_eq!(
        campaign_top[..len],
        [
            DonorRecord {
                donor: ctx.donors[1].pubkey(),
                donations_sum: 9700,
            },
            DonorRecord {
                donor: ctx.donors[0].pubkey(),
                donations_sum: 97,
            }
        ]
    );
}

async fn test_record_donors(ptc: &mut ProgramTestContext, ctx: &Ctx) {
    record_donors(ptc, ctx).await.unwrap();

    let platform: Platform = fetch(ptc, ctx.platform).await.unwrap();
    let clock: Clock = ptc.banks_client.get_sysvar().await.unwrap();

    assert!(platform.reward_procedure_is_in_process);
    let last_reward_procedure_ts = platform.last_reward_procedure_ts;
    assert_eq!(last_reward_procedure_ts, clock.unix_timestamp as u32);
    let donors_recorded = platform.donors_recorded;
    assert_eq!(donors_recorded, ctx.donors.len() as u32);

    let (seasonal_top, len) = fetch_seasonal_top(ptc, ctx).await.unwrap();
    assert_eq!(
        seasonal_top[..len],
        [
            DonorRecord {
                donor: ctx.donors[1].pubkey(),
                donations_sum: 9700,
            },
            DonorRecord {
                donor: ctx.donors[0].pubkey(),
                donations_sum: 97,
            }
        ]
    );
}

async fn test_drop_rewards(ptc: &mut ProgramTestContext, ctx: &Ctx) {
    drop_rewards(ptc, ctx).await.unwrap();

    let platform: Platform = fetch(ptc, ctx.platform).await.unwrap();

    assert!(!platform.reward_procedure_is_in_process);
    let donors_recorded = platform.donors_recorded;
    assert_eq!(donors_recorded, 0);

    let (seasonal_top, len) = fetch_seasonal_top(ptc, ctx).await.unwrap();
    assert_eq!(seasonal_top[..len], []);

    let donor_chrt: TokenAccount = fetch(
        ptc,
        get_associated_token_address(&ctx.donors[0].pubkey(), &ctx.chrt_mint),
    )
    .await
    .unwrap();
    assert_eq!(donor_chrt.amount, 1001);

    let donor_chrt: TokenAccount = fetch(
        ptc,
        get_associated_token_address(&ctx.donors[1].pubkey(), &ctx.chrt_mint),
    )
    .await
    .unwrap();
    assert_eq!(donor_chrt.amount, 1000);

    for donor in [0, 1] {
        let donor: Donor = fetch(ptc, find_donor(ctx.donors[donor].pubkey()))
            .await
            .unwrap();
        assert_eq!(donor.rewarded_donations_sum, donor.donations_sum);
    }

    const CODE: u32 = 6000 + CrowdfundingError::RewardCooldown as u32;
    assert_matches!(
        record_donors(ptc, ctx).await,
        Err(BanksClientError::TransactionError(
            TransactionError::InstructionError(0, InstructionError::Custom(CODE))
        ))
    );
}

async fn test_withdraw_donations(ptc: &mut ProgramTestContext, ctx: &Ctx) {
    withdraw_donations(ptc, ctx, 0).await.unwrap();

    assert_eq!(get_sol_vault_balance(ptc, ctx).await.unwrap(), 0);

    let (active_campaigns, len) = fetch_active_campaigns(ptc, ctx).await.unwrap();
    assert_eq!(
        active_campaigns[..len],
        [
            CampaignRecord {
                id: 0,
                donations_sum: 97 + 9700,
                withdrawn_sum: 97 + 9700,
            },
            CampaignRecord {
                id: 1,
                donations_sum: 0,
                withdrawn_sum: 0,
            }
        ]
    );
}

async fn test_liquidate_campaign(ptc: &mut ProgramTestContext, ctx: &Ctx) {
    const CODE: u32 = 6000 + CrowdfundingError::NotEnoughCHRTInVault as u32;
    assert_matches!(
        liquidate_campaign(ptc, ctx, 0).await,
        Err(BanksClientError::TransactionError(
            TransactionError::InstructionError(0, InstructionError::Custom(CODE))
        ))
    );
}

async fn test_stop_campaign(ptc: &mut ProgramTestContext, ctx: &Ctx) {
    stop_campaign(ptc, ctx, 0).await.unwrap();

    const CODE: u32 = ErrorCode::AccountOwnedByWrongProgram as u32;
    assert_matches!(
        donate(ptc, ctx, &ctx.donors[5], 0, 1).await,
        Err(BanksClientError::TransactionError(
            TransactionError::InstructionError(0, InstructionError::Custom(CODE))
        ))
    );

    let (active_campaigns, len) = fetch_active_campaigns(ptc, ctx).await.unwrap();
    assert_eq!(
        active_campaigns[..len],
        [CampaignRecord {
            id: 1,
            donations_sum: 0,
            withdrawn_sum: 0,
        }]
    );

    let platform: Platform = fetch(ptc, ctx.platform).await.unwrap();
    let sum_of_active_campaign_donations = platform.sum_of_active_campaign_donations;
    assert_eq!(sum_of_active_campaign_donations, 0);

    stop_campaign(ptc, ctx, 1).await.unwrap();

    let (active_campaigns, len) = fetch_active_campaigns(ptc, ctx).await.unwrap();
    assert_eq!(active_campaigns[..len], []);
}

async fn test_withdraw_fees(ptc: &mut ProgramTestContext, ctx: &Ctx) {
    withdraw_fees(ptc, ctx).await.unwrap();

    assert_eq!(get_fee_vault_balance(ptc, ctx).await.unwrap(), 0);
}
