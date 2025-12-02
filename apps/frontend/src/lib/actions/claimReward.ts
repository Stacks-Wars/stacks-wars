import {
	AssetString,
	ClarityType,
	FungiblePostCondition,
	StxPostCondition,
} from "@stacks/transactions";
import { generateSignature } from "./txSigner";
import { request } from "@stacks/connect";
import { getClaimFromJwt } from "../getClaimFromJwt";

const network = process.env.NEXT_PUBLIC_NETWORK || "testnet";

export const claimPoolReward = async (
	contract: `${string}.${string}`,
	amount: number
) => {
	const walletAddress = await getClaimFromJwt<string>("wallet");
	if (!walletAddress) {
		throw new Error("No wallet address found");
	}
	amount = Math.floor(amount * 1_000_000);

	try {
		const signature = await generateSignature(
			amount,
			walletAddress,
			contract
		);

		const stxPostCondition: StxPostCondition = {
			type: "stx-postcondition",
			address: contract,
			condition: "lte",
			amount,
		};

		const response = await request("stx_callContract", {
			contract,
			functionName: "claim-reward",
			functionArgs: [
				{ type: ClarityType.UInt, value: amount },
				{ type: ClarityType.Buffer, value: signature },
			],
			network,
			postConditionMode: "deny",
			postConditions: [stxPostCondition],
		});
		return response.txid;
	} catch (error) {
		console.error("Error claiming reward:", error);
		throw error;
	}
};

export const claimFtPoolReward = async (
	contract: `${string}.${string}`,
	asset: AssetString,
	amount: number
) => {
	const walletAddress = await getClaimFromJwt<string>("wallet");
	if (!walletAddress) {
		throw new Error("No wallet address found");
	}
	amount = Math.floor(amount * 1_000_000);

	try {
		const signature = await generateSignature(
			amount,
			walletAddress,
			contract
		);

		// eslint-disable-next-line @typescript-eslint/no-unused-vars
		const ftPostCondition: FungiblePostCondition = {
			type: "ft-postcondition",
			address: contract,
			condition: "lte",
			asset,
			amount,
		};

		const response = await request("stx_callContract", {
			contract,
			functionName: "claim-reward",
			functionArgs: [
				{ type: ClarityType.UInt, value: amount },
				{ type: ClarityType.Buffer, value: signature },
			],
			network,
			postConditionMode: "allow",
			//postConditions: [ftPostCondition],
		});
		return response.txid;
	} catch (error) {
		console.error("Error claiming reward:", error);
		throw error;
	}
};
