"use client";

import { useRoom } from "@/lib/contexts/room-context";
import { Button } from "@/components/ui/button";
import { ChevronLeft, Gamepad2 } from "lucide-react";
import Link from "next/link";
import ShareButton from "./_components/share-button";
import GameCard from "@/components/main/game-card";
import Participants from "./_components/participants";
import LobbyDetails from "./_components/lobby-details";
import { cn } from "@/lib/utils";
import { useUser } from "@/lib/stores/user";
import Loading from "@/app/loading";
import { useRoomView } from "@/lib/contexts/room-view-context";

export default function LobbySlot() {
	const {
		lobby,
		game,
		creator,
		players,
		joinRequests,
		isConnecting,
		isConnected,
		sendLobbyMessage,
	} = useRoom();
	const { setView } = useRoomView();
	const user = useUser();

	if (isConnecting || !lobby || !game || !creator) {
		return <Loading />;
	}

	const isCreator = user?.id === lobby.creatorId;
	//const canStartGame =
	//	isCreator &&
	//	lobby.status === "waiting" &&
	//	players.length >= game.minPlayers;
	const canStartGame = true;

	console.log("players:", players);
	const acceptedPlayers = players.filter((p) => p.state === "accepted");
	const pendingPlayers = joinRequests.filter((jr) => jr.state === "pending");

	return (
		<div className="container mx-auto px-4">
			<div
				className={cn(
					"space-y-4 sm:space-y-8",
					canStartGame && "mb-20 sm:mb-24"
				)}
			>
				<div className="flex items-center justify-between">
					<Button
						asChild
						variant={"link"}
						className="has-[>svg]:px-0 px-0 py-2.5"
					>
						<Link href={"/lobby"}>
							<ChevronLeft />
							Back to Lobby
						</Link>
					</Button>
					<div className="flex items-center gap-2 sm:gap-4">
						{(lobby.status === "inProgress" ||
							lobby.status === "finished") && (
							<Button
								variant="outline"
								size="sm"
								className="gap-2 shrink-0"
								onClick={() => setView("game")}
							>
								<Gamepad2 className="size-4" />
								<span className="hidden sm:inline">View </span>
								Game
							</Button>
						)}
						<ShareButton lobbyPath={lobby.path} />
					</div>
				</div>
				<GameCard game={game} />
				<LobbyDetails lobby={lobby} game={game} />
				<Participants
					players={acceptedPlayers}
					pendingPlayers={pendingPlayers}
					isCreator={isCreator}
				/>
			</div>
			{canStartGame && (
				<div className="fixed bottom-0 left-0 right-0 p-3 sm:p-4 bg-linear-to-t from-background via-background to-transparent pointer-events-none">
					<div className="container mx-auto pointer-events-auto">
						<Button
							size="lg"
							className="w-full sm:max-w-md mx-auto flex rounded-full text-sm sm:text-base lg:text-xl font-semibold h-11 sm:h-12 lg:h-14"
							onClick={() => {
								sendLobbyMessage("startGame");
							}}
							disabled={!isConnected}
						>
							Start Game
						</Button>
					</div>
				</div>
			)}
		</div>
	);
}
