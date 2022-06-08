import { BN } from "@project-serum/anchor";
import {
  SystemProgram,
  Keypair,
  SYSVAR_RENT_PUBKEY,
  PublicKey,
} from "@solana/web3.js";
import { TOKEN_PROGRAM_ID } from "@solana/spl-token";
import { Context } from "./ctx";
import { sha256 } from "js-sha256";
import bs58 from "bs58";
import { SEASONAL_TOP_CAPACITY } from "./config";

export async function initializeCrowdfunding(
  ctx: Context,
  activeCampaignsCapacity: number,
  incentiveCooldown: number,
  incentiveAmount: number | BN,
  platformFeeNum: number | BN,
  platformFeeDenom: number | BN,
  feeExemptionLimit: number | BN,
  liquidationLimit: number | BN
): Promise<void> {
  await ctx.program.methods
    .initialize(
      activeCampaignsCapacity,
      incentiveCooldown,
      new BN(incentiveAmount),
      new BN(platformFeeNum),
      new BN(platformFeeDenom),
      new BN(feeExemptionLimit),
      new BN(liquidationLimit)
    )
    .accounts({
      platform: ctx.platform,
      platformAuthority: ctx.platformAuthority.publicKey,
      feeVault: ctx.feeVault,
      solVault: ctx.solVault,
      chrtMint: ctx.chrtMint,
      rent: SYSVAR_RENT_PUBKEY,
      tokenProgram: TOKEN_PROGRAM_ID,
      systemProgram: SystemProgram.programId,
    })
    .signers([ctx.platformAuthority])
    .rpc();
}

export async function registerDonor(
  ctx: Context,
  donorAuthority: Keypair
): Promise<void> {
  await ctx.program.methods
    .registerDonor()
    .accounts({
      donor: await ctx.donor(donorAuthority.publicKey),
      donorAuthority: donorAuthority.publicKey,
      systemProgram: SystemProgram.programId,
    })
    .signers([donorAuthority])
    .rpc();
}

export async function startCampaign(ctx: Context): Promise<void> {
  const id = (await ctx.program.account.platform.fetch(ctx.platform))
    .campaignsCount;

  await ctx.program.methods
    .startCampaign()
    .accounts({
      platform: ctx.platform,
      chrtMint: ctx.chrtMint,
      campaign: await ctx.campaign(id),
      campaignAuthority: ctx.campaignAuthority.publicKey,
      totalDonationsToCampaign: await ctx.totalDonationsToCampaign(id),
      feeExemptionVault: await ctx.feeExemptionVault(id),
      liquidationVault: await ctx.liquidationVault(id),
      systemProgram: SystemProgram.programId,
    })
    .signers([ctx.campaignAuthority])
    .rpc();
}

export async function donate(
  ctx: Context,
  donorAuthority: Keypair,
  id: number,
  amount: number | BN
): Promise<void> {
  await ctx.program.methods
    .donate(new BN(amount))
    .accounts({
      platform: ctx.platform,
      feeVault: ctx.feeVault,
      solVault: ctx.solVault,
      campaign: await ctx.campaign(id),
      totalDonationsToCampaign: await ctx.totalDonationsToCampaign(id),
      feeExemptionVault: await ctx.feeExemptionVault(id),
      donor: await ctx.donor(donorAuthority.publicKey),
      donorAuthority: donorAuthority.publicKey,
      donorDonationsToCampaign: await ctx.donorDonationsToCampaign(
        donorAuthority.publicKey,
        id
      ),
      systemProgram: SystemProgram.programId,
    })
    .signers([donorAuthority])
    .rpc();
}

export async function donateWithReferer(
  ctx: Context,
  donorAuthority: Keypair,
  id: number,
  amount: number | BN,
  refererAuthority: PublicKey
): Promise<void> {
  await ctx.program.methods
    .donateWithReferer(new BN(amount))
    .accounts({
      donate: {
        platform: ctx.platform,
        feeVault: ctx.feeVault,
        solVault: ctx.solVault,
        campaign: await ctx.campaign(id),
        totalDonationsToCampaign: await ctx.totalDonationsToCampaign(id),
        feeExemptionVault: await ctx.feeExemptionVault(id),
        donor: await ctx.donor(donorAuthority.publicKey),
        donorAuthority: donorAuthority.publicKey,
        donorDonationsToCampaign: await ctx.donorDonationsToCampaign(
          donorAuthority.publicKey,
          id
        ),
        systemProgram: SystemProgram.programId,
      },
      chrtMint: ctx.chrtMint,
      referer: await ctx.donor(refererAuthority),
      refererAuthority,
      refererChrt: await ctx.chrtATA(refererAuthority),
      tokenProgram: TOKEN_PROGRAM_ID,
    })
    .signers([donorAuthority])
    .rpc();
}

export async function seasonalTop(ctx: Context): Promise<PublicKey[]> {
  const discriminator = Buffer.from(sha256.digest("account:Donor")).slice(0, 8);
  const filters = [
    { memcmp: { offset: 0, bytes: bs58.encode(discriminator) } },
  ];

  return (
    await ctx.connection.getProgramAccounts(ctx.program.programId, { filters })
  )
    .map((account) =>
      ctx.program.coder.accounts.decode("Donor", account.account.data)
    )
    .filter((d) => !d.donationsSum.eq(d.incentivizedDonationsSum))
    .sort((a, b) =>
      b.donationsSum
        .sub(b.incentivizedDonationsSum)
        .cmp(a.donationsSum.sub(a.incentivizedDonationsSum))
    )
    .slice(0, SEASONAL_TOP_CAPACITY)
    .map((d) => d.authority);
}

export async function incentivize(ctx: Context): Promise<void> {
  const remainingAccounts = [];

  const top = await seasonalTop(ctx);

  for (let i = 0; i < top.length; i++) {
    remainingAccounts.push(
      {
        pubkey: await ctx.donor(top[i]),
        isSigner: false,
        isWritable: true,
      },
      {
        pubkey: await ctx.chrtATA(top[i]),
        isSigner: false,
        isWritable: true,
      }
    );
  }

  await ctx.program.methods
    .incentivize()
    .accounts({
      platform: ctx.platform,
      platformAuthority: ctx.platformAuthority.publicKey,
      chrtMint: ctx.chrtMint,
      tokenProgram: TOKEN_PROGRAM_ID,
    })
    .remainingAccounts(remainingAccounts)
    .signers([ctx.platformAuthority])
    .rpc();
}

export async function withdrawDonations(
  ctx: Context,
  id: number
): Promise<void> {
  await ctx.program.methods
    .withdrawDonations()
    .accounts({
      platform: ctx.platform,
      solVault: ctx.solVault,
      campaign: await ctx.campaign(id),
      campaignAuthority: ctx.campaignAuthority.publicKey,
    })
    .signers([ctx.campaignAuthority])
    .rpc();
}

export async function stopCampaign(ctx: Context, id: number): Promise<void> {
  await ctx.program.methods
    .stopCampaign()
    .accounts({
      platform: ctx.platform,
      solVault: ctx.solVault,
      chrtMint: ctx.chrtMint,
      campaign: await ctx.campaign(id),
      campaignAuthority: ctx.campaignAuthority.publicKey,
      feeExemptionVault: await ctx.feeExemptionVault(id),
      liquidationVault: await ctx.liquidationVault(id),
      tokenProgram: TOKEN_PROGRAM_ID,
    })
    .signers([ctx.campaignAuthority])
    .rpc();
}

export async function liquidateCampaign(
  ctx: Context,
  id: number
): Promise<void> {
  await ctx.program.methods
    .liquidateCampaign()
    .accounts({
      platform: ctx.platform,
      feeVault: ctx.feeVault,
      solVault: ctx.solVault,
      chrtMint: ctx.chrtMint,
      campaign: await ctx.campaign(id),
      campaignAuthority: ctx.campaignAuthority.publicKey,
      feeExemptionVault: await ctx.feeExemptionVault(id),
      liquidationVault: await ctx.liquidationVault(id),
      tokenProgram: TOKEN_PROGRAM_ID,
    })
    .rpc();
}

export async function withdrawFees(ctx: Context): Promise<void> {
  await ctx.program.methods
    .withdrawFees()
    .accounts({
      platform: ctx.platform,
      platformAuthority: ctx.platformAuthority.publicKey,
      feeVault: ctx.feeVault,
    })
    .signers([ctx.platformAuthority])
    .rpc();
}
