import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { Crowdfunding } from "../target/types/crowdfunding";

const program: Program<Crowdfunding> = anchor.workspace.Crowdfunding;

function findConstant(name: string): string {
  return program.idl.constants.find((c) => c.name == name).value;
}

export const CHRT_DECIMALS: number = Number(findConstant("CHRT_DECIMALS"));
export const PLATFORM_FEE_NUM: number = Number(
  findConstant("PLATFORM_FEE_NUM")
);
export const PLATFORM_FEE_DENOM: number = Number(
  findConstant("PLATFORM_FEE_DENOM")
);
export const SEASONAL_TOP_CAPACITY: number = Number(
  findConstant("SEASONAL_TOP_CAPACITY")
);
export const PLATFORM_TOP_CAPACITY: number = Number(
  findConstant("PLATFORM_TOP_CAPACITY")
);
export const CAMPAIGN_TOP_CAPACITY: number = Number(
  findConstant("CAMPAIGN_TOP_CAPACITY")
);
