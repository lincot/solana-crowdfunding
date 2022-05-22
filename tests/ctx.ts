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

  donor1: Keypair;
  donor2: Keypair;

  constructor() {
    this.connection = new Connection("http://localhost:8899", "recent");
    this.program = anchor.workspace.Crowdfunding;
    this.payer = new Keypair();

    this.platformAuthority = new Keypair();
    this.campaignAuthority = new Keypair();
    this.donor1 = new Keypair();
    this.donor2 = new Keypair();
  }

  async setup() {
    await airdrop(this, [
      this.platformAuthority.publicKey,
      this.campaignAuthority.publicKey,
      this.donor1.publicKey,
      this.donor2.publicKey,
    ]);

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

  async donations(
    donorAuthority: PublicKey,
    campaign: PublicKey
  ): Promise<PublicKey> {
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

  async feeExemptionVault(campaignId: number): Promise<PublicKey> {
    return await findPDA(
      [
        Buffer.from("fee_exemption_vault"),
        new BN(campaignId).toArrayLike(Buffer, "le", 2),
      ],
      this.program.programId
    );
  }

  async liquidationVault(campaignId: number): Promise<PublicKey> {
    return await findPDA(
      [
        Buffer.from("liquidation_vault"),
        new BN(campaignId).toArrayLike(Buffer, "le", 2),
      ],
      this.program.programId
    );
  }

  async chrtATA(owner: PublicKey): Promise<TokenAccount> {
    return await findATA(this, owner, this.chrtMint);
  }

  async activeCampaigns() {
    const platform = await this.program.account.platform.fetch(this.platform);

    // @ts-ignore
    return platform.activeCampaigns.map((c: any) => {
      c.donationsSum = c.donationsSum.toNumber();
      c.withdrawnSum = c.withdrawnSum.toNumber();
      return c;
    });
  }

  async platformTop() {
    const platform = await this.program.account.platform.fetch(this.platform);

    // @ts-ignore
    return platform.top.map((d: any) => {
      d.donationsSum = d.donationsSum.toNumber();
      return d;
    });
  }

  async campaignTop(campaignId: number) {
    const campaign = await this.program.account.campaign.fetch(
      await this.campaign(campaignId)
    );

    // @ts-ignore
    return campaign.top.map((d: any) => {
      d.donationsSum = d.donationsSum.toNumber();
      return d;
    });
  }

  async solVaultBalance() {
    return (await this.connection.getBalance(this.solVault)) - 890880;
  }

  async feeVaultBalance() {
    return (await this.connection.getBalance(this.feeVault)) - 890880;
  }
}
