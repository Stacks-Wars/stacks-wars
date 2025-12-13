"use client";

import { createContext, useContext } from "react";
import { useRoomWebSocket } from "@/lib/hooks/useRoomWebSocket";
import {
	LobbyExtended,
	PlayerState,
	JoinRequest,
	ChatMessage,
	GamePlugin,
} from "@/lib/definitions";

interface RoomContextValue {
	isConnected: boolean;
	isConnecting: boolean;
	error: string | null;
	lobby: LobbyExtended | null;
	players: PlayerState[];
	joinRequests: JoinRequest[];
	chatHistory: ChatMessage[];
	gameState: unknown;
	gamePlugin: GamePlugin | undefined;
	sendGameMessage: (type: string, payload: unknown) => void;
	sendLobbyMessage: (type: string, payload?: unknown) => void;
}

const RoomContext = createContext<RoomContextValue | null>(null);

export function RoomProvider({
	children,
	lobbyPath,
}: {
	children: React.ReactNode;
	lobbyPath: string;
}) {
	const engineState = useRoomWebSocket({ lobbyPath });

	return (
		<RoomContext.Provider value={engineState}>
			{children}
		</RoomContext.Provider>
	);
}

export function useRoom() {
	const context = useContext(RoomContext);
	if (!context) {
		throw new Error(
			"useGameEngineContext must be used within GameEngineProvider"
		);
	}
	return context;
}
