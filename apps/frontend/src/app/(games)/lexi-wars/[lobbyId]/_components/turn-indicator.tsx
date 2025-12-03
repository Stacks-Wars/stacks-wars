import { truncateAddress } from "@/lib/utils";
import { Player } from "@/types/schema/player";
import { User } from "lucide-react";
import { useEffect, useState } from "react";

interface TurnIndicatorProps {
	currentPlayer: Player | null;
	userId: string;
	countdown: number | null;
}

export default function TurnIndicator({
	currentPlayer,
	userId,
	countdown,
}: TurnIndicatorProps) {
	const [isCurrentPlayer, setIsCurrentPlayer] = useState<boolean>(false);

	useEffect(() => {
		if (currentPlayer && currentPlayer.id === userId) {
			setIsCurrentPlayer(true);
		} else {
			setIsCurrentPlayer(false);
		}
	}, [currentPlayer, userId]);

	return (
		<div
			className={`rounded-xl border-2 p-3 sm:p-4 ${isCurrentPlayer ? "border-green-500/20 bg-green-500/10" : "bg-yellow-500/10"} `}
		>
			<div className="flex items-center justify-between">
				<div className="flex items-center gap-3">
					<div className="flex size-8 items-center justify-center rounded-full bg-green-500/10">
						<User
							className={`size-4 ${isCurrentPlayer ? "text-green-500" : "text-yellow-500"} `}
						/>
					</div>
					<h3
						className={`text-base font-semibold ${isCurrentPlayer ? "text-green-500" : "text-yellow-500"} `}
					>
						{isCurrentPlayer
							? "It's Your Turn!"
							: `Waiting for ${
									currentPlayer?.user.displayName ||
									currentPlayer?.user.username ||
									truncateAddress(
										currentPlayer?.user.walletAddress
									)
								}${countdown !== null ? ` (${countdown}s)` : ""}`}
					</h3>
				</div>
			</div>
		</div>
	);
}
