"use client";

import { Loader2 } from "lucide-react";
import { useRouter } from "next/navigation";
import { useState } from "react";
import { Button } from "@/components/ui/button";
import {
	Card,
	CardContent,
	CardDescription,
	CardHeader,
	CardTitle,
} from "@/components/ui/card";
import { ApiClient } from "@/lib/api/client";
import type { User } from "@/lib/definitions";
import { useUser, useUserActions } from "@/lib/stores/user";
import {
	connectWallet,
	disconnectWallet,
	isWalletConnected,
} from "@/lib/wallet";

export default function LoginModal() {
	const router = useRouter();
	const [isLoading, setIsLoading] = useState(false);
	const [error, setError] = useState<string | null>(null);
	const { setUser, clearUser } = useUserActions();
	const user = useUser();

	const handleConnect = async () => {
		// Check if already connected
		if (isWalletConnected() || user != null) {
			disconnectWallet();
			clearUser();
		}

		setIsLoading(true);
		setError(null);

		try {
			const walletAddress = await connectWallet();

			// Authenticate with backend
			const authResponse = await ApiClient.post<User>("/api/user", {
				walletAddress,
			});

			if (authResponse.error || !authResponse.data) {
				throw new Error(authResponse.error || "Authentication failed");
			}

			setUser(authResponse.data);

			router.back();
		} catch (err) {
			setError(
				err instanceof Error ? err.message : "Failed to connect wallet"
			);
		} finally {
			setIsLoading(false);
		}
	};

	return (
		<div className="flex min-h-screen items-center justify-center p-4">
			<Card>
				<CardContent className="sm:max-w-md">
					<CardHeader>
						<CardTitle>Connect Wallet</CardTitle>
						<CardDescription>
							Connect your Stacks wallet to join games and compete
							with other players.
						</CardDescription>
					</CardHeader>
					<div className="flex flex-col gap-4">
						{error && (
							<div className="rounded-md bg-destructive/10 p-3 text-sm text-destructive">
								{error}
							</div>
						)}
						<Button
							onClick={handleConnect}
							disabled={isLoading}
							className="w-full"
							size="lg"
						>
							{isLoading ? (
								<>
									<Loader2 className="mr-2 h-4 w-4 animate-spin" />
									Connecting...
								</>
							) : (
								"Connect with Stacks Wallet"
							)}
						</Button>
						<p className="text-center text-xs text-muted-foreground">
							By connecting your wallet, you agree to our Terms of
							Service and Privacy Policy.
						</p>
					</div>
				</CardContent>
			</Card>
		</div>
	);
}
