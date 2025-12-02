import {
	AssetString,
	ClarityType,
	FungiblePostCondition,
	StxPostCondition,
} from "@stacks/transactions";
import { request } from "@stacks/connect";
import { generateSignature } from "./txSigner";
import { getClaimFromJwt } from "../getClaimFromJwt";

const network = process.env.NEXT_PUBLIC_NETWORK || "testnet";

export const leaveGamePool = async (
	contract: `${string}.${string}`,
	amount: number
) => {
	const walletAddress = await getClaimFromJwt<string>("wallet");
	if (!walletAddress) {
		throw new Error("No wallet address found");
	}
	amount = amount * 1_000_000;

	try {
		const signature = await generateSignature(
			amount,
			walletAddress,
			contract
		);

		const stxPostCondition: StxPostCondition = {
			type: "stx-postcondition",
			address: contract,
			condition: "eq",
			amount,
		};

		const response = await request("stx_callContract", {
			contract,
			functionName: "leave",
			functionArgs: [{ type: ClarityType.Buffer, value: signature }],
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

export const leaveSponsoredGamePool = async (
	contract: `${string}.${string}`,
	isCreator: boolean,
	amount: number
) => {
	const walletAddress = await getClaimFromJwt<string>("wallet");
	if (!walletAddress) {
		throw new Error("No wallet address found");
	}
	amount = amount * 1_000_000;

	try {
		const signature = await generateSignature(
			amount,
			walletAddress,
			contract
		);

		let postConditions: StxPostCondition[] = [];

		if (isCreator) {
			const stxPostCondition: StxPostCondition = {
				type: "stx-postcondition",
				address: contract,
				condition: "eq",
				amount: amount,
			};
			postConditions = [stxPostCondition];
		}

		const response = await request("stx_callContract", {
			contract,
			functionName: "leave",
			functionArgs: [{ type: ClarityType.Buffer, value: signature }],
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

export const leaveSponsoredFtGamePool = async (
	contract: `${string}.${string}`,
	tokenId: AssetString,
	isCreator: boolean,
	amount: number
) => {
	const walletAddress = await getClaimFromJwt<string>("wallet");
	if (!walletAddress) {
		throw new Error("No wallet address found");
	}
	amount = amount * 1_000_000;

	try {
		const signature = await generateSignature(
			amount,
			walletAddress,
			contract
		);

		let postConditions: FungiblePostCondition[] = [];

		if (isCreator) {
			const ftPostCondition: FungiblePostCondition = {
				type: "ft-postcondition",
				address: contract,
				condition: "eq",
				asset: tokenId,
				amount: amount,
			};
			postConditions = [ftPostCondition];
		}

		const response = await request("stx_callContract", {
			contract,
			functionName: "leave",
			functionArgs: [{ type: ClarityType.Buffer, value: signature }],
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
