import {
	Dialog,
	DialogContent,
	DialogHeader,
	DialogTitle,
	DialogDescription,
	DialogFooter,
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { useCallback, useEffect, useState } from "react";
import { claimFtPoolReward, claimPoolReward } from "@/lib/actions/claimReward";
import { waitForTxConfirmed } from "@/lib/actions/waitForTxConfirmed";
import { toast } from "sonner";
import { apiRequest } from "@/lib/api";
import { formatNumber } from "@/lib/utils";
import { Lobby } from "@/types/schema/lobby";

interface ClaimRewardModalProps {
	showPrizeModal: boolean;
	setShowPrizeModal: (show: boolean) => void;
	setIsClaimed: (claimed: boolean) => void;
	rank: string | null;
	prizeAmount: number | null;
	lobbyId: string;
	warsPoint: number | null;
	lobby: Lobby;
}

export default function ClaimRewardModal({
	showPrizeModal,
	setShowPrizeModal,
	setIsClaimed,
	rank,
	prizeAmount,
	lobbyId,
	warsPoint,
	lobby,
}: ClaimRewardModalProps) {
	const [isLoading, setIsLoading] = useState(false);
	const [countdown, setCountdown] = useState(10);

	// Determine if user has a prize to claim
	const hasPrize = lobby.contractAddress && prizeAmount && prizeAmount > 0;

	const handleOkay = useCallback(() => {
		setIsClaimed(true);
		setShowPrizeModal(false);
	}, [setIsClaimed, setShowPrizeModal]);

	useEffect(() => {
		if (!hasPrize && showPrizeModal) {
			// Reset countdown when modal opens
			setCountdown(10);

			const timer = setInterval(() => {
				setCountdown((prev) => {
					if (prev <= 1) {
						clearInterval(timer);
						handleOkay();
						return 0;
					}
					return prev - 1;
				});
			}, 1000);

			return () => clearInterval(timer);
		}
	}, [hasPrize, showPrizeModal, handleOkay]);

	const handleClaim = async () => {
		try {
			if (!hasPrize) {
				toast.error("Something went wrong", {
					description: "Please contact support.",
				});
				return;
			}
			setIsLoading(true);
			if (!lobby.contractAddress) {
				throw new Error("Contract address is missing");
			}
			const contract = lobby.contractAddress as `${string}.${string}`;
			let claimTxId: string | undefined;
			if (lobby.tokenSymbol !== "STX" && lobby.tokenId) {
				claimTxId = await claimFtPoolReward(
					contract,
					lobby.tokenId,
					prizeAmount
				);
			} else {
				claimTxId = await claimPoolReward(contract, prizeAmount);
			}
			if (!claimTxId) {
				throw new Error(
					"Failed to join game pool: missing transaction ID"
				);
			}

			await waitForTxConfirmed(claimTxId);

			await apiRequest({
				path: `/lobby/${lobbyId}/claim-state`,
				method: "PATCH",
				body: {
					claim: {
						status: "claimed",
						data: {
							tx_id: claimTxId,
						},
					},
				},
			});

			setIsClaimed(true);
			toast.success("Reward claimed successfully!");
			setShowPrizeModal(false);
		} catch (error) {
			console.error("Error claiming reward:", error);
			toast.error("Failed to claim reward. Please try again later.");
		} finally {
			setIsLoading(false);
		}
	};

	return (
		<Dialog open={showPrizeModal} onOpenChange={setShowPrizeModal}>
			<DialogContent
				className="sm:max-w-[400px]"
				hideClose
				disableOutsideClose
			>
				<DialogHeader>
					<DialogTitle className="flex items-center gap-2 justify-center text-xl">
						{hasPrize ? "🏆 Congratulations!" : "🎮 Game Complete!"}
					</DialogTitle>
					<DialogDescription className="text-center space-y-2">
						{rank && (
							<div>
								Your rank: <strong>{rank}</strong>
							</div>
						)}
						{hasPrize && (
							<div className="text-green-600 font-semibold">
								🎉 You won{" "}
								<strong>
									{formatNumber(prizeAmount)}{" "}
									{lobby.tokenSymbol}
								</strong>
								!
							</div>
						)}
						{warsPoint !== null && (
							<div>
								You earned{" "}
								<strong>
									{warsPoint} wars point
									{warsPoint !== 1 ? "s" : ""}
								</strong>
							</div>
						)}
					</DialogDescription>
				</DialogHeader>

				<DialogFooter className="flex flex-col space-y-2">
					{hasPrize ? (
						<Button
							disabled={isLoading}
							onClick={handleClaim}
							className="w-full"
						>
							{isLoading ? "Claiming..." : "Claim Reward"}
						</Button>
					) : (
						<Button onClick={handleOkay} className="w-full">
							Okay ({countdown}s)
						</Button>
					)}
				</DialogFooter>
			</DialogContent>
		</Dialog>
	);
}
