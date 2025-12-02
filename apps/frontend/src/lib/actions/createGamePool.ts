import { request } from "@stacks/connect";
import { getSignerPublicKey } from "./txSigner";
import { getClarityCode } from "@/contracts/poolClarityCode";
import { getSponsoredClarityCode } from "@/contracts/sponsoredPoolClarityCode";
import { getSponsoredFtClarityCode } from "@/contracts/sponsoredFtPoolClarityCode";

const feeAddress = process.env.NEXT_PUBLIC_FEE_WALLET;
const network = process.env.NEXT_PUBLIC_NETWORK || "testnet";

export const transferFee = async () => {
	if (!feeAddress) {
		console.error("missing fee wallet");
		throw new Error("Fee wallet address not configured, contact support");
	}

	const feeAmount = 0.2 * 1_000_000; // 0.2 STX in microSTX

	try {
		return await request("stx_transferStx", {
			recipient: feeAddress,
			amount: feeAmount,
			network,
		});
	} catch (error) {
		console.error("Failed to transfer fee", error);
		throw error;
	}
};

export const createGamePool = async (
	amount: number,
	name: string,
	deployer: string
) => {
	const publicKey = await getSignerPublicKey();

	if (!feeAddress || !publicKey) {
		throw new Error("fee wallet address or public key not configured");
	}
	const clarityCode = getClarityCode(amount, deployer, feeAddress, publicKey);
	try {
		return await request("stx_deployContract", {
			name,
			clarityCode,
			network,
		});
	} catch (error) {
		console.error("Failed to deploy contract", error);
		throw error;
	}
};

export const createSponsoredGamePool = async (
	poolSize: number,
	name: string,
	deployer: string
) => {
	const publicKey = await getSignerPublicKey();

	if (!feeAddress || !publicKey) {
		throw new Error("fee wallet address or public key not configured");
	}
	const clarityCode = getSponsoredClarityCode(
		poolSize,
		deployer,
		feeAddress,
		publicKey
	);
	try {
		return await request("stx_deployContract", {
			name,
			clarityCode,
			network,
		});
	} catch (error) {
		console.error("Failed to deploy contract", error);
		throw error;
	}
};

export const createSponsoredFtGamePool = async (
	tokenContract: `'${string}.${string}`,
	tokenName: string,
	poolSize: number,
	contractName: string,
	deployer: string
) => {
	const publicKey = await getSignerPublicKey();

	if (!feeAddress || !publicKey) {
		throw new Error("fee wallet address or public key not configured");
	}
	const clarityCode = getSponsoredFtClarityCode(
		tokenContract,
		tokenName,
		poolSize,
		deployer,
		feeAddress,
		publicKey
	);
	try {
		return await request("stx_deployContract", {
			name: contractName,
			clarityCode,
			network,
		});
	} catch (error) {
		console.error("Failed to deploy FT sponsored contract", error);
		throw error;
	}
};
