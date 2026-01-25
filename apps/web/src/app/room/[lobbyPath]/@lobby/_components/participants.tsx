"use client";

import { usePlayers, useJoinRequests } from "@/lib/stores/room";
import { useLobby } from "@/lib/stores/room";
import { useUser } from "@/lib/stores/user";
import { useRoom } from "@/lib/contexts/room-context";
import Player from "./player";
import { Clock, Users } from "lucide-react";

export default function Participants() {
	const players = usePlayers();
	const joinRequests = useJoinRequests();
	const lobby = useLobby();
	const user = useUser();
	const { sendLobbyMessage } = useRoom();

	const isCreator = user?.id === lobby?.creatorId;
	const acceptedPlayers = players.filter((p) => p.state === "accepted");
	const pendingPlayers = joinRequests.filter((jr) => jr.state === "pending");

	const handleApprove = (userId: string) => {
		sendLobbyMessage({ type: "approveJoin", userId });
	};

	const handleReject = (userId: string) => {
		sendLobbyMessage({ type: "rejectJoin", userId });
	};

	const handleKick = (userId: string) => {
		sendLobbyMessage({ type: "kick", userId });
	};

	return (
		<div className="border rounded-3xl p-4 sm:p-6 lg:p-8 space-y-4 sm:space-y-6">
			<div className="space-y-3 sm:space-y-4">
				<p className="text-base sm:text-lg lg:text-xl font-medium flex items-center gap-2">
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
					<p className="text-base sm:text-lg lg:text-xl font-medium flex items-center gap-2">
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
