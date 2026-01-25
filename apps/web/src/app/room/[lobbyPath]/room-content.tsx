"use client";

import { useState } from "react";
import { RoomProvider } from "@/lib/contexts/room-context";
import { RoomViewProvider } from "@/lib/contexts/room-view-context";
import { toast } from "sonner";
import { useRouter } from "next/navigation";
import { useLobby } from "@/lib/stores/room";

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
	const router = useRouter();

	const handleActionSuccess = (
		action: string,
		message?: string,
		disconnect?: () => void
	) => {
		if (action === "lobbyDeleted") {
			// Disconnect WebSocket before redirect to prevent reconnection attempts
			disconnect?.();
			toast.error(message || "Lobby has been closed");
			router.replace("/lobby");
			return;
		}

		if (message) {
			toast.success(message);
		}
	};

	const handleActionError = (
		action: string,
		error: { code: string; message: string }
	) => {
		toast.error(error.message || `Action failed: ${action}`);
	};

	return (
		<RoomProvider
			lobbyPath={lobbyPath}
			onActionSuccess={handleActionSuccess}
			onActionError={handleActionError}
		>
			<RoomContentInner lobby={lobby} game={game} />
			{children}
		</RoomProvider>
	);
}
