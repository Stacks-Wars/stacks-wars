import {
	AssetString,
	FungiblePostCondition,
	StxPostCondition,
} from "@stacks/transactions";
import { request } from "@stacks/connect";
import { getClaimFromJwt } from "../getClaimFromJwt";

const network = process.env.NEXT_PUBLIC_NETWORK || "testnet";

export const joinGamePool = async (
	contract: `${string}.${string}`,
	amount: number
) => {
	const address = await getClaimFromJwt<string>("wallet");
	if (!address) {
		throw new Error("No wallet address found");
	}
	try {
		const stxPostCondition: StxPostCondition = {
			type: "stx-postcondition",
			address,
			condition: "eq",
			amount: amount * 1_000_000,
		};

		const response = await request("stx_callContract", {
			contract,
			functionName: "join",
			functionArgs: [],
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

export const joinSponsoredGamePool = async (
	contract: `${string}.${string}`,
	isCreator: boolean,
	amount: number
) => {
	const address = await getClaimFromJwt<string>("wallet");
	if (!address) {
		throw new Error("No wallet address found");
	}

	try {
		let postConditions: StxPostCondition[] = [];

		if (isCreator) {
			const stxPostCondition: StxPostCondition = {
				type: "stx-postcondition",
				address,
				condition: "eq",
				amount: amount * 1_000_000,
			};
			postConditions = [stxPostCondition];
		}

		const response = await request("stx_callContract", {
			contract,
			functionName: "join",
			functionArgs: [],
			network,
			postConditionMode: "deny",
			postConditions,
		});
		return response.txid;
	} catch (error) {
		console.error("Wallet returned an error:", error);
		throw error;
	}
};

export const joinSponsoredFtGamePool = async (
	contract: `${string}.${string}`,
	tokenId: AssetString,
	isCreator: boolean,
	amount: number
) => {
	const walletAddress = await getClaimFromJwt<string>("wallet");
	if (!walletAddress) {
		throw new Error("No wallet address found");
	}

	try {
		let postConditions: FungiblePostCondition[] = [];

		if (isCreator) {
			const ftPostCondition: FungiblePostCondition = {
				type: "ft-postcondition",
				address: walletAddress,
				condition: "eq",
				asset: tokenId,
				amount: amount * 1_000_000,
			};
			// eslint-disable-next-line @typescript-eslint/no-unused-vars
			postConditions = [ftPostCondition];
		}

		const response = await request("stx_callContract", {
			contract,
			functionName: "join",
			functionArgs: [],
			network,
			postConditionMode: "allow",
			//postConditions,
		});
		return response.txid;
	} catch (error) {
		console.error("Wallet returned an error:", error);
		throw error;
	}
};
