"use client";
import {
	createContext,
	useCallback,
	useContext,
	useEffect,
	useState,
	type ReactNode,
} from "react";
import { getClaimFromJwt } from "@/lib/getClaimFromJwt";
import { getWalletAddress, connectWallet } from "@/lib/wallet";
import { disconnect } from "@stacks/connect";
import { toast } from "sonner";
import { connectOrCreateUser, logoutUser } from "@/lib/actions/user";
import { apiRequest } from "@/lib/api";
import { User } from "@/types/schema/user";

interface ConnectUserContextType {
	isConnected: boolean;
	isConnecting: boolean;
	walletAddress?: string;
	user: User | null;
	userLoading: boolean;
	handleConnect: () => Promise<void>;
	handleDisconnect: () => Promise<void>;
	refetchUser: () => Promise<void>;
}

const ConnectUserContext = createContext<ConnectUserContextType | undefined>(
	undefined
);

export const ConnectUserProvider = ({ children }: { children: ReactNode }) => {
	const [isConnected, setIsConnected] = useState(false);
	const [walletAddress, setWalletAddress] = useState<string>();
	const [isConnecting, setIsConnecting] = useState(false);
	const [user, setUser] = useState<User | null>(null);
	const [userLoading, setUserLoading] = useState(false);

	const fetchUser = useCallback(async () => {
		try {
			setUserLoading(true);
			const userId = await getClaimFromJwt<string>("sub");
			if (!userId) {
				setUser(null);
				return;
			}

			const userData = await apiRequest<User>({
				path: `/user/${userId}`,
				method: "GET",
			});
			setUser(userData);
		} catch (error) {
			console.error("Failed to fetch user:", error);
			setUser(null);
		} finally {
			setUserLoading(false);
		}
	}, []);

	const refetchUser = useCallback(async () => {
		await fetchUser();
	}, [fetchUser]);

	const checkConnection = useCallback(async () => {
		const jwtWalletAddress = await getClaimFromJwt<string>("wallet");

		if (!jwtWalletAddress || getWalletAddress() !== jwtWalletAddress) {
			console.warn("Wallet mismatch or not found");
			disconnect();
			setIsConnected(false);
			setWalletAddress(undefined);
			setUser(null);
			return;
		}

		setWalletAddress(jwtWalletAddress);
		setIsConnected(true);
		// Fetch user data when connected
		await fetchUser();
	}, [fetchUser]);

	useEffect(() => {
		checkConnection();
	}, [checkConnection]);

	const handleConnect = useCallback(async () => {
		setIsConnecting(true);
		try {
			const address = await connectWallet();
			if (!address)
				throw new Error("No address returned from connectWallet");

			await connectOrCreateUser(address);
			await checkConnection();
			toast.success("Wallet connected successfully!");

			window.location.reload();
		} catch (error) {
			toast.error("Something went wrong, try again later.");
			console.error("Error connecting wallet:", error);
		}
		setIsConnecting(false);
	}, [checkConnection]);

	const handleDisconnect = useCallback(async () => {
		setIsConnecting(true);
		try {
			await logoutUser();
			toast.info("Wallet disconnected successfully.");
			setIsConnected(false);
			setWalletAddress(undefined);
			setUser(null);

			window.location.reload();
		} catch (error) {
			console.error(error);
			toast.error("Failed to disconnect wallet. Please try again.");
		}
		setIsConnecting(false);
	}, []);

	return (
		<ConnectUserContext.Provider
			value={{
				isConnected,
				isConnecting,
				walletAddress,
				user,
				userLoading,
				handleConnect,
				handleDisconnect,
				refetchUser,
			}}
		>
			{children}
		</ConnectUserContext.Provider>
	);
};

export const useConnectUser = () => {
	const context = useContext(ConnectUserContext);
	if (!context)
		throw new Error(
			"useConnectUser must be used within a ConnectUserProvider"
		);
	return context;
};
