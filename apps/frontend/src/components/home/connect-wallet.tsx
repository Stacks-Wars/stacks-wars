"use client";
import { Button } from "@/components/ui/button";
import {
	DropdownMenu,
	DropdownMenuContent,
	DropdownMenuItem,
	DropdownMenuSeparator,
	DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { Loader, Wallet2, User, LogOut, ChevronDown } from "lucide-react";
import { truncateAddress } from "@/lib/utils";
import { useConnectUser } from "@/contexts/ConnectWalletContext";
import { useRouter } from "next/navigation";

export default function ConnectWallet() {
	const {
		isConnecting,
		isConnected,
		walletAddress,
		user,
		userLoading,
		handleConnect,
		handleDisconnect,
	} = useConnectUser();

	const router = useRouter();

	const getProfileIdentifier = () => {
		if (!user) return walletAddress;
		return user.username || user.walletAddress;
	};

	const handleProfileClick = () => {
		const identifier = getProfileIdentifier();
		if (identifier) {
			router.push(`/u/${identifier}`);
		}
	};

	const handleDisconnectClick = async () => {
		await handleDisconnect();
	};

	if (!isConnected) {
		return (
			<Button
				variant="outline"
				onClick={handleConnect}
				disabled={isConnecting}
			>
				{isConnecting ? (
					<Loader className="size-4 mr-1 animate-spin" />
				) : (
					<Wallet2 className="size-4 mr-1" />
				)}
				{isConnecting ? "Connecting..." : "Connect wallet"}
			</Button>
		);
	}

	return (
		<DropdownMenu>
			<DropdownMenuTrigger asChild>
				<Button
					variant="outline"
					disabled={isConnecting || userLoading}
					className="flex items-center gap-2"
				>
					{userLoading ? (
						<Loader className="size-4 animate-spin" />
					) : (
						<>
							<span className="font-mono">
								{user?.displayName ||
									user?.username ||
									truncateAddress(walletAddress)}
							</span>
							<ChevronDown className="size-4" />
						</>
					)}
				</Button>
			</DropdownMenuTrigger>
			<DropdownMenuContent align="end" className="w-full">
				<DropdownMenuItem
					onClick={handleProfileClick}
					className="cursor-pointer"
				>
					<User className="size-4 mr-2" />
					Profile
				</DropdownMenuItem>
				<DropdownMenuSeparator />
				<DropdownMenuItem
					onClick={handleDisconnectClick}
					className="cursor-pointer text-destructive focus:text-destructive"
					disabled={isConnecting}
				>
					{isConnecting ? (
						<Loader className="size-4 mr-2 animate-spin" />
					) : (
						<LogOut className="size-4 mr-2" />
					)}
					Disconnect Wallet
				</DropdownMenuItem>
			</DropdownMenuContent>
		</DropdownMenu>
	);
}
