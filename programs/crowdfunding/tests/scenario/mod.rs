use crate::{api::*, ctx::*, utils::*};
use anchor_lang::prelude::ErrorCode;
use anchor_spl::token::TokenAccount;
use core::assert_matches::assert_matches;
use crowdfunding::state::*;
use solana_program::instruction::InstructionError;
use solana_program_test::*;
use solana_sdk::{signer::Signer, transaction::TransactionError};
use spl_associated_token_account::get_associated_token_address;

pub async fn scenario_test() {
    let (mut ptc, ctx) = get_ptc_and_ctx().await;
    initializes(&mut ptc, &ctx).await;
    ctx.create_atas(&mut ptc).await;
    registers_donors(&mut ptc, &ctx).await;
    starts_campaign(&mut ptc, &ctx).await;
    donates(&mut ptc, &ctx).await;
    withdraws_donations(&mut ptc, &ctx).await;
    withdraws_fees(&mut ptc, &ctx).await;
    incentivizes(&mut ptc, &ctx).await;
    exempts_from_fees(&mut ptc, &ctx).await;
    starts_more_campaigns_and_donates(&mut ptc, &ctx).await;
    liquidates_campaign(&mut ptc, &ctx).await;
    withdraws_donations_that_came_from_liquidation(&mut ptc, &ctx).await;
    sorts_top_with_more_than_10_donors(&mut ptc, &ctx).await;
}

async fn initializes(ptc: &mut ProgramTestContext, ctx: &Ctx) {
    initialize(ptc, ctx, 0, 10000, 300, 1000, 2000)
        .await
        .unwrap();
}

async fn registers_donors(ptc: &mut ProgramTestContext, ctx: &Ctx) {
    for donor in &ctx.donors {
        register_donor(ptc, donor).await.unwrap();
    }
}

async fn starts_campaign(ptc: &mut ProgramTestContext, ctx: &Ctx) {
    start_campaign(ptc, ctx).await.unwrap();

    let (active_campaigns, len) = fetch_active_campaigns(ptc, ctx).await;
    assert_eq!(
        active_campaigns[..len],
        [CampaignRecord {
            id: 0,
            donations_sum: 0,
            withdrawn_sum: 0,
        }]
    );
}

async fn donates(ptc: &mut ProgramTestContext, ctx: &Ctx) {
    donate(ptc, ctx, &ctx.donors[0], 0, 100).await.unwrap();
    let (platform_top, len) = fetch_platform_top(ptc, ctx).await;
    assert_eq!(
        platform_top[..len],
        [DonorRecord {
            donor: ctx.donors[0].pubkey(),
            donations_sum: 97,
        }]
    );

    donate(ptc, ctx, &ctx.donors[0], 0, 1000).await.unwrap();
    let (platform_top, len) = fetch_platform_top(ptc, ctx).await;
    assert_eq!(
        platform_top[..len],
        [DonorRecord {
            donor: ctx.donors[0].pubkey(),
            donations_sum: 97 + 970,
        },]
    );

    donate(ptc, ctx, &ctx.donors[2], 0, 10000).await.unwrap();
    let (platform_top, len) = fetch_platform_top(ptc, ctx).await;
    assert_eq!(
        &platform_top[..len],
        [
            DonorRecord {
                donor: ctx.donors[2].pubkey(),
                donations_sum: 9700,
            },
            DonorRecord {
                donor: ctx.donors[0].pubkey(),
                donations_sum: 97 + 970,
            },
        ]
    );

    donate(ptc, ctx, &ctx.donors[3], 0, 1).await.unwrap();
    let (platform_top, len) = fetch_platform_top(ptc, ctx).await;
    assert_eq!(
        &platform_top[..len],
        [
            DonorRecord {
                donor: ctx.donors[2].pubkey(),
                donations_sum: 9700,
            },
            DonorRecord {
                donor: ctx.donors[0].pubkey(),
                donations_sum: 97 + 970,
            },
            DonorRecord {
                donor: ctx.donors[3].pubkey(),
                donations_sum: 1,
            }
        ]
    );
}

async fn withdraws_donations(ptc: &mut ProgramTestContext, ctx: &Ctx) {
    let (active_campaigns, len) = fetch_active_campaigns(ptc, ctx).await;
    assert_eq!(
        active_campaigns[..len],
        [CampaignRecord {
            id: 0,
            donations_sum: 97 + 970 + 9700 + 1,
            withdrawn_sum: 0,
        }]
    );

    withdraw_donations(ptc, ctx, 0).await.unwrap();

    assert_eq!(get_sol_vault_balance(ptc, ctx).await, 0);
    let (active_campaigns, len) = fetch_active_campaigns(ptc, ctx).await;
    assert_eq!(
        active_campaigns[..len],
        [CampaignRecord {
            id: 0,
            donations_sum: 97 + 970 + 9700 + 1,
            withdrawn_sum: 97 + 970 + 9700 + 1,
        }]
    );
}

async fn withdraws_fees(ptc: &mut ProgramTestContext, ctx: &Ctx) {
    withdraw_fees(ptc, ctx).await.unwrap();

    assert_eq!(get_fee_vault_balance(ptc, ctx).await, 0);
}

async fn incentivizes(ptc: &mut ProgramTestContext, ctx: &Ctx) {
    incentivize(ptc, ctx).await.unwrap();

    let donor_chrt: TokenAccount = fetch(
        ptc,
        get_associated_token_address(&ctx.donors[0].pubkey(), &ctx.chrt_mint),
    )
    .await;
    assert_eq!(donor_chrt.amount, 10000);

    let donor_chrt: TokenAccount = fetch(
        ptc,
        get_associated_token_address(&ctx.donors[1].pubkey(), &ctx.chrt_mint),
    )
    .await;
    assert_eq!(donor_chrt.amount, 0);

    let donor_chrt: TokenAccount = fetch(
        ptc,
        get_associated_token_address(&ctx.donors[2].pubkey(), &ctx.chrt_mint),
    )
    .await;
    assert_eq!(donor_chrt.amount, 10000);

    let donor_chrt: TokenAccount = fetch(
        ptc,
        get_associated_token_address(&ctx.donors[3].pubkey(), &ctx.chrt_mint),
    )
    .await;
    assert_eq!(donor_chrt.amount, 10000);

    let donor_chrt: TokenAccount = fetch(
        ptc,
        get_associated_token_address(&ctx.donors[4].pubkey(), &ctx.chrt_mint),
    )
    .await;
    assert_eq!(donor_chrt.amount, 0);
}

async fn exempts_from_fees(ptc: &mut ProgramTestContext, ctx: &Ctx) {
    transfer_tokens(
        ptc,
        &get_associated_token_address(&ctx.donors[0].pubkey(), &ctx.chrt_mint),
        &find_fee_exemption_vault(0),
        1000,
        &ctx.donors[0],
    )
    .await
    .unwrap();

    donate(ptc, ctx, &ctx.donors[3], 0, 100_000).await.unwrap();
    let (active_campaigns, len) = fetch_active_campaigns(ptc, ctx).await;
    assert_eq!(
        active_campaigns[..len],
        [CampaignRecord {
            id: 0,
            donations_sum: 97 + 970 + 9700 + 1 + 100_000,
            withdrawn_sum: 97 + 970 + 9700 + 1,
        }]
    );
}

async fn starts_more_campaigns_and_donates(ptc: &mut ProgramTestContext, ctx: &Ctx) {
    start_campaign(ptc, ctx).await.unwrap();
    start_campaign(ptc, ctx).await.unwrap();

    let (active_campaigns, len) = fetch_active_campaigns(ptc, ctx).await;
    assert_eq!(
        active_campaigns[..len],
        [
            CampaignRecord {
                id: 0,
                donations_sum: 97 + 970 + 9700 + 1 + 100_000,
                withdrawn_sum: 97 + 970 + 9700 + 1,
            },
            CampaignRecord {
                id: 1,
                donations_sum: 0,
                withdrawn_sum: 0,
            },
            CampaignRecord {
                id: 2,
                donations_sum: 0,
                withdrawn_sum: 0,
            }
        ]
    );

    donate(ptc, ctx, &ctx.donors[3], 1, 1).await.unwrap();
    donate(ptc, ctx, &ctx.donors[3], 2, 9).await.unwrap();

    let (active_campaigns, len) = fetch_active_campaigns(ptc, ctx).await;
    assert_eq!(
        active_campaigns[..len],
        [
            CampaignRecord {
                id: 0,
                donations_sum: 97 + 970 + 9700 + 1 + 100_000,
                withdrawn_sum: 97 + 970 + 9700 + 1,
            },
            CampaignRecord {
                id: 1,
                donations_sum: 1,
                withdrawn_sum: 0,
            },
            CampaignRecord {
                id: 2,
                donations_sum: 9,
                withdrawn_sum: 0,
            }
        ]
    );
}

async fn liquidates_campaign(ptc: &mut ProgramTestContext, ctx: &Ctx) {
    // this causes the next liquidate_campaign to panic
    // const CODE: u32 = 6000 + CrowdfundingError::NotEnoughCHRTInVault as u32;
    // assert_matches!(
    //     liquidate_campaign(ptc, ctx, 0).await,
    //     Err(BanksClientError::TransactionError(
    //         TransactionError::InstructionError(0, InstructionError::Custom(CODE))
    //     ))
    // );

    transfer_tokens(
        ptc,
        &get_associated_token_address(&ctx.donors[0].pubkey(), &ctx.chrt_mint),
        &find_liquidation_vault(0),
        2000,
        &ctx.donors[0],
    )
    .await
    .unwrap();

    liquidate_campaign(ptc, ctx, 0).await.unwrap();

    const CODE: u32 = ErrorCode::AccountOwnedByWrongProgram as u32;
    assert_matches!(
        donate(ptc, ctx, &ctx.donors[5], 0, 1).await,
        Err(BanksClientError::TransactionError(
            TransactionError::InstructionError(0, InstructionError::Custom(CODE))
        ))
    );

    let (active_campaigns, len) = fetch_active_campaigns(ptc, ctx).await;
    assert_eq!(
        active_campaigns[..len],
        [
            CampaignRecord {
                id: 1,
                donations_sum: 1 + 10000,
                withdrawn_sum: 0,
            },
            CampaignRecord {
                id: 2,
                donations_sum: 9 + 90000,
                withdrawn_sum: 0,
            }
        ]
    );
}

async fn withdraws_donations_that_came_from_liquidation(ptc: &mut ProgramTestContext, ctx: &Ctx) {
    assert_eq!(get_sol_vault_balance(ptc, ctx).await, 1 + 10000 + 9 + 90000);

    withdraw_donations(ptc, ctx, 1).await.unwrap();

    assert_eq!(get_sol_vault_balance(ptc, ctx).await, 9 + 90000);
    let (active_campaigns, len) = fetch_active_campaigns(ptc, ctx).await;
    assert_eq!(
        active_campaigns[..len],
        [
            CampaignRecord {
                id: 1,
                donations_sum: 1 + 10000,
                withdrawn_sum: 1 + 10000,
            },
            CampaignRecord {
                id: 2,
                donations_sum: 9 + 90000,
                withdrawn_sum: 0,
            }
        ]
    );

    stop_campaign(ptc, ctx, 2).await.unwrap();

    let (active_campaigns, len) = fetch_active_campaigns(ptc, ctx).await;
    assert_eq!(
        active_campaigns[..len],
        [CampaignRecord {
            id: 1,
            donations_sum: 1 + 10000,
            withdrawn_sum: 1 + 10000,
        }]
    );

    stop_campaign(ptc, ctx, 1).await.unwrap();

    let (active_campaigns, len) = fetch_active_campaigns(ptc, ctx).await;
    assert_eq!(active_campaigns[..len], []);

    withdraw_fees(ptc, ctx).await.unwrap();
}

async fn sorts_top_with_more_than_10_donors(ptc: &mut ProgramTestContext, ctx: &Ctx) {
    start_campaign(ptc, ctx).await.unwrap();

    incentivize(ptc, ctx).await.unwrap();

    donate(ptc, ctx, &ctx.donors[14], 3, 14).await.unwrap();
    donate(ptc, ctx, &ctx.donors[2], 3, 2).await.unwrap();
    donate(ptc, ctx, &ctx.donors[5], 3, 5).await.unwrap();
    donate(ptc, ctx, &ctx.donors[1], 3, 1).await.unwrap();
    donate(ptc, ctx, &ctx.donors[11], 3, 11).await.unwrap();
    donate(ptc, ctx, &ctx.donors[12], 3, 12).await.unwrap();
    donate(ptc, ctx, &ctx.donors[10], 3, 10).await.unwrap();
    donate(ptc, ctx, &ctx.donors[9], 3, 9).await.unwrap();
    donate(ptc, ctx, &ctx.donors[13], 3, 13).await.unwrap();
    donate(ptc, ctx, &ctx.donors[7], 3, 7).await.unwrap();
    donate(ptc, ctx, &ctx.donors[8], 3, 8).await.unwrap();
    donate(ptc, ctx, &ctx.donors[4], 3, 4).await.unwrap();
    donate(ptc, ctx, &ctx.donors[3], 3, 3).await.unwrap();

    let (campaign_top, len) = fetch_campaign_top(ptc, 3).await;
    assert_eq!(
        campaign_top[..len],
        [
            DonorRecord {
                donor: ctx.donors[14].pubkey(),
                donations_sum: 14,
            },
            DonorRecord {
                donor: ctx.donors[13].pubkey(),
                donations_sum: 13,
            },
            DonorRecord {
                donor: ctx.donors[12].pubkey(),
                donations_sum: 12,
            },
            DonorRecord {
                donor: ctx.donors[11].pubkey(),
                donations_sum: 11,
            },
            DonorRecord {
                donor: ctx.donors[10].pubkey(),
                donations_sum: 10,
            },
            DonorRecord {
                donor: ctx.donors[9].pubkey(),
                donations_sum: 9,
            },
            DonorRecord {
                donor: ctx.donors[8].pubkey(),
                donations_sum: 8,
            },
            DonorRecord {
                donor: ctx.donors[7].pubkey(),
                donations_sum: 7,
            },
            DonorRecord {
                donor: ctx.donors[5].pubkey(),
                donations_sum: 5,
            },
            DonorRecord {
                donor: ctx.donors[4].pubkey(),
                donations_sum: 4,
            }
        ]
    );

    let mut donors = heapless::Vec::<_, DONORS_LEN>::new();
    for donor in &campaign_top[..len] {
        donors.push(donor.donor).unwrap();
    }
    assert_eq!(get_seasonal_top(ptc, ctx).await, donors);
}
