"use client";

import { createContext, useContext, ReactNode } from "react";
import { useRoomWebSocket } from "@/lib/hooks/useRoomWebSocket";
import { RoomClientMessage } from "@/lib/websocket/roomClient";
import {
	ChatMessage,
	JoinRequest,
	LobbyExtended,
	PlayerState,
} from "@/lib/definitions";

interface RoomContextValue {
	lobby: LobbyExtended | null;
	players: PlayerState[];
	joinRequests: JoinRequest[];
	chatHistory: ChatMessage[];
	isConnected: boolean;
	isConnecting: boolean;
	error: string | null;
	sendMessage: (message: RoomClientMessage) => void;
}

const RoomContext = createContext<RoomContextValue | null>(null);

export function RoomProvider({
	lobbyPath,
	children,
}: {
	lobbyPath: string;
	children: ReactNode;
}) {
	const roomState = useRoomWebSocket({
		lobbyPath,
		onError: (error) => {
			console.error("[Room] WebSocket error:", error);
		},
		onClose: () => {
			console.log("[Room] WebSocket closed");
		},
	});

	return (
		<RoomContext.Provider value={roomState}>
			{children}
		</RoomContext.Provider>
	);
}

export function useRoom() {
	const context = useContext(RoomContext);
	if (!context) {
		throw new Error("useRoom must be used within RoomProvider");
	}
	return context;
}
