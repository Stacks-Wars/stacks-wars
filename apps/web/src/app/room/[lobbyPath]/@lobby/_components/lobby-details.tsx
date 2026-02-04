"use client";

import ChatDialog from "@/components/room/chat";
import { useLobby, useGame } from "@/lib/stores/room";

export default function LobbyDetails() {
	const lobby = useLobby();
	const game = useGame();

	if (!lobby || !game) return null;

	return (
		<div className="flex w-full flex-col items-center">
			<div className="w-full space-y-4 rounded-3xl border p-4 sm:space-y-6 sm:p-6 lg:p-8">
				<div className="space-y-2">
					<h2 className="text-lg font-semibold sm:text-xl lg:text-2xl">
						{lobby.name}
					</h2>
					{lobby.description && (
						<p className="text-xs sm:text-sm">
							{lobby.description}
						</p>
					)}
				</div>

				<div className="grid grid-cols-2 gap-3 sm:gap-4 md:grid-cols-4">
					<div className="space-y-0.5 sm:space-y-1">
						<p className="text-muted-foreground text-xs sm:text-sm">
							Entry Amount
						</p>
						<p className="truncate text-base font-medium sm:text-lg lg:text-xl">
							{lobby.entryAmount ? lobby.entryAmount : 0}{" "}
							{lobby.tokenSymbol || "STX"}
						</p>
					</div>

					<div className="space-y-0.5 sm:space-y-1">
						<p className="text-muted-foreground text-xs sm:text-sm">
							Prize Pool
						</p>
						<p className="truncate text-base font-medium sm:text-lg lg:text-xl">
							{lobby.currentAmount ? lobby.currentAmount : 0}{" "}
							{lobby.tokenSymbol || "STX"}
						</p>
					</div>

					<div className="space-y-0.5 sm:space-y-1">
						<p className="text-muted-foreground text-xs sm:text-sm">
							Players
						</p>
						<p className="text-base font-medium sm:text-lg lg:text-xl">
							{lobby.participantCount}/{game.maxPlayers}
						</p>
					</div>

					<div className="space-y-0.5 sm:space-y-1">
						<p className="text-muted-foreground text-xs sm:text-sm">
							Status
						</p>
						<p className="text-base font-medium capitalize sm:text-lg lg:text-xl">
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
