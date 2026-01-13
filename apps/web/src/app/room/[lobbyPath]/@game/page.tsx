"use client";

import { useRoom } from "@/lib/contexts/room-context";
import { Button } from "@/components/ui/button";
import { Users } from "lucide-react";
import Loading from "@/app/loading";
import { useRoomView } from "@/lib/contexts/room-view-context";

export default function GameSlot() {
	const { lobby, game, creator, gameState, gamePlugin, isConnecting } = useRoom();
	const { setView } = useRoomView();

	if (isConnecting || !lobby || !game || !creator) {
		return <Loading />;
	}

	return (
		<div className="container mx-auto px-4">
			<div className="space-y-4 sm:space-y-8">
				<div className="flex items-center justify-between">
					<h1 className="text-xl sm:text-2xl font-bold truncate">
						{lobby.name}
					</h1>
					<Button
						variant="outline"
						size="sm"
						className="gap-2 shrink-0"
						onClick={() => setView("lobby")}
					>
						<Users className="size-4" />
						<span className="hidden sm:inline">View </span>Lobby
					</Button>
				</div>

				{gamePlugin ? (
					<div>
						{/* Render game plugin's component */}
						<gamePlugin.GameComponent
							state={gameState}
							sendMessage={() => {}}
							lobby={lobby}
							game={game}
							creator={creator}
							players={[]}
						/>
					</div>
				) : (
					<div className="bg-card rounded-lg p-8 text-center">
						<p className="text-muted-foreground text-lg">
							Game component is missing for {lobby.gamePath}
						</p>
						<p className="text-sm text-muted-foreground mt-2">
							Please ensure the game plugin is properly
							registered.
						</p>
					</div>
				)}
			</div>
		</div>
	);
}
