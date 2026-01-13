"use client";

import { createContext, useContext } from "react";

interface RoomViewContextValue {
	currentView: "lobby" | "game";
	setView: (view: "lobby" | "game" | null) => void;
}

const RoomViewContext = createContext<RoomViewContextValue | null>(null);

export function RoomViewProvider({
	children,
	value,
}: {
	children: React.ReactNode;
	value: RoomViewContextValue;
}) {
	return (
		<RoomViewContext.Provider value={value}>
			{children}
		</RoomViewContext.Provider>
	);
}

export function useRoomView() {
	const context = useContext(RoomViewContext);
	if (!context) {
		throw new Error("useRoomView must be used within RoomViewProvider");
	}
	return context;
}
