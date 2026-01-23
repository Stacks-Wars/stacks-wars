"use client";

import { Button } from "@/components/ui/button";
import GameCard from "@/components/main/game-card";
import Participants from "./_components/participants";
import LobbyDetails from "./_components/lobby-details";
import { cn } from "@/lib/utils";
import Loading from "@/app/loading";
import { useRoomView } from "@/lib/contexts/room-view-context";
import { useRoom } from "@/lib/contexts/room-context";
import { useIsActionLoading } from "@/lib/stores/room";
import RoomHeader from "./_components/header";

export default function LobbySlot() {
	const {
		lobby,
		game,
		creator,
		players,
		joinRequests,
		chatHistory,
		isConnecting,
		isConnected,
		user,
		isAuthenticated,
		sendLobbyMessage,
		latency,
	} = useRoom();

	// Get loading states from store
	const isStartGameLoading = useIsActionLoading("updateLobbyStatus");

	if (isConnecting || !lobby || !game || !creator) {
		return <Loading />;
	}

	const isCreator = user?.id === lobby.creatorId;
	const isInLobby = players.some((p) => p.userId === user?.id);
	const currentPlayerRequest = joinRequests.find(
		(jr) => jr.userId === user?.id
	);
	const isJoinRequestPending = currentPlayerRequest?.state === "pending";
	const isJoinRequestAccepted = currentPlayerRequest?.state === "accepted";

	const handleJoinOrLeave = () => {
		if (isInLobby) {
			sendLobbyMessage({ type: "leave" });
		} else if (lobby.isPrivate && !isJoinRequestAccepted) {
			sendLobbyMessage({ type: "joinRequest" });
		} else {
			sendLobbyMessage({ type: "join" });
		}
	};

	const handleApproveJoin = (userId: string) => {
		sendLobbyMessage({ type: "approveJoin", userId });
	};

	const handleRejectJoin = (userId: string) => {
		sendLobbyMessage({ type: "rejectJoin", userId });
	};

	const handleKick = (userId: string) => {
		sendLobbyMessage({ type: "kick", userId });
	};

	const handleSendMessage = (content: string) => {
		sendLobbyMessage({ type: "sendMessage", content });
	};

	const handleAddReaction = (messageId: string, emoji: string) => {
		console.log("addReaction called");
		sendLobbyMessage({ type: "addReaction", messageId, emoji });
	};

	const handleRemoveReaction = (messageId: string, emoji: string) => {
		console.log("removeReaction called");
		sendLobbyMessage({ type: "removeReaction", messageId, emoji });
	};

	const handleStartGame = () => {
		sendLobbyMessage({ type: "updateLobbyStatus", status: "starting" });
	};

	const canStartGame =
		isCreator &&
		lobby.status === "waiting" &&
		players.length >= game.minPlayers;
	//const canStartGame = true;

	const acceptedPlayers = players.filter((p) => p.state === "accepted");
	const pendingPlayers = joinRequests.filter((jr) => jr.state === "pending");

	return (
		<div className="container mx-auto p-4">
			<div
				className={cn(
					"space-y-4 sm:space-y-8",
					canStartGame && "mb-20 sm:mb-24"
				)}
			>
				<RoomHeader
					lobby={lobby}
					isConnected={isConnected}
					isConnecting={isConnecting}
					latency={latency}
				/>
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
				<LobbyDetails
					lobby={lobby}
					game={game}
					players={players}
					chatHistory={chatHistory}
					currentUserId={user?.id}
					onSendMessage={handleSendMessage}
					onAddReaction={handleAddReaction}
					onRemoveReaction={handleRemoveReaction}
				/>
				<Participants
					players={acceptedPlayers}
					pendingPlayers={pendingPlayers}
					isCreator={isCreator}
					onApprove={handleApproveJoin}
					onReject={handleRejectJoin}
					onKick={handleKick}
				/>
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
