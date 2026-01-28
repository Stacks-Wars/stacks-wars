import { request } from "@stacks/connect";
import type {
	AssetString,
	ContractIdString,
	FungiblePostCondition,
	StxPostCondition,
} from "@stacks/transactions";

/**
 * Join a normal lobby contract
 */
export async function joinNormalContract(params: {
	contract: ContractIdString;
	amount: number;
	address: string;
}) {
	const network = process.env.NEXT_PUBLIC_NETWORK || "mainnet";

	const stxPostCondition: StxPostCondition = {
		type: "stx-postcondition",
		address: params.address,
		condition: "eq",
		amount: params.amount * 1_000_000,
	};

	const response = await request("stx_callContract", {
		contract: params.contract,
		functionName: "join",
		functionArgs: [],
		network,
		postConditionMode: "deny",
		postConditions: [stxPostCondition],
	});

	return response.txid;
}

/**
 * Join a sponsored lobby contract
 */
export async function joinSponsoredContract(params: {
	contract: ContractIdString;
	amount: number;
	isCreator: boolean;
	tokenId?: AssetString;
	address: string;
}) {
	const network = process.env.NEXT_PUBLIC_NETWORK || "mainnet";

	let postConditions: (StxPostCondition | FungiblePostCondition)[] = [];

	if (params.isCreator) {
		if (params.contract.endsWith("-stacks-wars-stx-vault")) {
			const stxPostCondition: StxPostCondition = {
				type: "stx-postcondition",
				address: params.address,
				condition: "eq",
				amount: params.amount * 1_000_000,
			};
			postConditions = [stxPostCondition];
		} else if (params.tokenId) {
			const ftPostCondition: FungiblePostCondition = {
				type: "ft-postcondition",
				address: params.address,
				condition: "eq",
				asset: params.tokenId,
				amount: params.amount * 1_000_000,
			};
			postConditions = [ftPostCondition];
		}
	}

	const response = await request("stx_callContract", {
		contract: params.contract,
		functionName: "join",
		functionArgs: [],
		network,
		postConditionMode: "deny",
		postConditions,
	});

	return response.txid;
}
