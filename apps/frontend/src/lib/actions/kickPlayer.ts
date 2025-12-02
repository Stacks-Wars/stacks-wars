import {
	AssetString,
	ClarityType,
	FungiblePostCondition,
	StxPostCondition,
} from "@stacks/transactions";
import { request } from "@stacks/connect";

const network = process.env.NEXT_PUBLIC_NETWORK || "testnet";

export const kickFromPool = async (
	contract: `${string}.${string}`,
	playerAddress: string,
	amount: number
) => {
	try {
		amount = amount * 1_000_000;

		const stxPostCondition: StxPostCondition = {
			type: "stx-postcondition",
			address: contract,
			condition: "eq",
			amount,
		};

		const response = await request("stx_callContract", {
			contract,
			functionName: "kick",
			functionArgs: [
				{ type: ClarityType.PrincipalStandard, value: playerAddress },
			],
			network,
			postConditionMode: "deny",
			postConditions: [stxPostCondition],
		});
		return response.txid;
	} catch (error) {
		console.error("Wallet returned an error:", error);
		throw error;
	}
};

export const kickFromFtPool = async (
	contract: `${string}.${string}`,
	tokenId: AssetString,
	playerAddress: string,
	amount: number
) => {
	try {
		amount = amount * 1_000_000;

		const ftPostCondition: FungiblePostCondition = {
			type: "ft-postcondition",
			address: contract,
			condition: "eq",
			asset: tokenId,
			amount,
		};

		const response = await request("stx_callContract", {
			contract,
			functionName: "kick",
			functionArgs: [
				{ type: ClarityType.PrincipalStandard, value: playerAddress },
			],
			network,
			postConditionMode: "deny",
			postConditions: [ftPostCondition],
		});
		return response.txid;
	} catch (error) {
		console.error("Wallet returned an error:", error);
		throw error;
	}
};
