import { BN } from "@project-serum/anchor";
import {
  SystemProgram,
  Keypair,
  SYSVAR_RENT_PUBKEY,
  PublicKey,
} from "@solana/web3.js";
import { TOKEN_PROGRAM_ID } from "@solana/spl-token";
import { Context } from "./ctx";

export async function initializeCrowdfunding(
  ctx: Context,
  maxLiquidations: number,
  incentiveCooldown: number,
  incentiveAmount: number | BN,
  platformFeeNum: number | BN,
  platformFeeDenom: number | BN,
  feeExemptionLimit: number | BN,
  liquidationLimit: number | BN
): Promise<void> {
  await ctx.program.methods
    .initialize(
      maxLiquidations,
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
      liquidatedSolVault: ctx.liquidatedSolVault,
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
      solVault: await ctx.solVault(id),
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
      liquidatedSolVault: ctx.liquidatedSolVault,
      campaign: await ctx.campaign(id),
      solVault: await ctx.solVault(id),
      feeExemptionVault: await ctx.feeExemptionVault(id),
      donor: await ctx.donor(donorAuthority.publicKey),
      donorAuthority: donorAuthority.publicKey,
      donations: await ctx.donations(
        donorAuthority.publicKey,
        await ctx.campaign(id)
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
        liquidatedSolVault: ctx.liquidatedSolVault,
        campaign: await ctx.campaign(id),
        solVault: await ctx.solVault(id),
        feeExemptionVault: await ctx.feeExemptionVault(id),
        donor: await ctx.donor(donorAuthority.publicKey),
        donorAuthority: donorAuthority.publicKey,
        donations: await ctx.donations(
          donorAuthority.publicKey,
          await ctx.campaign(id)
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

export async function incentivize(ctx: Context): Promise<void> {
  const remainingAccounts = [];

  const top = (await ctx.program.account.platform.fetch(ctx.platform))
    .platformTop;

  // @ts-ignore
  for (let i = 0; i < top.length; i++) {
    remainingAccounts.push({
      pubkey: await ctx.chrtATA(top[i].donor),
      isSigner: false,
      isWritable: true,
    });
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
      campaign: await ctx.campaign(id),
      campaignAuthority: ctx.campaignAuthority.publicKey,
      solVault: await ctx.solVault(id),
    })
    .signers([ctx.campaignAuthority])
    .rpc();
}

export async function stopCampaign(ctx: Context, id: number): Promise<void> {
  await ctx.program.methods
    .stopCampaign()
    .accounts({
      platform: ctx.platform,
      liquidatedSolVault: ctx.liquidatedSolVault,
      chrtMint: ctx.chrtMint,
      campaign: await ctx.campaign(id),
      campaignAuthority: ctx.campaignAuthority.publicKey,
      solVault: await ctx.solVault(id),
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
      liquidatedSolVault: ctx.liquidatedSolVault,
      chrtMint: ctx.chrtMint,
      campaign: await ctx.campaign(id),
      campaignAuthority: ctx.campaignAuthority.publicKey,
      solVault: await ctx.solVault(id),
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
