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
  it("initialize", async () => {
    const maxLiquidations = 100;
    const incentiveCooldown = 10;
    const incentiveAmount = 1000;
    const platformFeeNum = 3;
    const platformFeeDenom = 100;
    const feeExemptionLimit = 1000;
    const liquidationLimit = 10000;
    await initializeCrowdfunding(
      ctx,
      maxLiquidations,
      incentiveCooldown,
      incentiveAmount,
      platformFeeNum,
      platformFeeDenom,
      feeExemptionLimit,
      liquidationLimit
    );

    const platform = await ctx.program.account.platform.fetch(ctx.platform);
    expect(platform.bump).to.gt(200);
    expect(platform.bumpLiquidatedSolVault).to.gt(200);
    expect(platform.bumpChrtMint).to.gt(200);
    expect(platform.authority).to.eql(ctx.platformAuthority.publicKey);
    expect(platform.maxLiquidations).to.eql(maxLiquidations);
    expect(platform.incentiveCooldown).to.eql(incentiveCooldown);
    expect(platform.incentiveAmount.toNumber()).to.eql(incentiveAmount);
    expect(platform.platformFeeNum.toNumber()).to.eql(platformFeeNum);
    expect(platform.platformFeeDenom.toNumber()).to.eql(platformFeeDenom);
    expect(platform.feeExemptionLimit.toNumber()).to.eql(feeExemptionLimit);
    expect(platform.liquidationLimit.toNumber()).to.eql(liquidationLimit);
  });

  it("registerDonor", async () => {
    await registerDonor(ctx, ctx.donor1);
    await registerDonor(ctx, ctx.donor2);
  });

  it("startCampaign", async () => {
    await startCampaign(ctx);

    const platform = await ctx.program.account.platform.fetch(ctx.platform);
    expect(platform.campaignsCount).to.eql(1);

    const campaign = await ctx.program.account.campaign.fetch(
      await ctx.campaign(0)
    );
    expect(campaign.bump).to.gt(200);
    expect(campaign.bumpSolVault).to.gt(200);
    expect(campaign.bumpFeeExemptionVault).to.gt(200);
    expect(campaign.bumpLiquidationVault).to.gt(200);
    expect(campaign.authority).to.eql(ctx.campaignAuthority.publicKey);
    expect(campaign.id).to.eql(0);
    expect(campaign.lastClaimTs).to.be.within(
      +new Date() / 1000 - 7,
      +new Date() / 1000
    );

    await startCampaign(ctx);
  });

  it("donate", async () => {
    await donate(ctx, ctx.donor1, 0, 600);
    await donate(ctx, ctx.donor1, 0, 600);
  });

  it("donateWithReferer", async () => {
    await donateWithReferer(ctx, ctx.donor2, 0, 500, ctx.donor1.publicKey);
  });

  it("incentivize", async () => {
    await incentivize(ctx);
  });

  it("liquidateCampaign", async () => {
    await expect(liquidateCampaign(ctx, 0)).to.be.rejectedWith(
      "NotEnoughCHRTInVault"
    );
  });

  it("withdrawDonations", async () => {
    await withdrawDonations(ctx, 0);
  });

  it("stopCampaign", async () => {
    await stopCampaign(ctx, 0);
  });

  it("withdrawFees", async () => {
    await withdrawFees(ctx);
  });
});
