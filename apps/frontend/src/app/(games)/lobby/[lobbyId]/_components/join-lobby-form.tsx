import { useEffect, useState } from "react";
import { Button } from "@/components/ui/button";
import {
	Card,
	CardHeader,
	CardTitle,
	CardDescription,
	CardFooter,
} from "@/components/ui/card";
import {
	Dialog,
	DialogContent,
	DialogDescription,
	DialogFooter,
	DialogHeader,
	DialogTitle,
} from "@/components/ui/dialog";
import { Loader2, AlertTriangle } from "lucide-react";
import { toast } from "sonner";
import { LobbyClientMessage } from "@/hooks/useLobbySocket";
import {
	joinGamePool,
	joinSponsoredGamePool,
} from "@/lib/actions/joinGamePool";
import { waitForTxConfirmed } from "@/lib/actions/waitForTxConfirmed";
import {
	leaveGamePool,
	leaveSponsoredFtGamePool,
	leaveSponsoredGamePool,
} from "@/lib/actions/leaveGamePool";
import { useRouter } from "next/navigation";
import { JoinState, Lobby } from "@/types/schema/lobby";
import { Player } from "@/types/schema/player";

interface JoinLobbyFormProps {
	lobby: Lobby;
	players: Player[];
	joinState: JoinState | null;
	lobbyId: string;
	userId: string;
	sendMessage: (msg: LobbyClientMessage) => Promise<void>;
	disconnect: () => void;
	chatDisconnect: () => void;
	onLeaveCheck?: (callback: (isConnected: boolean) => void) => void;
	cachedPlayerConnectionStatus?: boolean | null;
}

export default function JoinLobbyForm({
	lobby,
	players,
	joinState,
	userId,
	sendMessage,
	disconnect,
	chatDisconnect,
	onLeaveCheck,
	cachedPlayerConnectionStatus,
}: JoinLobbyFormProps) {
	const [joined, setJoined] = useState(false);
	const [loading, setLoading] = useState(false);
	const [showLeaveConfirmation, setShowLeaveConfirmation] = useState(false);
	const [pendingLeaveCheck, setPendingLeaveCheck] = useState(false);
	const isParticipant = players.some((p) => p.id === userId);
	const isCreator = userId === lobby.creator.id;

	const router = useRouter();

	useEffect(() => {
		if (isParticipant) setJoined(true);
		else setJoined(false);
	}, [isParticipant]);

	const handleLeave = async () => {
		setLoading(true);
		setPendingLeaveCheck(true);

		try {
			if (isCreator && players.length > 1) {
				toast.error("You can't leave the lobby with participants", {
					description:
						"Please remove all participants before leaving.",
				});
				return;
			}

			if (cachedPlayerConnectionStatus !== null) {
				if (cachedPlayerConnectionStatus) {
					toast.error("Cannot leave lobby", {
						description:
							"You have already played in this lobby and cannot withdraw from the pool.",
					});
					setLoading(false);
					setPendingLeaveCheck(false);
					setShowLeaveConfirmation(false);
					return;
				}

				await proceedWithLeave();
				return;
			}

			await sendMessage({ type: "requestLeave" });

			if (onLeaveCheck) {
				onLeaveCheck(async (isConnected: boolean) => {
					if (isConnected) {
						toast.error("Cannot leave lobby", {
							description:
								"You have already played in this lobby and cannot withdraw from the pool.",
						});
						setLoading(false);
						setPendingLeaveCheck(false);
						setShowLeaveConfirmation(false);
						return;
					}

					await proceedWithLeave();
				});
			} else {
				await proceedWithLeave();
			}
		} catch (error) {
			console.error("An error occurred:", error);
			toast.error("Failed to leave. Please try again.");
			setLoading(false);
			setPendingLeaveCheck(false);
			setShowLeaveConfirmation(false);
		}
	};

	const proceedWithLeave = async () => {
		try {
			if (lobby.contractAddress && lobby.entryAmount !== null) {
				const contract = lobby.contractAddress as `${string}.${string}`;
				let leaveTxId: string | undefined;
				if (lobby.entryAmount === 0 && lobby.currentAmount) {
					if (lobby.tokenSymbol === "STX") {
						if (isCreator) {
							leaveTxId = await leaveSponsoredGamePool(
								contract,
								isCreator,
								lobby.currentAmount
							);
						} else {
							leaveTxId = await leaveSponsoredGamePool(
								contract,
								isCreator,
								lobby.entryAmount
							);
						}
					} else if (lobby.tokenId) {
						if (isCreator) {
							leaveTxId = await leaveSponsoredFtGamePool(
								contract,
								lobby.tokenId,
								isCreator,
								lobby.currentAmount
							);
						} else {
							leaveTxId = await leaveSponsoredFtGamePool(
								contract,
								lobby.tokenId,
								isCreator,
								lobby.entryAmount
							);
						}
					}
				} else {
					leaveTxId = await leaveGamePool(
						contract,
						lobby.entryAmount
					);
				}
				if (!leaveTxId) {
					throw new Error(
						"Failed to leave game pool: missing transaction ID"
					);
				}

				await waitForTxConfirmed(leaveTxId);
			}
			await sendMessage({ type: "leaveLobby" });
			setJoined(false);
			if (isCreator) {
				disconnect();
				chatDisconnect();
				router.replace(`/lobby`);
			}
		} catch (error) {
			console.error("An error occurred:", error);
		} finally {
			setLoading(false);
			setPendingLeaveCheck(false);
			setShowLeaveConfirmation(false);
		}
	};

	const handleClick = async () => {
		setLoading(true);

		try {
			if (joined) {
				// Show confirmation dialog before leaving
				setLoading(false);
				setShowLeaveConfirmation(true);
				return;
			}

			if (joinState === "pending") return;

			if (joinState === null || joinState === "rejected") {
				await sendMessage({ type: "requestJoin" });
				return;
			}

			if (joinState === "allowed") {
				if (lobby.contractAddress && lobby.entryAmount !== null) {
					const contract =
						lobby.contractAddress as `${string}.${string}`;
					const amount = lobby.entryAmount;
					let joinTxId: string | undefined;

					if (lobby.entryAmount === 0 && lobby.currentAmount) {
						joinTxId = await joinSponsoredGamePool(
							contract,
							isCreator,
							lobby.currentAmount
						);
					} else {
						joinTxId = await joinGamePool(contract, amount);
					}
					if (!joinTxId) {
						throw new Error(
							"Failed to join game pool: missing transaction ID"
						);
					}

					await waitForTxConfirmed(joinTxId);

					await sendMessage({
						type: "joinLobby",
						txId: joinTxId,
					});
				} else {
					await sendMessage({ type: "joinLobby", txId: undefined });
				}
			}
		} catch (error) {
			console.error("An error occured:", error);
			toast.error("Failed. Please try again.");
		} finally {
			setLoading(false);
		}
	};

	const getButtonText = () => {
		if (loading || joinState === "pending") {
			if (pendingLeaveCheck) {
				return "Checking...";
			}
			return joined ? "Leaving..." : "Processing...";
		}

		if (joined) {
			return isCreator ? "Leave and Delete Lobby" : "Leave Lobby";
		}

		switch (joinState) {
			case "allowed":
				return "Join Lobby";
			case "rejected":
				return "Request to Join";
			case null:
				return "Request to Join";
			default:
				return "Unknown State";
		}
	};

	return (
		<>
			<Card className="bg-primary/10 overflow-hidden">
				<CardHeader className="pb-3">
					<CardTitle className="truncate">
						{joined ? "Leave Lobby" : "Join Lobby"}
					</CardTitle>
					<CardDescription className="wrap-break-word">
						{joined
							? "You're currently in this lobby"
							: "Join this lobby to participate in the game"}
					</CardDescription>
				</CardHeader>
				<CardFooter className="pt-3">
					<Button
						className="w-full"
						size="lg"
						variant={joined ? "destructive" : "default"}
						onClick={handleClick}
						disabled={
							loading || (!joined && joinState === "pending")
						}
					>
						{(loading || joinState === "pending") && (
							<Loader2 className="mr-2 h-4 w-4 shrink-0 animate-spin" />
						)}
						<span className="truncate">{getButtonText()}</span>
					</Button>
				</CardFooter>
			</Card>

			<Dialog
				open={showLeaveConfirmation}
				onOpenChange={setShowLeaveConfirmation}
			>
				<DialogContent className="sm:max-w-[425px]">
					<DialogHeader>
						<DialogTitle className="text-destructive flex items-center gap-2">
							<AlertTriangle className="h-5 w-5" />
							{isCreator
								? "Confirm Delete Lobby"
								: "Confirm Leave Lobby"}
						</DialogTitle>
						<DialogDescription className="text-left">
							{isCreator ? (
								<>
									⚠️ Continuing will{" "}
									<strong>
										permanently delete this lobby
									</strong>
									. This action cannot be undone. Are you sure
									you want to continue?
								</>
							) : (
								<>
									⚠️ You will lose{" "}
									<strong>10 Wars Points</strong> for leaving
									this lobby. Are you sure you want to
									continue?
								</>
							)}
						</DialogDescription>
					</DialogHeader>
					<DialogFooter className="gap-2">
						<Button
							variant="outline"
							onClick={() => setShowLeaveConfirmation(false)}
							disabled={loading}
						>
							Cancel
						</Button>
						<Button
							variant="destructive"
							onClick={handleLeave}
							disabled={loading}
						>
							{loading && (
								<Loader2 className="mr-2 h-4 w-4 animate-spin" />
							)}
							{isCreator ? "Delete Lobby" : "Continue & Leave"}
						</Button>
					</DialogFooter>
				</DialogContent>
			</Dialog>
		</>
	);
}
