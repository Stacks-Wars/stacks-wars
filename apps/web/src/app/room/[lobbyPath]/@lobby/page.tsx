"use client";

import { Button } from "@/components/ui/button";
import GameCard from "@/components/main/game-card";
import Participants from "./_components/participants";
import LobbyDetails from "./_components/lobby-details";
import { cn } from "@/lib/utils";
import Loading from "@/app/loading";
import { useRoom } from "@/lib/contexts/room-context";
import {
	useLobby,
	useGame,
	usePlayers,
	useJoinRequests,
	useRoomConnected,
	useRoomConnecting,
	useIsActionLoading,
	useCountdown,
} from "@/lib/stores/room";
import { useUser, useIsAuthenticated } from "@/lib/stores/user";
import RoomHeader from "@/components/room/room-header";
import {
	joinNormalContract,
	joinSponsoredContract,
} from "@/lib/contract-utils/join";
import {
	leaveNormalContract,
	leaveSponsoredContract,
} from "@/lib/contract-utils/leave";
import type {
	AssetString,
	ContractIdString,
} from "@stacks/connect/dist/types/methods";
import { toast } from "sonner";
import { waitForTxConfirmed } from "@/lib/contract-utils/waitForTxConfirmed";

export default function LobbySlot() {
	const { sendLobbyMessage } = useRoom();

	// Get state from stores
	const lobby = useLobby();
	const game = useGame();
	const players = usePlayers();
	const joinRequests = useJoinRequests();
	const isConnecting = useRoomConnecting();
	const isConnected = useRoomConnected();
	const user = useUser();
	const isAuthenticated = useIsAuthenticated();
	const isStartGameLoading = useIsActionLoading("updateLobbyStatus-starting");
	const isCancelGameLoading = useIsActionLoading("updateLobbyStatus-waiting");
	const countdown = useCountdown();

	if (isConnecting || !lobby || !game) {
		return <Loading />;
	}

	const isCreator = user?.id === lobby.creatorId;
	const isInLobby = players.some((p) => p.userId === user?.id);
	const currentPlayerRequest = joinRequests.find(
		(jr) => jr.userId === user?.id
	);
	const isJoinRequestPending = currentPlayerRequest?.state === "pending";
	const isJoinRequestAccepted = currentPlayerRequest?.state === "accepted";

	const handleJoinOrLeave = async () => {
		if (!lobby || !user) {
			toast.error("You must be logged in to perform this action.");
			return;
		}
		// If leaving
		if (isInLobby) {
			if (lobby.contractAddress) {
				let leaveTxId;
				const contract = lobby.contractAddress as ContractIdString;
				try {
					if (lobby.isSponsored) {
						if (
							lobby.tokenContractId &&
							lobby.tokenSymbol &&
							lobby.currentAmount
						) {
							const tokenId =
								`${lobby.tokenContractId}::${lobby.tokenSymbol}` as AssetString;
							let amount = lobby.entryAmount || 0;
							if (isCreator) {
								amount = lobby.currentAmount;
							}
							leaveTxId = await leaveSponsoredContract({
								contract,
								amount,
								walletAddress: user.walletAddress,
								isCreator,
								tokenId,
							});
						} else {
							toast.error(
								"Cannot leave lobby: missing token information."
							);
							return;
						}
					} else {
						if (!lobby.entryAmount) {
							toast.error(
								"Cannot leave lobby: missing entry amount."
							);
							return;
						}
						leaveTxId = await leaveNormalContract({
							contract,
							amount: lobby.entryAmount,
							walletAddress: user.walletAddress,
						});
					}
					if (!leaveTxId) {
						toast.error("Failed to leave contract", {
							description: "Please try again later.",
						});
						return;
					}
					await waitForTxConfirmed(leaveTxId);
				} catch (err) {
					toast.error(
						"Contract transaction failed. Please try again."
					);
					console.error("Leave contract failed", err);
				}
			}
			sendLobbyMessage({ type: "leave" });
			return;
		}

		// If joining
		if (lobby.isPrivate && !isJoinRequestAccepted) {
			sendLobbyMessage({ type: "joinRequest" });
			return;
		}

		if (lobby.contractAddress) {
			let joinTxId;
			const contract = lobby.contractAddress as ContractIdString;
			try {
				if (lobby.isSponsored) {
					if (
						lobby.tokenContractId &&
						lobby.tokenSymbol &&
						lobby.currentAmount
					) {
						const tokenId =
							`${lobby.tokenContractId}::${lobby.tokenSymbol}` as AssetString;
						let amount = lobby.entryAmount || 0;
						if (isCreator) {
							amount = lobby.currentAmount;
						}
						joinTxId = await joinSponsoredContract({
							contract,
							amount,
							isCreator,
							tokenId,
							address: user.walletAddress,
						});
					} else {
						toast.error(
							"Cannot join lobby: missing token information."
						);
						return;
					}
				} else {
					if (!lobby.entryAmount) {
						toast.error("Cannot join lobby: missing entry amount.");
						return;
					}
					joinTxId = await joinNormalContract({
						contract,
						amount: lobby.entryAmount,
						address: user.walletAddress,
					});
				}
				if (!joinTxId) {
					toast.error("Failed to leave contract", {
						description: "Please try again later.",
					});
					return;
				}
				await waitForTxConfirmed(joinTxId);
			} catch (err) {
				toast.error("Contract transaction failed. Please try again.");
				console.error("Join contract failed", err);
			}
		}
		sendLobbyMessage({ type: "join" });
	};

	const handleStartGame = () => {
		sendLobbyMessage({ type: "updateLobbyStatus", status: "starting" });
	};

	const handleCancelStart = () => {
		sendLobbyMessage({ type: "updateLobbyStatus", status: "waiting" });
	};

	const canStartGame =
		isCreator &&
		lobby.status === "waiting" &&
		players.length >= game.minPlayers;

	return (
		<div className="container mx-auto p-4 pt-0">
			{/* Countdown Overlay */}
			{countdown !== null && countdown > 0 && (
				<div className="fixed inset-0 z-50 flex items-center justify-center bg-background/80 backdrop-blur-sm">
					<div className="flex flex-col items-center gap-6 text-center">
						<p className="text-lg sm:text-xl text-muted-foreground">
							Game starting in
						</p>
						<div className="relative flex items-center justify-center">
							<div className="absolute size-32 sm:size-40 lg:size-48 rounded-full border-4 border-primary/20" />
							<div
								className="absolute size-32 sm:size-40 lg:size-48 rounded-full border-4 border-primary border-t-transparent animate-spin"
								style={{ animationDuration: "1s" }}
							/>
							<span className="text-6xl sm:text-7xl lg:text-8xl font-bold text-primary">
								{countdown}
							</span>
						</div>
						{isCreator && (
							<Button
								variant="outline"
								size="lg"
								onClick={handleCancelStart}
								disabled={isCancelGameLoading}
								className="mt-4"
							>
								{isCancelGameLoading
									? "Cancelling..."
									: "Cancel"}
							</Button>
						)}
					</div>
				</div>
			)}

			<div
				className={cn(
					"space-y-4 sm:space-y-8",
					canStartGame && "mb-15 sm:mb-20 lg:mb-22"
				)}
			>
				<RoomHeader />
				<GameCard
					game={game}
					action="joinLobby"
					onAction={handleJoinOrLeave}
					isInLobby={isInLobby}
					isPrivate={lobby.isPrivate}
					isJoinRequestPending={isJoinRequestPending}
					isJoinRequestAccepted={isJoinRequestAccepted}
					isAuthenticated={isAuthenticated}
				/>
				<LobbyDetails />
				<Participants />
			</div>
			{canStartGame && (
				<div className="fixed bottom-0 left-0 right-0 p-3 sm:p-4 ">
					<div className="container mx-auto pointer-events-auto">
						<Button
							size="lg"
							className="w-full sm:max-w-md mx-auto flex rounded-full text-sm sm:text-base lg:text-xl font-semibold h-11 sm:h-12 lg:h-14"
							onClick={handleStartGame}
							disabled={!isConnected || isStartGameLoading}
						>
							{isStartGameLoading ? "Starting..." : "Start Game"}
						</Button>
					</div>
				</div>
			)}
		</div>
	);
}
