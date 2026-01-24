"use client";

import { useRoom } from "@/lib/contexts/room-context";
import Loading from "@/app/loading";
import { useGame, useLobby, useRoomConnecting } from "@/lib/stores/room";

export default function GameSlot() {
	const { gameState, gamePlugin, sendGameMessage } = useRoom();
	const lobby = useLobby();
	const game = useGame();
	const isConnecting = useRoomConnecting();

	if (isConnecting || !lobby || !game) {
		return <Loading />;
	}

	return (
		<div className="container mx-auto px-4">
			{gamePlugin ? (
				<gamePlugin.GameComponent
					state={gameState}
					sendMessage={sendGameMessage}
					lobby={lobby}
					game={game}
				/>
			) : (
				<div className="bg-card rounded-lg p-8 text-center">
					<p className="text-muted-foreground text-lg">
						Game component is missing for {lobby.gamePath}
					</p>
					<p className="text-sm text-muted-foreground mt-2">
						Please ensure the game plugin is properly registered if
						you're the dev.
					</p>
				</div>
			)}
		</div>
	);
}
