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

chai.use(chaiAsPromised);

const ctx = new Context();

before(async () => {
  await ctx.setup();
});

describe("crowdfunding", () => {
  it("Initialize", async () => {
    const campaignsCapacity = 100;
    const incentiveCooldown = 10;
    const incentiveAmount = 1000;
    const platformFeeNum = 3;
    const platformFeeDenom = 100;
    const feeExemptionLimit = 1000;
    const liquidationLimit = 10000;
    await initializeCrowdfunding(
      ctx,
      campaignsCapacity,
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
    expect(platform.campaignsCapacity).to.eql(campaignsCapacity);
    expect(platform.incentiveCooldown).to.eql(incentiveCooldown);
    expect(platform.incentiveAmount.toNumber()).to.eql(incentiveAmount);
    expect(platform.platformFeeNum.toNumber()).to.eql(platformFeeNum);
    expect(platform.platformFeeDenom.toNumber()).to.eql(platformFeeDenom);
    expect(platform.feeExemptionLimit.toNumber()).to.eql(feeExemptionLimit);
    expect(platform.liquidationLimit.toNumber()).to.eql(liquidationLimit);
  });

  it("RegisterDonor", async () => {
    await registerDonor(ctx, ctx.donor1);
    await registerDonor(ctx, ctx.donor2);

    const donor = await ctx.program.account.donor.fetch(
      await ctx.donor(ctx.donor1.publicKey)
    );
    expect(donor.bump).to.gt(200);
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

    expect(await ctx.campaigns()).to.eql([
      { donationsSum: 0, withdrawnSum: 0, isClosed: false },
    ]);

    await startCampaign(ctx);

    expect(await ctx.campaigns()).to.eql([
      { donationsSum: 0, withdrawnSum: 0, isClosed: false },
      { donationsSum: 0, withdrawnSum: 0, isClosed: false },
    ]);
  });

  it("Donate", async () => {
    await donate(ctx, ctx.donor1, 0, 100);

    const platform = await ctx.program.account.platform.fetch(ctx.platform);
    expect(platform.sumOfAllDonations.toNumber()).to.eql(97);
    expect(platform.sumOfActiveCampaignDonations.toNumber()).to.eql(97);

    const donor = await ctx.program.account.donor.fetch(
      await ctx.donor(ctx.donor1.publicKey)
    );
    expect(donor.donationsSum.toNumber()).to.eql(97);
    expect(donor.seasonalDonationsSum.toNumber()).to.eql(97);
    expect(donor.lastDonationTs).to.be.within(
      +new Date() / 1000 - 5,
      +new Date() / 1000
    );

    const donations = await ctx.program.account.donations.fetch(
      await ctx.donations(ctx.donor1.publicKey, await ctx.campaign(0))
    );
    expect(donations.donationsSum.toNumber()).to.eql(97);

    expect(await ctx.solVaultBalance()).to.eql(97);
    expect(await ctx.feeVaultBalance()).to.eql(3);
    expect(await ctx.campaigns()).to.eql([
      { donationsSum: 97, withdrawnSum: 0, isClosed: false },
      { donationsSum: 0, withdrawnSum: 0, isClosed: false },
    ]);

    expect(await ctx.platformTop()).to.eql([
      { donor: ctx.donor1.publicKey, donationsSum: 97 },
    ]);
    expect(await ctx.seasonalTop()).to.eql([
      { donor: ctx.donor1.publicKey, donationsSum: 97 },
    ]);
    expect(await ctx.campaignTop(0)).to.eql([
      { donor: ctx.donor1.publicKey, donationsSum: 97 },
    ]);
  });

  it("DonateWithReferer", async () => {
    await donateWithReferer(ctx, ctx.donor2, 0, 10_000, ctx.donor1.publicKey);

    expect(await (await ctx.chrtATA(ctx.donor1.publicKey)).amount(ctx)).to.eql(
      1
    );

    const platform = await ctx.program.account.platform.fetch(ctx.platform);
    expect(platform.sumOfAllDonations.toNumber()).to.eql(97 + 9_700);
    expect(platform.sumOfActiveCampaignDonations.toNumber()).to.eql(97 + 9_700);

    const donor = await ctx.program.account.donor.fetch(
      await ctx.donor(ctx.donor2.publicKey)
    );
    expect(donor.donationsSum.toNumber()).to.eql(9_700);
    expect(donor.seasonalDonationsSum.toNumber()).to.eql(9_700);
    expect(donor.lastDonationTs).to.be.within(
      +new Date() / 1000 - 7,
      +new Date() / 1000
    );

    const donations = await ctx.program.account.donations.fetch(
      await ctx.donations(ctx.donor2.publicKey, await ctx.campaign(0))
    );
    expect(donations.donationsSum.toNumber()).to.eql(9_700);

    expect(await ctx.solVaultBalance()).to.eql(97 + 9_700);
    expect(await ctx.feeVaultBalance()).to.eql(3 + 300);
    expect(await ctx.campaigns()).to.eql([
      { donationsSum: 97 + 9_700, withdrawnSum: 0, isClosed: false },
      { donationsSum: 0, withdrawnSum: 0, isClosed: false },
    ]);

    expect(await ctx.platformTop()).to.eql([
      { donor: ctx.donor2.publicKey, donationsSum: 9_700 },
      { donor: ctx.donor1.publicKey, donationsSum: 97 },
    ]);
    expect(await ctx.seasonalTop()).to.eql([
      { donor: ctx.donor2.publicKey, donationsSum: 9_700 },
      { donor: ctx.donor1.publicKey, donationsSum: 97 },
    ]);
    expect(await ctx.campaignTop(0)).to.eql([
      { donor: ctx.donor2.publicKey, donationsSum: 9_700 },
      { donor: ctx.donor1.publicKey, donationsSum: 97 },
    ]);
  });

  it("Incentivize", async () => {
    await incentivize(ctx);

    await expect(incentivize(ctx)).to.be.rejectedWith("IncentiveCooldown");

    const platform = await ctx.program.account.platform.fetch(ctx.platform);
    expect(platform.lastIncentiveTs).to.be.within(
      +new Date() / 1000 - 5,
      +new Date() / 1000
    );

    expect(await (await ctx.chrtATA(ctx.donor1.publicKey)).amount(ctx)).to.eql(
      1001
    );
    expect(await (await ctx.chrtATA(ctx.donor2.publicKey)).amount(ctx)).to.eql(
      1000
    );

    expect(await ctx.seasonalTop()).to.eql([]);
  });

  it("WithdrawDonations", async () => {
    await withdrawDonations(ctx, 0);

    expect(await ctx.solVaultBalance()).to.eql(0);
    expect(await ctx.campaigns()).to.eql([
      { donationsSum: 97 + 9_700, withdrawnSum: 97 + 9_700, isClosed: false },
      { donationsSum: 0, withdrawnSum: 0, isClosed: false },
    ]);
  });

  it("LiquidateCampaign", async () => {
    await expect(liquidateCampaign(ctx, 0)).to.be.rejectedWith(
      "NotEnoughCHRTInVault"
    );
  });

  it("StopCampaign", async () => {
    await stopCampaign(ctx, 0);

    expect(await ctx.campaigns()).to.eql([
      { donationsSum: 97 + 9_700, withdrawnSum: 97 + 9_700, isClosed: true },
      { donationsSum: 0, withdrawnSum: 0, isClosed: false },
    ]);
  });

  it("WithdrawFees", async () => {
    await withdrawFees(ctx);

    expect(await ctx.feeVaultBalance()).to.eql(0);
  });
});
