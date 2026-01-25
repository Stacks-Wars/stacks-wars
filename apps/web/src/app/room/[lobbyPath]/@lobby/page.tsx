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

	const handleJoinOrLeave = () => {
		if (isInLobby) {
			sendLobbyMessage({ type: "leave" });
		} else if (lobby.isPrivate && !isJoinRequestAccepted) {
			sendLobbyMessage({ type: "joinRequest" });
		} else {
			sendLobbyMessage({ type: "join" });
		}
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
