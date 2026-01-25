"use client";

import ChatDialog from "@/components/room/chat";
import { useLobby, useGame } from "@/lib/stores/room";

export default function LobbyDetails() {
	const lobby = useLobby();
	const game = useGame();

	if (!lobby || !game) return null;

	return (
		<div className="flex flex-col items-center w-full">
			<div className="bg-card border rounded-3xl p-4 sm:p-6 lg:p-8 space-y-4 sm:space-y-6 w-full">
				<div className="space-y-2">
					<h2 className="text-lg sm:text-xl lg:text-2xl font-semibold">
						{lobby.name}
					</h2>
					{lobby.description && (
						<p className="text-xs sm:text-sm">
							{lobby.description}
						</p>
					)}
				</div>

				<div className="grid grid-cols-2 md:grid-cols-4 gap-3 sm:gap-4">
					<div className="space-y-0.5 sm:space-y-1">
						<p className="text-xs sm:text-sm text-muted-foreground">
							Entry Amount
						</p>
						<p className="text-base sm:text-lg lg:text-xl font-medium truncate">
							{lobby.entryAmount ? lobby.entryAmount : 0}{" "}
							{lobby.tokenSymbol || "STX"}
						</p>
					</div>

					<div className="space-y-0.5 sm:space-y-1">
						<p className="text-xs sm:text-sm text-muted-foreground">
							Prize Pool
						</p>
						<p className="text-base sm:text-lg lg:text-xl font-medium truncate">
							{lobby.currentAmount ? lobby.currentAmount : 0}{" "}
							{lobby.tokenSymbol || "STX"}
						</p>
					</div>

					<div className="space-y-0.5 sm:space-y-1">
						<p className="text-xs sm:text-sm text-muted-foreground">
							Players
						</p>
						<p className="text-base sm:text-lg lg:text-xl font-medium">
							{lobby.participantCount}/{game.maxPlayers}
						</p>
					</div>

					<div className="space-y-0.5 sm:space-y-1">
						<p className="text-xs sm:text-sm text-muted-foreground">
							Status
						</p>
						<p className="text-base sm:text-lg lg:text-xl font-medium capitalize">
							{lobby.status === "inProgress"
								? "In Progress"
								: lobby.status}
						</p>
					</div>
				</div>
			</div>
			<ChatDialog
				buttonVariant="default"
				buttonClassName="-translate-y-1/2 size-10 sm:size-12 lg:size-16 -mb-5 sm:-mb-6 lg:-mb-8"
			/>
		</div>
	);
}
