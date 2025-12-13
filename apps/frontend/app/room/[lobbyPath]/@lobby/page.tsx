"use client";

import { Lobby } from "./_components/lobby";
import { Loader2 } from "lucide-react";
import { useRoom } from "@/lib/contexts/room-context";

export default function LobbySlot() {
	const {
		lobby,
		players,
		joinRequests,
		chatHistory,
		sendLobbyMessage,
		isConnecting,
	} = useRoom();

	// Show loading state while connecting
	if (isConnecting || !lobby) {
		return (
			<div className="flex min-h-screen items-center justify-center">
				<Loader2 className="h-8 w-8 animate-spin" />
				<span className="ml-2">Connecting...</span>
			</div>
		);
	}

	// Wrap sendLobbyMessage to match Lobby's expected signature
	const handleSendMessage = (type: string, payload?: unknown) => {
		sendLobbyMessage(type, payload);
	};

	return (
		<Lobby
			lobby={lobby}
			players={players}
			joinRequests={joinRequests}
			chatHistory={chatHistory}
			onSendMessage={handleSendMessage}
		/>
	);
}
