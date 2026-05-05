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
}

export function RoomProvider({ children, lobbyPath }: RoomProviderProps) {
	const engineState = useRoomWebSocket({
		lobbyPath,
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
