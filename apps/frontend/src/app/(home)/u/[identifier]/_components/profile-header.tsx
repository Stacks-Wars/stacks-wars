"use client";

import { Avatar, AvatarFallback } from "@/components/ui/avatar";
import { Badge } from "@/components/ui/badge";
import { Card, CardContent } from "@/components/ui/card";
import { User } from "@/types/schema/user";
import { Copy, Check } from "lucide-react";
import { useState } from "react";
import { toast } from "sonner";

interface ProfileHeaderProps {
	user: User;
	isOwner: boolean;
}

function truncateAddress(address: string): string {
	return `${address.slice(0, 6)}...${address.slice(-4)}`;
}

export default function ProfileHeader({ user, isOwner }: ProfileHeaderProps) {
	const [copied, setCopied] = useState(false);

	const displayName =
		user.displayName ||
		user.username ||
		truncateAddress(user.walletAddress);

	const copyToClipboard = async () => {
		try {
			await navigator.clipboard.writeText(user.walletAddress);
			setCopied(true);
			toast.success("Wallet address copied!");
			setTimeout(() => setCopied(false), 2000);
		} catch (error) {
			console.error("Failed to copy address:", error);
			toast.error("Failed to copy address");
		}
	};

	return (
		<Card className="from-primary/10 to-primary/5 bg-linear-to-r">
			<CardContent className="p-6">
				<div className="flex flex-col items-center gap-6 sm:flex-row">
					<Avatar className="border-background h-24 w-24 border-4">
						<AvatarFallback className="bg-primary/20 text-2xl font-bold">
							{displayName.charAt(0).toUpperCase()}
						</AvatarFallback>
					</Avatar>

					<div className="flex-1 space-y-2 text-center sm:text-left">
						<div className="space-y-1">
							<h1 className="text-3xl font-bold tracking-tight">
								{displayName}
							</h1>
							{user.username && user.displayName && (
								<p className="text-muted-foreground text-lg">
									@{user.username}
								</p>
							)}
						</div>

						<div className="flex flex-col items-center gap-2 sm:flex-row">
							<div
								className="bg-muted hover:bg-muted/80 flex cursor-pointer items-center gap-2 rounded-lg px-3 py-1.5 transition-colors"
								onClick={copyToClipboard}
							>
								<span className="font-mono text-sm">
									{truncateAddress(user.walletAddress)}
								</span>
								{copied ? (
									<Check className="h-4 w-4 text-green-500" />
								) : (
									<Copy className="text-muted-foreground h-4 w-4" />
								)}
							</div>

							<Badge
								variant={
									user.warsPoint >= 0
										? "default"
										: "destructive"
								}
								className="font-mono"
							>
								{user.warsPoint.toFixed(0)} Wars Points
							</Badge>

							{isOwner && (
								<Badge
									variant="outline"
									className="text-primary"
								>
									Your Profile
								</Badge>
							)}
						</div>
					</div>
				</div>
			</CardContent>
		</Card>
	);
}
