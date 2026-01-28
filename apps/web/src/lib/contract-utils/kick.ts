import { request } from "@stacks/connect";
import type {
	AssetString,
	ContractIdString,
	StxPostCondition,
} from "@stacks/transactions";
import { ClarityType } from "@stacks/transactions";
import { generateSignature } from "./signature";
import type { FungiblePostCondition } from "@stacks/transactions";

/**
 * Kick a player from the lobby (creator only)
 * Supports both STX and FT vaults
 */
export async function kickPlayerContract(params: {
	contract: ContractIdString;
	playerAddress: string;
	amount: number;
	tokenId: AssetString;
}) {
	const network = process.env.NEXT_PUBLIC_NETWORK || "mainnet";
	const signature = await generateSignature(
		params.amount,
		params.playerAddress,
		params.contract
	);

	let postConditions: (StxPostCondition | FungiblePostCondition)[] = [];

	if (params.contract.endsWith("-stacks-wars-stx-vault")) {
		const stxPostCondition: StxPostCondition = {
			type: "stx-postcondition",
			address: params.contract,
			condition: "eq",
			amount: params.amount * 1_000_000,
		};
		postConditions = [stxPostCondition];
	} else {
		const ftPostCondition: FungiblePostCondition = {
			type: "ft-postcondition",
			address: params.contract,
			condition: "eq",
			asset: params.tokenId,
			amount: params.amount * 1_000_000,
		};
		postConditions = [ftPostCondition];
	}

	const response = await request("stx_callContract", {
		contract: params.contract,
		functionName: "kick",
		functionArgs: [
			{
				type: ClarityType.PrincipalStandard,
				value: params.playerAddress,
			},
			{ type: ClarityType.Buffer, value: signature },
		],
		network,
		postConditionMode: "deny",
		postConditions,
	});
	return response.txid;
}
