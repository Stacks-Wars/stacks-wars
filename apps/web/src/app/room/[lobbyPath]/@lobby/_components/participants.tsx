"use client";

import {
	usePlayers,
	useJoinRequests,
	useLobbyActions,
} from "@/lib/stores/room";
import { useLobby } from "@/lib/stores/room";
import { useUser } from "@/lib/stores/user";
import { useRoom } from "@/lib/contexts/room-context";
import Player from "./player";
import { Clock, Users } from "lucide-react";
import { kickPlayerContract } from "@/lib/contract-utils/kick";
import type { AssetString, ContractIdString } from "@stacks/transactions";
import { toast } from "sonner";
import { waitForTxConfirmed } from "@/lib/contract-utils/waitForTxConfirmed";
import { useRef } from "react";

export default function Participants() {
	const players = usePlayers();
	const joinRequests = useJoinRequests();
	const lobby = useLobby();
	const user = useUser();
	const { sendLobbyMessage } = useRoom();
	const pendingActionsRef = useRef<Set<string>>(new Set());
	const lobbyActions = useLobbyActions();

	const isCreator = user?.id === lobby?.creatorId;
	const acceptedPlayers = players.filter((p) => p.state === "accepted");
	const pendingPlayers = joinRequests.filter((jr) => jr.state === "pending");

	const handleApprove = (userId: string) => {
		sendLobbyMessage({ type: "approveJoin", userId });
	};

	const handleReject = (userId: string) => {
		sendLobbyMessage({ type: "rejectJoin", userId });
	};

	const handleKick = async (userId: string, playerAddress: string) => {
		if (!lobby) return;
		if (lobby.contractAddress) {
			try {
				const contract = lobby.contractAddress as ContractIdString;
				let amount = 0;
				if (!lobby.isSponsored && lobby.entryAmount) {
					amount = lobby.entryAmount;
				}

				const tokenId =
					`${lobby.tokenContractId}::${lobby.tokenSymbol}` as AssetString;

				const kickTxId = await kickPlayerContract({
					contract,
					playerAddress,
					amount,
					tokenId,
				});

				if (!kickTxId) {
					toast.error("Failed to kick player in contract", {
						description: "Please try again later.",
					});
					return;
				}
				pendingActionsRef.current.add(`kick-${userId}`);
				lobbyActions.setActionLoading(`kick-${userId}`, true);
				await waitForTxConfirmed(kickTxId);
			} catch (err) {
				console.error("Kick contract failed", err);
				toast.error("Contract transaction failed. Please try again.");
				return;
			}
		}
		sendLobbyMessage({ type: "kick", userId });
	};

	return (
		<div className="space-y-4 rounded-3xl border p-4 sm:space-y-6 sm:p-6 lg:p-8">
			<div className="space-y-3 sm:space-y-4">
				<p className="flex items-center gap-2 text-base font-medium sm:text-lg lg:text-xl">
					<Users className="size-4 sm:size-5" />
					<span>Participants</span>
				</p>
				<div className="space-y-2 sm:space-y-3">
					{acceptedPlayers.map((player) => (
						<Player
							key={player.userId}
							player={player}
							isCreator={isCreator}
							onKick={handleKick}
							kickActionKey={`kick-${player.userId}`}
						/>
					))}
				</div>
			</div>
			{pendingPlayers.length > 0 && (
				<div className="space-y-3 sm:space-y-4">
					<p className="flex items-center gap-2 text-base font-medium sm:text-lg lg:text-xl">
						<Clock className="size-4 sm:size-5" />
						<span>Pending Requests</span>
					</p>
					<div className="space-y-2 sm:space-y-3">
						{pendingPlayers.map((pendingPlayer) => (
							<Player
								key={pendingPlayer.userId}
								player={pendingPlayer}
								isCreator={isCreator}
								onApprove={handleApprove}
								onReject={handleReject}
								approveActionKey={`approve-${pendingPlayer.userId}`}
								rejectActionKey={`reject-${pendingPlayer.userId}`}
							/>
						))}
					</div>
				</div>
			)}
		</div>
	);
}
