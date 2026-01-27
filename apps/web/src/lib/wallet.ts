"use client";
import {
	connect,
	disconnect,
	getLocalStorage,
	isConnected,
	request,
} from "@stacks/connect";
import { nanoid } from "nanoid";

/**
 * Connect to a Stacks wallet
 * @returns Connected wallet addresses
 */
export async function connectWallet() {
	if (isConnected()) {
		console.log("Already authenticated");
		return getStxAddress();
	}

	const response = await connect();
	const address = response.addresses[2].address;
	return address;
}

/**
 * Disconnect from the current wallet
 */
export function disconnectWallet() {
	disconnect();
}

/**
 * Check if wallet is currently connected
 */
export function isWalletConnected() {
	return isConnected();
}

/**
 * Get cached wallet data from local storage
 */
export function getStxAddress() {
	const userData = getLocalStorage();

	if (!userData?.addresses) {
		return null;
	}

	const stxAddress = userData.addresses.stx[0]?.address;
	return stxAddress;
}

/**
 * Get full account details from the wallet
 */
export async function getAccountDetails() {
	const result = await request("stx_getAccounts");
	const account = result.accounts[0];

	return {
		address: account.address,
		publicKey: account.publicKey,
		gaiaHubUrl: account.gaiaHubUrl,
	};
}

/**
 * Transfer STX tokens
 */
export async function transferSTX(params: {
	amount: string;
	recipient: string;
	memo?: string;
}) {
	const response = await request("stx_transferStx", {
		amount: params.amount,
		recipient: params.recipient,
		memo: params.memo || "",
	});

	return response.txid;
}

/**
 * Deploy a Stacks contract
 */
export async function deployStacksContract(params: {
	clarityCode: string;
	tokenName: string;
}) {
	const name = `${nanoid(4)}-stacks-wars-${params.tokenName.toLocaleLowerCase()}-vault`;
	const network = process.env.NEXT_PUBLIC_NETWORK || "testnet";

	const result = await request("stx_deployContract", {
		name,
		clarityCode: params.clarityCode,
		network,
	});

	const contractAddress = `${getStxAddress()}.${name}`;

	return { txid: result.txid, contractAddress };
}
