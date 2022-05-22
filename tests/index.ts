import { expect } from "chai";
import * as chai from "chai";
import chaiAsPromised from "chai-as-promised";
import { Context } from "./ctx";
import {
  startCampaign,
  initializeCrowdfunding,
  liquidateCampaign,
  stopCampaign,
  registerDonor,
  donate,
  donateWithReferer,
  incentivize,
  withdrawFees,
  withdrawDonations,
} from "./crowdfundingAPI";
import { transfer } from "./token";

chai.use(chaiAsPromised);

const ctx = new Context();

before(async () => {
  await ctx.setup();
});

describe("crowdfunding", () => {
  it("Initialize", async () => {
    const activeCampaignsCapacity = 100;
    const incentiveCooldown = 2;
    const incentiveAmount = 1000;
    const platformFeeNum = 3;
    const platformFeeDenom = 100;
    const feeExemptionLimit = 1000;
    const liquidationLimit = 2000;
    await initializeCrowdfunding(
      ctx,
      activeCampaignsCapacity,
      incentiveCooldown,
      incentiveAmount,
      platformFeeNum,
      platformFeeDenom,
      feeExemptionLimit,
      liquidationLimit
    );

    const platform = await ctx.program.account.platform.fetch(ctx.platform);
    expect(platform.bump).to.gt(200);
    expect(platform.bumpFeeVault).to.gt(200);
    expect(platform.bumpSolVault).to.gt(200);
    expect(platform.bumpChrtMint).to.gt(200);
    expect(platform.authority).to.eql(ctx.platformAuthority.publicKey);
    expect(platform.activeCampaignsCapacity).to.eql(activeCampaignsCapacity);
    expect(platform.incentiveCooldown).to.eql(incentiveCooldown);
    expect(platform.incentiveAmount.toNumber()).to.eql(incentiveAmount);
    expect(platform.platformFeeNum.toNumber()).to.eql(platformFeeNum);
    expect(platform.platformFeeDenom.toNumber()).to.eql(platformFeeDenom);
    expect(platform.feeExemptionLimit.toNumber()).to.eql(feeExemptionLimit);
    expect(platform.liquidationLimit.toNumber()).to.eql(liquidationLimit);
  });

  it("RegisterDonor", async () => {
    const promises = [];
    for (let i = 0; i < ctx.donors.length; i++) {
      promises.push(registerDonor(ctx, ctx.donors[i]));
    }
    await Promise.all(promises);

    const donor = await ctx.program.account.donor.fetch(
      await ctx.donor(ctx.donors[0].publicKey)
    );
    expect(donor.bump).to.gt(200);
    expect(donor.authority).to.eql(ctx.donors[0].publicKey);
  });

  it("StartCampaign", async () => {
    await startCampaign(ctx);

    const campaign = await ctx.program.account.campaign.fetch(
      await ctx.campaign(0)
    );
    expect(campaign.bump).to.gt(200);
    expect(campaign.bumpFeeExemptionVault).to.gt(200);
    expect(campaign.bumpLiquidationVault).to.gt(200);
    expect(campaign.authority).to.eql(ctx.campaignAuthority.publicKey);
    expect(campaign.id).to.eql(0);

    expect(await ctx.activeCampaigns()).to.eql([
      { id: 0, donationsSum: 0, withdrawnSum: 0 },
    ]);

    await startCampaign(ctx);

    expect(await ctx.activeCampaigns()).to.eql([
      { id: 0, donationsSum: 0, withdrawnSum: 0 },
      { id: 1, donationsSum: 0, withdrawnSum: 0 },
    ]);
  });

  it("Donate", async () => {
    await donate(ctx, ctx.donors[0], 0, 100);

    const platform = await ctx.program.account.platform.fetch(ctx.platform);
    expect(platform.sumOfAllDonations.toNumber()).to.eql(97);
    expect(platform.sumOfActiveCampaignDonations.toNumber()).to.eql(97);

    const donor = await ctx.program.account.donor.fetch(
      await ctx.donor(ctx.donors[0].publicKey)
    );
    expect(donor.donationsSum.toNumber()).to.eql(97);

    const donations = await ctx.program.account.donations.fetch(
      await ctx.donations(ctx.donors[0].publicKey, await ctx.campaign(0))
    );
    expect(donations.donationsSum.toNumber()).to.eql(97);

    expect(await ctx.solVaultBalance()).to.eql(97);
    expect(await ctx.feeVaultBalance()).to.eql(3);
    expect(await ctx.activeCampaigns()).to.eql([
      { id: 0, donationsSum: 97, withdrawnSum: 0 },
      { id: 1, donationsSum: 0, withdrawnSum: 0 },
    ]);

    expect(await ctx.platformTop()).to.eql([
      { donor: ctx.donors[0].publicKey, donationsSum: 97 },
    ]);
    expect(await ctx.campaignTop(0)).to.eql([
      { donor: ctx.donors[0].publicKey, donationsSum: 97 },
    ]);
  });

  it("DonateWithReferer", async () => {
    await donateWithReferer(
      ctx,
      ctx.donors[1],
      0,
      10_000,
      ctx.donors[0].publicKey
    );

    expect(
      await (await ctx.chrtATA(ctx.donors[0].publicKey)).amount(ctx)
    ).to.eql(1);

    const platform = await ctx.program.account.platform.fetch(ctx.platform);
    expect(platform.sumOfAllDonations.toNumber()).to.eql(97 + 9_700);
    expect(platform.sumOfActiveCampaignDonations.toNumber()).to.eql(97 + 9_700);

    const donor = await ctx.program.account.donor.fetch(
      await ctx.donor(ctx.donors[1].publicKey)
    );
    expect(donor.donationsSum.toNumber()).to.eql(9_700);

    const donations = await ctx.program.account.donations.fetch(
      await ctx.donations(ctx.donors[1].publicKey, await ctx.campaign(0))
    );
    expect(donations.donationsSum.toNumber()).to.eql(9_700);

    expect(await ctx.solVaultBalance()).to.eql(97 + 9_700);
    expect(await ctx.feeVaultBalance()).to.eql(3 + 300);
    expect(await ctx.activeCampaigns()).to.eql([
      { id: 0, donationsSum: 97 + 9_700, withdrawnSum: 0 },
      { id: 1, donationsSum: 0, withdrawnSum: 0 },
    ]);

    expect(await ctx.platformTop()).to.eql([
      { donor: ctx.donors[1].publicKey, donationsSum: 9_700 },
      { donor: ctx.donors[0].publicKey, donationsSum: 97 },
    ]);
    expect(await ctx.campaignTop(0)).to.eql([
      { donor: ctx.donors[1].publicKey, donationsSum: 9_700 },
      { donor: ctx.donors[0].publicKey, donationsSum: 97 },
    ]);
  });

  it("Incentivize", async () => {
    await incentivize(ctx);

    await expect(incentivize(ctx)).to.be.rejectedWith("IncentiveCooldown");

    const platform = await ctx.program.account.platform.fetch(ctx.platform);
    expect(platform.lastIncentiveTs).to.be.within(
      +new Date() / 1000 - 7,
      +new Date() / 1000
    );

    expect(
      await (await ctx.chrtATA(ctx.donors[0].publicKey)).amount(ctx)
    ).to.eql(1001);
    expect(
      await (await ctx.chrtATA(ctx.donors[1].publicKey)).amount(ctx)
    ).to.eql(1000);

    const donor0 = await ctx.program.account.donor.fetch(
      await ctx.donor(ctx.donors[0].publicKey)
    );
    expect(donor0.incentivizedDonationsSum).to.eql(donor0.donationsSum);

    const donor1 = await ctx.program.account.donor.fetch(
      await ctx.donor(ctx.donors[1].publicKey)
    );
    expect(donor1.incentivizedDonationsSum).to.eql(donor1.donationsSum);
  });

  it("WithdrawDonations", async () => {
    await withdrawDonations(ctx, 0);

    expect(await ctx.solVaultBalance()).to.eql(0);
    expect(await ctx.activeCampaigns()).to.eql([
      { id: 0, donationsSum: 97 + 9_700, withdrawnSum: 97 + 9_700 },
      { id: 1, donationsSum: 0, withdrawnSum: 0 },
    ]);
  });

  it("LiquidateCampaign", async () => {
    await expect(liquidateCampaign(ctx, 0)).to.be.rejectedWith(
      "NotEnoughCHRTInVault"
    );
  });

  it("StopCampaign", async () => {
    await stopCampaign(ctx, 0);

    await expect(donate(ctx, ctx.donors[5], 0, 1)).to.be.rejectedWith(
      "AccountNotInitialized"
    );

    expect(await ctx.activeCampaigns()).to.eql([
      { id: 1, donationsSum: 0, withdrawnSum: 0 },
    ]);

    const platform = await ctx.program.account.platform.fetch(ctx.platform);
    expect(platform.sumOfActiveCampaignDonations.toNumber()).to.eql(0);

    await stopCampaign(ctx, 1);

    expect(await ctx.activeCampaigns()).to.eql([]);
  });

  it("WithdrawFees", async () => {
    await withdrawFees(ctx);

    expect(await ctx.feeVaultBalance()).to.eql(0);
  });
});

describe("scenario 1", async () => {
  it("starts campaign", async () => {
    await startCampaign(ctx);

    expect(await ctx.activeCampaigns()).to.eql([
      { id: 2, donationsSum: 0, withdrawnSum: 0 },
    ]);
  });

  it("donates", async () => {
    await donate(ctx, ctx.donors[0], 2, 1_000);
    expect(await ctx.platformTop()).to.eql([
      { donor: ctx.donors[1].publicKey, donationsSum: 9_700 },
      { donor: ctx.donors[0].publicKey, donationsSum: 97 + 970 },
    ]);
    expect(await ctx.campaignTop(2)).to.eql([
      { donor: ctx.donors[0].publicKey, donationsSum: 970 },
    ]);

    await donate(ctx, ctx.donors[0], 2, 10_000);
    expect(await ctx.platformTop()).to.eql([
      { donor: ctx.donors[0].publicKey, donationsSum: 97 + 970 + 9_700 },
      { donor: ctx.donors[1].publicKey, donationsSum: 9_700 },
    ]);
    expect(await ctx.campaignTop(2)).to.eql([
      { donor: ctx.donors[0].publicKey, donationsSum: 970 + 9_700 },
    ]);

    await donate(ctx, ctx.donors[2], 2, 100_000);
    expect(await ctx.platformTop()).to.eql([
      { donor: ctx.donors[2].publicKey, donationsSum: 97_000 },
      { donor: ctx.donors[0].publicKey, donationsSum: 97 + 970 + 9_700 },
      { donor: ctx.donors[1].publicKey, donationsSum: 9_700 },
    ]);
    expect(await ctx.campaignTop(2)).to.eql([
      { donor: ctx.donors[2].publicKey, donationsSum: 97_000 },
      { donor: ctx.donors[0].publicKey, donationsSum: 970 + 9_700 },
    ]);

    await donate(ctx, ctx.donors[3], 2, 1);
    expect(await ctx.platformTop()).to.eql([
      { donor: ctx.donors[2].publicKey, donationsSum: 97_000 },
      { donor: ctx.donors[0].publicKey, donationsSum: 97 + 970 + 9_700 },
      { donor: ctx.donors[1].publicKey, donationsSum: 9_700 },
      { donor: ctx.donors[3].publicKey, donationsSum: 1 },
    ]);
    expect(await ctx.campaignTop(2)).to.eql([
      { donor: ctx.donors[2].publicKey, donationsSum: 97_000 },
      { donor: ctx.donors[0].publicKey, donationsSum: 970 + 9_700 },
      { donor: ctx.donors[3].publicKey, donationsSum: 1 },
    ]);
  });

  const donatedTo2 = 970 + 9_700 + 97_000 + 1;

  it("withdraws donations", async () => {
    expect(await ctx.activeCampaigns()).to.eql([
      { id: 2, donationsSum: donatedTo2, withdrawnSum: 0 },
    ]);

    await withdrawDonations(ctx, 2);

    expect(await ctx.solVaultBalance()).to.eql(0);
    expect(await ctx.activeCampaigns()).to.eql([
      { id: 2, donationsSum: donatedTo2, withdrawnSum: donatedTo2 },
    ]);
  });

  it("withdraws fees", async () => {
    await withdrawFees(ctx);

    expect(await ctx.feeVaultBalance()).to.eql(0);
  });

  it("exempts from fees", async () => {
    await transfer(
      ctx,
      await ctx.chrtATA(ctx.donors[1].publicKey),
      await ctx.feeExemptionVault(2),
      ctx.donors[1],
      1000
    );

    await donate(ctx, ctx.donors[3], 2, 100_000);

    expect(await ctx.activeCampaigns()).to.eql([
      { id: 2, donationsSum: donatedTo2 + 100_000, withdrawnSum: donatedTo2 },
    ]);
  });

  it("starts more campaigns and donates", async () => {
    await startCampaign(ctx);
    await startCampaign(ctx);

    expect(await ctx.activeCampaigns()).to.eql([
      { id: 2, donationsSum: donatedTo2 + 100_000, withdrawnSum: donatedTo2 },
      { id: 3, donationsSum: 0, withdrawnSum: 0 },
      { id: 4, donationsSum: 0, withdrawnSum: 0 },
    ]);

    await donate(ctx, ctx.donors[3], 3, 1);
    await donate(ctx, ctx.donors[3], 4, 9);

    expect(await ctx.activeCampaigns()).to.eql([
      { id: 2, donationsSum: donatedTo2 + 100_000, withdrawnSum: donatedTo2 },
      { id: 3, donationsSum: 1, withdrawnSum: 0 },
      { id: 4, donationsSum: 9, withdrawnSum: 0 },
    ]);
  });

  it("incentivizes", async () => {
    await incentivize(ctx);

    expect(
      await (await ctx.chrtATA(ctx.donors[0].publicKey)).amount(ctx)
    ).to.eql(2001);
    expect(
      await (await ctx.chrtATA(ctx.donors[1].publicKey)).amount(ctx)
    ).to.eql(0);
    expect(
      await (await ctx.chrtATA(ctx.donors[2].publicKey)).amount(ctx)
    ).to.eql(1000);
    expect(
      await (await ctx.chrtATA(ctx.donors[3].publicKey)).amount(ctx)
    ).to.eql(1000);
  });

  it("liquidates campaign", async () => {
    await expect(liquidateCampaign(ctx, 2)).to.be.rejectedWith(
      "NotEnoughCHRTInVault"
    );

    await transfer(
      ctx,
      await ctx.chrtATA(ctx.donors[0].publicKey),
      await ctx.liquidationVault(2),
      ctx.donors[0],
      2000
    );

    await liquidateCampaign(ctx, 2);

    await expect(donate(ctx, ctx.donors[5], 2, 1)).to.be.rejectedWith(
      "AccountNotInitialized"
    );

    expect(await ctx.activeCampaigns()).to.eql([
      { id: 3, donationsSum: 1 + 10000, withdrawnSum: 0 },
      { id: 4, donationsSum: 9 + 90000, withdrawnSum: 0 },
    ]);
  });
});
