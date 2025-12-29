import type { JoinRequest, PlayerState } from "@/lib/definitions";
import Player from "./player";
import { Clock, Users } from "lucide-react";

interface ParticipantsProps {
	players: PlayerState[];
	pendingPlayers: JoinRequest[];
	isCreator: boolean;
}

export default function Participants({
	players,
	pendingPlayers,
	isCreator,
}: ParticipantsProps) {
	return (
		<div className="border rounded-3xl p-8 space-y-6">
			<div className="space-y-4">
				<p className="text-xl font-medium flex items-center gap-2.5">
					<Users />
					<span>Participants</span>
				</p>
				<div>
					{players.map((player) => (
						<Player
							key={player.userId}
							player={player}
							isCreator={isCreator}
						/>
					))}
				</div>
			</div>
			<div className="space-y-4">
				<p className="text-xl font-medium flex items-center gap-2.5">
					<Clock />
					<span>Pending Requests</span>
				</p>
				<div>
					{pendingPlayers.map((pendingPlayer) => (
						<Player
							key={pendingPlayer.playerId}
							player={pendingPlayer}
							isCreator={isCreator}
						/>
					))}
				</div>
			</div>
		</div>
	);
}
