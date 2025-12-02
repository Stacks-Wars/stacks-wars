import { useEffect, useState } from "react";
import { getClaimFromJwt } from "@/lib/getClaimFromJwt";
import { getWalletAddress, connectWallet } from "@/lib/wallet";
import { disconnect } from "@stacks/connect";
import { toast } from "sonner";
import { connectOrCreateUser, logoutUser } from "@/lib/actions/user";

export const useConnectUser = () => {
	const [isConnected, setIsConnected] = useState(false);
	const [walletAddress, setWalletAddress] = useState<string>();
	const [isConnecting, setIsConnecting] = useState(false);

	const checkConnection = async () => {
		if (typeof window === "undefined") return;

		const jwtWalletAddress = await getClaimFromJwt<string>("wallet");

		if (!jwtWalletAddress || getWalletAddress() !== jwtWalletAddress) {
			console.warn("Wallet mismatch or not found");
			disconnect();
			setIsConnected(false);
			setWalletAddress(undefined);
			return;
		}

		setWalletAddress(jwtWalletAddress);
		setIsConnected(true);
	};

	useEffect(() => {
		checkConnection();
	}, []);

	const handleConnect = async () => {
		setIsConnecting(true);
		try {
			const address = await connectWallet();
			if (!address)
				throw new Error("No address returned from connectWallet");

			await connectOrCreateUser(address);
			await checkConnection();
			toast.success("Wallet connected successfully!");
		} catch (error) {
			toast.error("Something went wrong, try again later.");
			console.error("Error connecting wallet:", error);
		}
		setIsConnecting(false);
	};

	const handleDisconnect = async () => {
		setIsConnecting(true);
		try {
			await logoutUser();
			toast.info("Wallet disconnected successfully.");
			setIsConnected(false);
			setWalletAddress(undefined);
		} catch (error) {
			console.error(error);
			toast.error("Failed to disconnect wallet. Please try again.");
		}
		setIsConnecting(false);
	};

	return {
		isConnecting,
		isConnected,
		walletAddress,
		handleConnect,
		handleDisconnect,
	};
};
