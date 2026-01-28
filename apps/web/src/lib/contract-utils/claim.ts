import { request } from "@stacks/connect";
import type { ContractIdString, StxPostCondition } from "@stacks/transactions";
import { ClarityType } from "@stacks/transactions";
import { generateSignature } from "./signature";
import type { FungiblePostCondition, AssetString } from "@stacks/transactions";

/**
 * Claim rewards from the contract
 * Adds a postcondition for STX or FT claim
 */
export async function claimRewardContract(params: {
	contract: ContractIdString;
	amount: number;
	walletAddress: string;
	tokenId: AssetString;
}) {
	const network = process.env.NEXT_PUBLIC_NETWORK || "mainnet";
	const signature = await generateSignature(
		params.amount,
		params.walletAddress,
		params.contract
	);

	let postConditions: (StxPostCondition | FungiblePostCondition)[] = [];

	if (params.contract.endsWith("-stacks-wars-stx-vault")) {
		const stxPostCondition: StxPostCondition = {
			type: "stx-postcondition",
			address: params.contract,
			condition: "lte",
			amount: params.amount * 1_000_000,
		};
		postConditions = [stxPostCondition];
	} else {
		const ftPostCondition: FungiblePostCondition = {
			type: "ft-postcondition",
			address: params.contract,
			condition: "lte",
			asset: params.tokenId,
			amount: params.amount * 1_000_000,
		};
		postConditions = [ftPostCondition];
	}

	const response = await request("stx_callContract", {
		contract: params.contract,
		functionName: "claim",
		functionArgs: [
			{ type: ClarityType.UInt, value: params.amount * 1_000_000 },
			{ type: ClarityType.Buffer, value: signature },
		],
		network,
		postConditionMode: "deny",
		postConditions,
	});
	return response.txid;
}
