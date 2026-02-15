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
	useLobbyActions,
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
import type { AssetString, ContractIdString } from "@stacks/transactions";
import { toast } from "sonner";
import {
	ExpectedError,
	waitForTxConfirmed,
} from "@/lib/contract-utils/waitForTxConfirmed";
import StartCountdown from "./_components/start-countdown";
import SponsorParticipation from "./_components/sponsor-participation";
import { useRef } from "react";

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
	const pendingActionsRef = useRef<Set<string>>(new Set());
	const lobbyActions = useLobbyActions();

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
				const leaveKey = `leave-${user.id}`;
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
					pendingActionsRef.current.add(leaveKey);
					lobbyActions.setActionLoading(leaveKey, true);
					await waitForTxConfirmed(
						leaveTxId,
						ExpectedError.ERR_NOT_JOINED
					);
				} catch (err) {
					toast.error(
						"Contract transaction failed. Please try again."
					);
					console.error("Leave contract failed", err);
					if (pendingActionsRef.current.has(leaveKey)) {
						pendingActionsRef.current.delete(leaveKey);
						lobbyActions.clearActionLoading(leaveKey);
					}
					return;
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
			const joinKey = `join-${user?.id}`;
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
				pendingActionsRef.current.add(joinKey);
				lobbyActions.setActionLoading(joinKey, true);
				await waitForTxConfirmed(
					joinTxId,
					ExpectedError.ERR_ALREADY_JOINED
				);
			} catch (err) {
				toast.error("Contract transaction failed. Please try again.");
				console.error("Join contract failed", err);
				if (pendingActionsRef.current.has(joinKey)) {
					pendingActionsRef.current.delete(joinKey);
					lobbyActions.clearActionLoading(joinKey);
				}
				return;
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

	// Count active participants (exclude spectating sponsored creator)
	const activePlayerCount = players.filter((p) => {
		if (
			lobby.isSponsored &&
			p.userId === lobby.creatorId &&
			p.status === "notJoined"
		)
			return false;
		return true;
	}).length;

	const canStartGame =
		isCreator &&
		lobby.status === "waiting" &&
		activePlayerCount >= game.minPlayers;

	return (
		<div className="container mx-auto p-4 pt-0">
			{/* Countdown Overlay */}
			<StartCountdown
				isCreator={isCreator}
				handleCancelStart={handleCancelStart}
			/>

			<div
				className={cn(
					"space-y-4 sm:space-y-8",
					canStartGame && "mb-15 sm:mb-20 lg:mb-22"
				)}
			>
				<RoomHeader slot="lobby" />
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
				<SponsorParticipation />
				<Participants />
			</div>
			{canStartGame && (
				<div className="fixed right-0 bottom-0 left-0 p-3 sm:p-4">
					<div className="pointer-events-auto container mx-auto">
						<Button
							size="lg"
							className="mx-auto flex h-11 w-full rounded-full text-sm font-semibold sm:h-12 sm:max-w-md sm:text-base lg:h-14 lg:text-xl"
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
