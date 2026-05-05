"use client";

import { useState } from "react";
import { Button } from "@/components/ui/button";
import { ApiClient } from "@/lib/api/client";
import { LogOut, Loader2 } from "lucide-react";
import { toast } from "sonner";
import { useRouter } from "next/navigation";
import { useUser, useUserActions } from "@/lib/stores/user";
import type { User } from "@/lib/definitions";

let disconnect: typeof import("@stacks/connect").disconnect;
if (typeof window !== "undefined") {
	disconnect = (await import("@stacks/connect")).disconnect;
}

interface LogoutButtonProps {
	userProfile: User;
}

export default function LogoutButton({ userProfile }: LogoutButtonProps) {
	const [isLoggingOut, setIsLoggingOut] = useState(false);
	const router = useRouter();
	const { clearUser } = useUserActions();
	const user = useUser();

	const handleLogout = async () => {
		setIsLoggingOut(true);

		try {
			disconnect();

			// Call backend logout to revoke token and clear cookie
			await ApiClient.post("/api/logout");

			// Refresh auth state (will clear user since token is revoked)
			router.refresh();
			clearUser();

			toast.success("Logged out successfully");
		} catch (error) {
			console.error("Logout failed:", error);
			toast.error("Failed to logout");
		} finally {
			setIsLoggingOut(false);
		}
	};

	return (
		<>
			{user?.id === userProfile.id && (
				<Button
					onClick={handleLogout}
					disabled={isLoggingOut}
					variant="outline"
					className="bg-muted h-6 -translate-y-1/2 rounded-full text-xs has-[>svg]:px-3.5 sm:h-12 sm:text-base sm:has-[>svg]:px-7"
				>
					{isLoggingOut ? (
						<Loader2 className="animate-spin" />
					) : (
						<LogOut />
					)}
					Logout
				</Button>
			)}
		</>
	);
}
