"use client";

import { Button } from "@/components/ui/button";
import { useRoom } from "@/lib/contexts/room-context";
import { useLobby, usePlayers, useIsActionLoading } from "@/lib/stores/room";
import { useUser } from "@/lib/stores/user";
import { Eye, Gamepad2 } from "lucide-react";

/**
 * Shows a participation toggle section for sponsored lobby creators.
 * Sponsored creators start as spectators (NotJoined) and can opt-in to participate.
 */
export default function SponsorParticipation() {
	const { sendLobbyMessage } = useRoom();
	const lobby = useLobby();
	const players = usePlayers();
	const user = useUser();
	const isToggling = useIsActionLoading("toggleParticipation");

	if (!lobby || !user) return null;

	const isCreator = user.id === lobby.creatorId;
	const isSponsored = lobby.isSponsored;

	// Only show for sponsored lobby creators
	if (!isCreator || !isSponsored) return null;

	// Don't show during starting, active game, or finished state
	if (
		lobby.status === "starting" ||
		lobby.status === "inProgress" ||
		lobby.status === "finished"
	)
		return null;

	const creatorPlayer = players.find((p) => p.userId === user.id);
	const isParticipating = creatorPlayer?.status === "joined";

	const handleToggle = () => {
		sendLobbyMessage({
			type: "toggleParticipation",
			participate: !isParticipating,
		});
	};

	return (
		<div className="flex w-full flex-col items-center">
			<div className="w-full rounded-3xl border p-4 sm:p-6 lg:p-8">
				<div className="flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between sm:gap-4">
					<div className="flex items-start gap-3">
						<div className="bg-muted mt-0.5 rounded-full p-2">
							{isParticipating ? (
								<Gamepad2 className="h-5 w-5" />
							) : (
								<Eye className="h-5 w-5" />
							)}
						</div>
						<div className="space-y-1">
							<p className="text-sm font-medium sm:text-base">
								{isParticipating
									? "You are participating in this game"
									: "You are currently a spectator"}
							</p>
							<p className="text-muted-foreground text-xs sm:text-sm">
								{isParticipating
									? "You will be included as a player when the game starts. You can switch back to spectating."
									: "As the sponsor, you are spectating by default. Click below to participate as a player."}
							</p>
						</div>
					</div>
					<Button
						variant={isParticipating ? "outline" : "default"}
						size="sm"
						className="w-full shrink-0 rounded-full px-4 py-3 sm:w-auto"
						onClick={handleToggle}
						disabled={isToggling}
					>
						{isToggling
							? "Updating..."
							: isParticipating
								? "Switch to Spectator"
								: "Participate"}
					</Button>
				</div>
			</div>
		</div>
	);
}
