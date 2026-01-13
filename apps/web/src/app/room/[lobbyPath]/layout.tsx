"use client";

import { useState } from "react";
import { RoomProvider, useRoom } from "@/lib/contexts/room-context";
import { RoomViewProvider } from "@/lib/contexts/room-view-context";

function RoomContent({
	lobby,
	game,
}: {
	lobby: React.ReactNode;
	game: React.ReactNode;
}) {
	const { lobby: lobbyData } = useRoom();
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

export default async function RoomLayout({
	children,
	lobby,
	game,
	params,
}: {
	children: React.ReactNode;
	lobby: React.ReactNode;
	game: React.ReactNode;
	params: Promise<{ lobbyPath: string }>;
}) {
	const lobbyPath = (await params).lobbyPath;

	return (
		<RoomProvider lobbyPath={lobbyPath}>
			<RoomContent lobby={lobby} game={game} />
			{children}
		</RoomProvider>
	);
}
