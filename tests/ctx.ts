import * as anchor from "@project-serum/anchor";
import { BN, Program } from "@project-serum/anchor";
import { Connection, Keypair, PublicKey } from "@solana/web3.js";
import { Crowdfunding } from "../target/types/crowdfunding";
import { findATA, TokenAccount } from "./token";
import { airdrop, findPDA } from "./utils";

export class Context {
  connection: Connection;

  program: Program<Crowdfunding>;

  payer: Keypair;

  platform: PublicKey;
  platformAuthority: Keypair;
  feeVault: PublicKey;
  solVault: PublicKey;

  chrtMint: PublicKey;

  campaignAuthority: Keypair;

  donors: Keypair[];

  constructor() {
    const provider = anchor.AnchorProvider.env();
    anchor.setProvider(provider);
    this.connection = provider.connection;
    this.program = anchor.workspace.Crowdfunding;
    this.payer = new Keypair();

    this.platformAuthority = new Keypair();
    this.campaignAuthority = new Keypair();
    this.donors = [];
    for (let i = 0; i < 15; i++) {
      this.donors.push(new Keypair());
    }
  }

  async setup() {
    await airdrop(
      this,
      [
        this.platformAuthority.publicKey,
        this.campaignAuthority.publicKey,
      ].concat(this.donors.map((d) => d.publicKey))
    );

    this.platform = await findPDA(
      [Buffer.from("platform")],
      this.program.programId
    );
    this.feeVault = await findPDA(
      [Buffer.from("fee_vault")],
      this.program.programId
    );
    this.solVault = await findPDA(
      [Buffer.from("sol_vault")],
      this.program.programId
    );
    this.chrtMint = await findPDA(
      [Buffer.from("chrt_mint")],
      this.program.programId
    );
  }

  async donor(donorAuthority: PublicKey): Promise<PublicKey> {
    return await findPDA(
      [Buffer.from("donor"), donorAuthority.toBuffer()],
      this.program.programId
    );
  }

  async donorDonationsToCampaign(
    donorAuthority: PublicKey,
    campaignId: number
  ): Promise<PublicKey> {
    const campaign = await this.campaign(campaignId);

    return await findPDA(
      [
        Buffer.from("donations"),
        donorAuthority.toBuffer(),
        campaign.toBuffer(),
      ],
      this.program.programId
    );
  }

  async campaign(campaignId: number): Promise<PublicKey> {
    return await findPDA(
      [
        Buffer.from("campaign"),
        new BN(campaignId).toArrayLike(Buffer, "le", 2),
      ],
      this.program.programId
    );
  }

  async totalDonationsToCampaign(campaignId: number): Promise<PublicKey> {
    const campaign = await this.campaign(campaignId);

    return await findPDA(
      [Buffer.from("donations"), campaign.toBuffer()],
      this.program.programId
    );
  }

  async feeExemptionVault(campaignId: number): Promise<TokenAccount> {
    const address = await findPDA(
      [
        Buffer.from("fee_exemption_vault"),
        new BN(campaignId).toArrayLike(Buffer, "le", 2),
      ],
      this.program.programId
    );
    return new TokenAccount(address, this.chrtMint);
  }

  async liquidationVault(campaignId: number): Promise<TokenAccount> {
    const address = await findPDA(
      [
        Buffer.from("liquidation_vault"),
        new BN(campaignId).toArrayLike(Buffer, "le", 2),
      ],
      this.program.programId
    );
    return new TokenAccount(address, this.chrtMint);
  }

  async chrtATA(owner: PublicKey): Promise<TokenAccount> {
    return await findATA(this, owner, this.chrtMint);
  }

  async activeCampaigns() {
    const platform = await this.program.account.platform.fetch(this.platform);

    // @ts-ignore
    return platform.activeCampaigns
      .slice(0, platform.activeCampaignsCount)
      .map((c: any) => {
        c.donationsSum = c.donationsSum.toNumber();
        c.withdrawnSum = c.withdrawnSum.toNumber();
        return c;
      });
  }

  async platformTop() {
    const platform = await this.program.account.platform.fetch(this.platform);

    // @ts-ignore
    for (var i = 0; i < platform.top.length; i++) {
      if (platform.top[i].donor.toBuffer().every((i) => i === 0)) {
        break;
      }
    }

    // @ts-ignore
    return platform.top.slice(0, i).map((d: any) => {
      d.donationsSum = d.donationsSum.toNumber();
      return d;
    });
  }

  async campaignTop(campaignId: number) {
    const campaign = await this.program.account.campaign.fetch(
      await this.campaign(campaignId)
    );

    // @ts-ignore
    for (var i = 0; i < campaign.top.length; i++) {
      if (campaign.top[i].donor.toBuffer().every((i) => i === 0)) {
        break;
      }
    }

    // @ts-ignore
    return campaign.top.slice(0, i).map((d: any) => {
      d.donationsSum = d.donationsSum.toNumber();
      return d;
    });
  }

  async solVaultBalance() {
    return (
      (await this.connection.getBalance(this.solVault)) -
      (await this.connection.getMinimumBalanceForRentExemption(9))
    );
  }

  async feeVaultBalance() {
    return (
      (await this.connection.getBalance(this.feeVault)) -
      (await this.connection.getMinimumBalanceForRentExemption(9))
    );
  }
}
