import type { JoinRequest, PlayerState } from "@/lib/definitions";
import Player from "./player";
import { Clock, Users } from "lucide-react";

interface ParticipantsProps {
	players: PlayerState[];
	pendingPlayers: JoinRequest[];
	isCreator: boolean;
	onApprove?: (userId: string) => void;
	onReject?: (userId: string) => void;
	onKick?: (userId: string) => void;
}

export default function Participants({
	players,
	pendingPlayers,
	isCreator,
	onApprove,
	onReject,
	onKick,
}: ParticipantsProps) {
	return (
		<div className="border rounded-3xl p-4 sm:p-6 lg:p-8 space-y-4 sm:space-y-6">
			<div className="space-y-3 sm:space-y-4">
				<p className="text-base sm:text-lg lg:text-xl font-medium flex items-center gap-2">
					<Users className="size-4 sm:size-5" />
					<span>Participants</span>
				</p>
				<div className="space-y-2 sm:space-y-3">
					{players.map((player) => (
						<Player
							key={player.userId}
							player={player}
							isCreator={isCreator}
							onKick={onKick}
							kickActionKey={`kick-${player.userId}`}
						/>
					))}
				</div>
			</div>
			{pendingPlayers.length > 0 && (
				<div className="space-y-3 sm:space-y-4">
					<p className="text-base sm:text-lg lg:text-xl font-medium flex items-center gap-2">
						<Clock className="size-4 sm:size-5" />
						<span>Pending Requests</span>
					</p>
					<div className="space-y-2 sm:space-y-3">
						{pendingPlayers.map((pendingPlayer) => (
							<Player
								key={pendingPlayer.userId}
								player={pendingPlayer}
								isCreator={isCreator}
								onApprove={onApprove}
								onReject={onReject}
								approveActionKey={`approve-${pendingPlayer.userId}`}
								rejectActionKey={`reject-${pendingPlayer.userId}`}
							/>
						))}
					</div>
				</div>
			)}
		</div>
	);
}
