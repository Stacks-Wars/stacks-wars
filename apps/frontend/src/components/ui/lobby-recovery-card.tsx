import { useState } from "react";
import {
	Card,
	CardContent,
	CardDescription,
	CardFooter,
	CardHeader,
	CardTitle,
} from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { AlertTriangle, Clock, Loader2 } from "lucide-react";
import { toast } from "sonner";
import {
	LobbyRecoveryData,
	SponsoredLobbyRecoveryData,
} from "@/lib/db/lobby-recovery";

interface LobbyRecoveryCardProps {
	recoveryData: LobbyRecoveryData | SponsoredLobbyRecoveryData;
	onContinue: () => Promise<void>;
	onDiscard: () => void;
	isSponsored?: boolean;
}

export default function LobbyRecoveryCard({
	recoveryData,
	onContinue,
	onDiscard,
	isSponsored = false,
}: LobbyRecoveryCardProps) {
	const [isLoading, setIsLoading] = useState(false);

	const handleContinue = async () => {
		setIsLoading(true);
		try {
			await onContinue();
		} catch (error) {
			toast.error("Failed to continue lobby creation");
			console.error("Recovery continue error:", error);
		} finally {
			setIsLoading(false);
		}
	};

	const getStatusMessage = () => {
		switch (recoveryData.status) {
			case "pending":
				return "Ready to deploy the contract and start lobby creation.";
			case "deployed":
				return isSponsored
					? "Contract deployed successfully. You need to join the pool to continue."
					: "Pool contract deployed successfully. You need to join the pool to continue.";
			case "joined":
				return "Pool joined successfully. Complete the lobby creation process.";
			default:
				return "Ready to continue lobby creation.";
		}
	};

	const getButtonText = () => {
		switch (recoveryData.status) {
			case "pending":
				return "Deploy & Continue";
			case "deployed":
				return "Join Pool & Complete";
			case "joined":
				return "Complete Creation";
			default:
				return "Continue";
		}
	};

	const formatCreatedAt = (timestamp: number) => {
		return new Date(timestamp).toLocaleString();
	};

	return (
		<Card className="border-orange-200 bg-orange-50/50 dark:border-orange-800 dark:bg-orange-950/50">
			<CardHeader>
				<CardTitle className="flex items-center gap-2 text-orange-800 dark:text-orange-200">
					<AlertTriangle className="h-5 w-5" />
					Resume Lobby Creation
				</CardTitle>
				<CardDescription className="text-orange-700 dark:text-orange-300">
					We found an interrupted lobby creation process. You can
					continue where you left off or start fresh.
				</CardDescription>
			</CardHeader>
			<CardContent className="space-y-4">
				<div className="grid grid-cols-1 gap-4 text-sm md:grid-cols-2">
					<div>
						<span className="font-medium">Lobby Name:</span>
						<p className="text-muted-foreground">
							{recoveryData.formData.name}
						</p>
					</div>
					{recoveryData.formData.description && (
						<div>
							<span className="font-medium">Description:</span>
							<p className="text-muted-foreground">
								{recoveryData.formData.description}
							</p>
						</div>
					)}
					{/*<div>
						<span className="font-medium">Type:</span>
						<p className="text-muted-foreground">
							{isSponsored ? "Sponsored Lobby" : "Pool Lobby"}
						</p>
					</div>*/}
					<div>
						<span className="font-medium">
							{isSponsored ? "Pool Size:" : "Entry Amount:"}
						</span>
						<p className="text-muted-foreground">
							{"poolSize" in recoveryData.formData
								? `${recoveryData.formData.poolSize}`
								: `${recoveryData.formData.amount} STX`}
						</p>
					</div>
				</div>

				<div className="text-muted-foreground flex items-center gap-2 text-sm">
					<Clock className="h-4 w-4" />
					<span>
						Started: {formatCreatedAt(recoveryData.createdAt)}
					</span>
				</div>

				<div className="rounded-md border border-blue-200 bg-blue-50 p-3 dark:border-blue-800 dark:bg-blue-950/50">
					<p className="text-sm text-blue-800 dark:text-blue-200">
						{getStatusMessage()}
					</p>
				</div>

				{recoveryData.deployedContract && (
					<div className="rounded-md border border-green-200 bg-green-50 p-3 dark:border-green-800 dark:bg-green-950/50">
						<p className="text-sm font-medium text-green-800 dark:text-green-200">
							Contract Deployed
						</p>
						<p className="font-mono text-xs break-all text-green-700 dark:text-green-300">
							{recoveryData.deployedContract.contractAddress}
						</p>
					</div>
				)}

				{recoveryData.joinedContract && (
					<div className="rounded-md border border-green-200 bg-green-50 p-3 dark:border-green-800 dark:bg-green-950/50">
						<p className="text-sm font-medium text-green-800 dark:text-green-200">
							Pool Joined
						</p>
						<p className="font-mono text-xs text-green-700 dark:text-green-300">
							Tx: {recoveryData.joinedContract.txId}
						</p>
					</div>
				)}
			</CardContent>
			<CardFooter className="flex gap-2">
				<Button
					onClick={handleContinue}
					disabled={isLoading}
					className="flex-1"
				>
					{isLoading && (
						<Loader2 className="mr-2 h-4 w-4 animate-spin" />
					)}
					{getButtonText()}
				</Button>
				<Button
					variant="outline"
					onClick={onDiscard}
					disabled={isLoading}
				>
					Discard
				</Button>
			</CardFooter>
		</Card>
	);
}
