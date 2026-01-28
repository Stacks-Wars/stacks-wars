import { request } from "@stacks/connect";
import type {
	AssetString,
	ContractIdString,
	FungiblePostCondition,
	StxPostCondition,
} from "@stacks/transactions";
import { ClarityType } from "@stacks/transactions";
import { generateSignature } from "./signature";

/**
 * Leave a normal lobby contract
 */
export async function leaveNormalContract(params: {
	contract: ContractIdString;
	amount: number;
	walletAddress: string;
}) {
	const network = process.env.NEXT_PUBLIC_NETWORK || "mainnet";
	const signature = await generateSignature(
		params.amount,
		params.walletAddress,
		params.contract
	);

	const stxPostCondition: StxPostCondition = {
		type: "stx-postcondition",
		address: params.contract,
		condition: "eq",
		amount: params.amount * 1_000_000,
	};

	const response = await request("stx_callContract", {
		contract: params.contract,
		functionName: "leave",
		functionArgs: [{ type: ClarityType.Buffer, value: signature }],
		network,
		postConditionMode: "deny",
		postConditions: [stxPostCondition],
	});
	return response.txid;
}

/**
 * Leave a sponsored lobby contract
 */
export async function leaveSponsoredContract(params: {
	contract: ContractIdString;
	amount: number;
	walletAddress: string;
	isCreator: boolean;
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
		if (params.isCreator) {
			const stxPostCondition: StxPostCondition = {
				type: "stx-postcondition",
				address: params.contract,
				condition: "eq",
				amount: params.amount * 1_000_000,
			};
			postConditions = [stxPostCondition];
		}
	} else {
		if (params.isCreator && params.tokenId) {
			const ftPostCondition: FungiblePostCondition = {
				type: "ft-postcondition",
				address: params.contract,
				condition: "eq",
				asset: params.tokenId,
				amount: params.amount * 1_000_000,
			};
			postConditions = [ftPostCondition];
		}
	}

	const response = await request("stx_callContract", {
		contract: params.contract,
		functionName: "leave",
		functionArgs: [{ type: ClarityType.Buffer, value: signature }],
		network,
		postConditionMode: "deny",
		postConditions,
	});
	return response.txid;
}
