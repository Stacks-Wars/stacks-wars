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
		<Card className="bg-gradient-to-r from-primary/10 to-primary/5">
			<CardContent className="p-6">
				<div className="flex flex-col sm:flex-row items-center gap-6">
					<Avatar className="h-24 w-24 border-4 border-background">
						<AvatarFallback className="bg-primary/20 text-2xl font-bold">
							{displayName.charAt(0).toUpperCase()}
						</AvatarFallback>
					</Avatar>

					<div className="flex-1 text-center sm:text-left space-y-2">
						<div className="space-y-1">
							<h1 className="text-3xl font-bold tracking-tight">
								{displayName}
							</h1>
							{user.username && user.displayName && (
								<p className="text-lg text-muted-foreground">
									@{user.username}
								</p>
							)}
						</div>

						<div className="flex flex-col sm:flex-row items-center gap-2">
							<div
								className="flex items-center gap-2 bg-muted px-3 py-1.5 rounded-lg cursor-pointer hover:bg-muted/80 transition-colors"
								onClick={copyToClipboard}
							>
								<span className="font-mono text-sm">
									{truncateAddress(user.walletAddress)}
								</span>
								{copied ? (
									<Check className="h-4 w-4 text-green-500" />
								) : (
									<Copy className="h-4 w-4 text-muted-foreground" />
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
