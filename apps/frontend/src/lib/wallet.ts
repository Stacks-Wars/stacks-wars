import { connect, disconnect, getLocalStorage } from "@stacks/connect";
import { toast } from "sonner";
import { getClaimFromJwt } from "./getClaimFromJwt";

export const connectWallet = async () => {
	try {
		const response = await connect({
			//network: "testnet"
		}); // stores users address in local storage by default
		return response.addresses[2].address; // returns stx address
	} catch (error) {
		toast.error("Failed to connect wallet. Please try again.");
		console.error("Error connecting wallet:", error);
	}
};

export const isConnected = async (): Promise<boolean> => {
	const walletAddress = await getClaimFromJwt<string>("wallet");

	if (!walletAddress) return false;

	if (getWalletAddress() !== walletAddress) {
		console.warn("Wallet address does not match JWT claim");
		disconnect();
		return false;
	}

	return true;
};

export const getWalletAddress = () => {
	const data = getLocalStorage();
	return data?.addresses.stx[0].address;
};

export const disconnectWallet = () => {
	try {
		disconnect();
		toast.info("Wallet disconnected successfully.");
	} catch (error) {
		toast.error("Failed to disconnect wallet. Please try again.");
		console.error("Error disconnecting wallet:", error);
	}
};
