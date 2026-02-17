"use client";

import { Loader2 } from "lucide-react";
import { useRouter, useSearchParams } from "next/navigation";
import { useState } from "react";
import { toast } from "sonner";
import { Button } from "@/components/ui/button";
import { ApiClient } from "@/lib/api/client";
import type { User } from "@/lib/definitions";
import { useUser, useUserActions } from "@/lib/stores/user";

let connect: typeof import("@stacks/connect").connect;
let disconnect: typeof import("@stacks/connect").disconnect;
let isConnected: typeof import("@stacks/connect").isConnected;
if (typeof window !== "undefined") {
	const stacksConnect = require("@stacks/connect");
	connect = stacksConnect.connect;
	disconnect = stacksConnect.disconnect;
	isConnected = stacksConnect.isConnected;
}

export default function HandleConnect() {
	const router = useRouter();
	const searchParams = useSearchParams();
	const redirectUrl = searchParams.get("redirect");
	const [isLoading, setIsLoading] = useState(false);
	const { setUser, clearUser } = useUserActions();
	const user = useUser();

	const handleConnect = async () => {
		setIsLoading(true);

		// Check if already connected
		if (isConnected() || user != null) {
			disconnect();
			clearUser();
		}

		try {
			const walletAddress = (await connect()).addresses[2].address;

			// Authenticate with backend
			const authResponse = await ApiClient.post<User>("/api/user", {
				walletAddress,
			});

			if (authResponse.error || !authResponse.data) {
				throw new Error(authResponse.error || "Authentication failed");
			}

			setUser(authResponse.data);

			if (redirectUrl) {
				router.push(redirectUrl as any);
			} else {
				router.back();
			}
		} catch (err) {
			toast.error(
				err instanceof Error ? err.message : "Failed to connect wallet",
			);
		} finally {
			setIsLoading(false);
		}
	};

	return (
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
	);
}
