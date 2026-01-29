"use client";

import {
	Dialog,
	DialogContent,
	DialogHeader,
	DialogTitle,
	DialogDescription,
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import {
	useGameOverData,
	useIsActionLoading,
	useLobby,
	useLobbyActions,
} from "@/lib/stores/room";
import { useUser } from "@/lib/stores/user";
import { claimRewardContract } from "@/lib/contract-utils/claim";
import { waitForTxConfirmed } from "@/lib/contract-utils/waitForTxConfirmed";
import type {
	AssetString,
	ContractIdString,
} from "@stacks/connect/dist/types/methods";
import { toast } from "sonner";
import { Trophy, Sparkles, Coins, Loader2 } from "lucide-react";
import { cn } from "@/lib/utils";
import { useRoom } from "@/lib/contexts/room-context";

const rankLabels: Record<number, string> = {
	1: "1st Place",
	2: "2nd Place",
	3: "3rd Place",
};

const rankColors: Record<number, string> = {
	1: "text-yellow-500",
	2: "text-gray-400",
	3: "text-amber-600",
};

export default function GameOverModal() {
	const gameOverData = useGameOverData();
	const lobbyActions = useLobbyActions();
	const lobby = useLobby();
	const user = useUser();
	const { sendLobbyMessage } = useRoom();
	const isClaiming = useIsActionLoading("claimReward");

	const handleClose = () => {
		lobbyActions.setGameOver(null);
	};

	const handleClaim = async () => {
		if (!lobby || !user || !gameOverData?.prize || !lobby.contractAddress) {
			toast.error("Missing data for claim.");
			return;
		}
		try {
			const contract = lobby.contractAddress as ContractIdString;
			const tokenId =
				`${lobby.tokenContractId}::${lobby.tokenSymbol}` as AssetString;
			const txId = await claimRewardContract({
				contract,
				amount: gameOverData.prize,
				walletAddress: user.walletAddress,
				tokenId,
			});
			if (!txId) {
				toast.error("Failed to claim reward", {
					description: "Please try again later.",
				});
				return;
			}
			await waitForTxConfirmed(txId);
			sendLobbyMessage({ type: "claimReward", txId });
		} catch (err) {
			toast.error("Contract transaction failed. Please try again.");
			console.error("Claim contract failed", err);
		}
	};

	if (!gameOverData) return null;

	const { rank, prize, warsPoint } = gameOverData;
	const isTopThree = rank <= 3;
	const rankLabel = rankLabels[rank] || `#${rank}`;
	const rankColor = rankColors[rank] || "text-muted-foreground";

	return (
		<Dialog
			open={!!gameOverData}
			onOpenChange={(open) => !open && handleClose()}
		>
			<DialogContent
				className="sm:max-w-md"
				showCloseButton={false}
				disableOutsideClose={true}
			>
				<DialogHeader className="text-center">
					<DialogTitle className="text-2xl font-bold">
						Game Over!
					</DialogTitle>
					<DialogDescription className="sr-only">
						Your game results and rewards
					</DialogDescription>
				</DialogHeader>

				<div className="flex flex-col items-center gap-6 py-4">
					{/* Rank Display */}
					<div className="flex flex-col items-center gap-2">
						{isTopThree && (
							<Trophy
								className={cn("size-16", rankColor)}
								strokeWidth={1.5}
							/>
						)}
						<p
							className={cn(
								"text-4xl font-bold",
								isTopThree ? rankColor : "text-foreground"
							)}
						>
							{rankLabel}
						</p>
						{!isTopThree && (
							<p className="text-muted-foreground text-sm">
								Better luck next time!
							</p>
						)}
					</div>

					{/* Rewards Section */}
					<div className="w-full space-y-3">
						{/* Prize (if won) */}
						{prize != null && prize > 0 && (
							<div className="flex items-center justify-between rounded-lg border bg-card p-4">
								<div className="flex items-center gap-3">
									<div className="flex size-10 items-center justify-center rounded-full bg-green-500/10">
										<Coins className="size-5 text-green-500" />
									</div>
									<span className="font-medium">
										Prize Won
									</span>
								</div>
								<span className="text-xl font-bold text-green-500">
									+{prize.toFixed(2)}{" "}
									{lobby?.tokenSymbol || "STX"}
								</span>
							</div>
						)}

						{/* Wars Points */}
						<div className="flex items-center justify-between rounded-lg border bg-card p-4">
							<div className="flex items-center gap-3">
								<div className="flex size-10 items-center justify-center rounded-full bg-primary/10">
									<Sparkles className="size-5 text-primary" />
								</div>
								<span className="font-medium">Wars Points</span>
							</div>
							<span className="text-xl font-bold text-primary">
								+{warsPoint}
							</span>
						</div>
					</div>
				</div>

				<div className="flex justify-center pt-2">
					{prize != null && prize > 0 ? (
						<Button
							onClick={handleClaim}
							className="w-full flex items-center justify-center"
							disabled={isClaiming}
						>
							{isClaiming ? (
								<>
									<Loader2 className="mr-2 h-4 w-4 animate-spin" />
									Claiming...
								</>
							) : (
								"Claim Reward"
							)}
						</Button>
					) : (
						<Button onClick={handleClose} className="w-full">
							Close
						</Button>
					)}
				</div>
			</DialogContent>
		</Dialog>
	);
}
