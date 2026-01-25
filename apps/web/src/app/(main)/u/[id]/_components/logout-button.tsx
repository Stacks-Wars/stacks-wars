"use client";

import { useState } from "react";
import { Button } from "@/components/ui/button";
import { ApiClient } from "@/lib/api/client";
import { disconnectWallet } from "@/lib/wallet";
import { LogOut, Loader2 } from "lucide-react";
import { toast } from "sonner";
import { useRouter } from "next/navigation";

export default function LogoutButton() {
	const [isLoggingOut, setIsLoggingOut] = useState(false);
	const router = useRouter();

	const handleLogout = async () => {
		setIsLoggingOut(true);

		try {
			disconnectWallet();

			// Call backend logout to revoke token and clear cookie
			await ApiClient.post("/api/logout");

			router.refresh();

			toast.success("Logged out successfully");
		} catch (error) {
			console.error("Logout failed:", error);
			toast.error("Failed to logout");
		} finally {
			setIsLoggingOut(false);
		}
	};

	return (
		<Button
			onClick={handleLogout}
			disabled={isLoggingOut}
			variant="outline"
			className="rounded-full bg-muted text-xs sm:text-base h-6 sm:h-12 has-[>svg]:px-3.5 sm:has-[>svg]:px-7 -translate-y-1/2"
		>
			{isLoggingOut ? <Loader2 className="animate-spin" /> : <LogOut />}
			Logout
		</Button>
	);
}
