"use client";

import { useRoom } from "@/lib/contexts/room-context";
import { Loader2 } from "lucide-react";

export default function Game() {
	const {
		lobby,
		players,
		gameState,
		sendGameMessage,
		gamePlugin,
		isConnecting,
	} = useRoom();

	// Show loading state while connecting
	if (isConnecting || !lobby) {
		return (
			<div className="flex min-h-screen items-center justify-center">
				<Loader2 className="h-8 w-8 animate-spin" />
				<span className="ml-2">Loading game...</span>
			</div>
		);
	}

	// Check if game plugin is available
	if (!gamePlugin) {
		return (
			<div className="container mx-auto px-4 py-8 text-center">
				<p className="text-destructive">
					Game plugin not found for: {lobby.gamePath}
				</p>
			</div>
		);
	}

	const GameComponent = gamePlugin.GameComponent;
	return (
		<GameComponent
			state={gameState}
			sendMessage={sendGameMessage}
			lobby={lobby}
			players={players}
		/>
	);
}
