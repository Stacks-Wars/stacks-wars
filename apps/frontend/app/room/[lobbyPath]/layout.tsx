"use client";

import { useParams } from "next/navigation";
import { RoomProvider, useRoom } from "@/lib/contexts/room-context";

/**
 * Room Layout with Parallel Routes
 *
 * Conditionally renders @lobby or @game slots based on lobby.status from WebSocket
 */
function RoomContent({
	children,
	lobby,
	game,
}: {
	children: React.ReactNode;
	lobby: React.ReactNode;
	game: React.ReactNode;
}) {
	const { lobby: lobbyData } = useRoom();

	return (
		<div className="flex min-h-screen flex-col">
			{/* Conditionally render slots based on lobby status */}
			{(!lobbyData || lobbyData.status === "waiting") && lobby}
			{lobbyData?.status === "inProgress" && game}
			{lobbyData?.status === "finished" && (
				<div className="container mx-auto px-4 py-8 text-center">
					<h2 className="text-2xl font-bold">Game Completed!</h2>
					<p className="mt-2 text-muted-foreground">
						Thanks for playing!
					</p>
				</div>
			)}
			{children}
		</div>
	);
}

export default function RoomLayout({
	children,
	lobby,
	game,
}: {
	children: React.ReactNode;
	lobby: React.ReactNode;
	game: React.ReactNode;
}) {
	const params = useParams();
	const lobbyPath = params.lobbyPath as string;

	return (
		<RoomProvider lobbyPath={lobbyPath}>
			<RoomContent lobby={lobby} game={game}>
				{children}
			</RoomContent>
		</RoomProvider>
	);
}
