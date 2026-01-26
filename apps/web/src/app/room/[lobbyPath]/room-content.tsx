"use client";

import { useState } from "react";
import { RoomProvider } from "@/lib/contexts/room-context";
import { RoomViewProvider } from "@/lib/contexts/room-view-context";
import { useLobby } from "@/lib/stores/room";
import GameOverModal from "@/components/room/game-over-modal";
import FinalStandingsModal from "@/components/room/final-standings-modal";

function RoomContentInner({
	lobby,
	game,
}: {
	lobby: React.ReactNode;
	game: React.ReactNode;
}) {
	const lobbyData = useLobby();
	const [manualView, setManualView] = useState<"lobby" | "game" | null>(null);

	// Determine automatic view based on lobby status
	const autoView =
		!lobbyData ||
		lobbyData.status === "waiting" ||
		lobbyData.status === "starting"
			? "lobby"
			: "game";

	// Use manual view if set, otherwise use automatic view
	const currentView = manualView || autoView;

	// Reset manual view when status changes to ensure proper flow
	// (e.g., when game starts, show game even if user was viewing lobby)
	const showLobby = currentView === "lobby";

	return (
		<RoomViewProvider value={{ currentView, setView: setManualView }}>
			<div>{showLobby ? lobby : game}</div>
		</RoomViewProvider>
	);
}

export default function RoomContent({
	lobby,
	game,
	lobbyPath,
	children,
}: {
	lobby: React.ReactNode;
	game: React.ReactNode;
	lobbyPath: string;
	children?: React.ReactNode;
}) {
	return (
		<RoomProvider lobbyPath={lobbyPath}>
			<RoomContentInner lobby={lobby} game={game} />
			{children}
			<GameOverModal />
			<FinalStandingsModal />
		</RoomProvider>
	);
}
