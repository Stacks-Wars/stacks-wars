"use client";

import { createContext, useContext } from "react";
import {
	useRoomWebSocket,
	type UseRoomWebSocketReturn,
} from "@/lib/hooks/useRoomWebSocket";

const RoomContext = createContext<UseRoomWebSocketReturn | null>(null);

interface RoomProviderProps {
	children: React.ReactNode;
	lobbyPath: string;
	onActionSuccess?: (action: string, message?: string) => void;
	onActionError?: (action: string, error: { code: string; message: string }) => void;
}

export function RoomProvider({ 
	children, 
	lobbyPath,
	onActionSuccess,
	onActionError,
}: RoomProviderProps) {
	const engineState = useRoomWebSocket({ 
		lobbyPath,
		onActionSuccess,
		onActionError,
	});

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
