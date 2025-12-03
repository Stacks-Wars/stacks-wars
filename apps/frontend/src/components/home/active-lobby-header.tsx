import { CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { truncateAddress } from "@/lib/utils";
import { Lobby } from "@/types/schema/lobby";

interface LobbyProps {
	lobby: Lobby;
}
export default function ActiveLobbyHeader({ lobby }: LobbyProps) {
	const now = Date.now();
	const creatorLastPingTime = lobby.creatorLastPing;
	const isCreatorActive = creatorLastPingTime
		? now - creatorLastPingTime <= 60000 // 60 seconds
		: false; // If no creatorLastPing, consider inactive

	return (
		<CardHeader className="pb-3">
			<div className="flex items-start justify-between">
				<CardTitle>{lobby.name}</CardTitle>
				<div className="flex items-center gap-2">
					{/*<Badge
						variant={
							lobby.state === "waiting" ? "default" : "secondary"
						}
					>
						{lobby.state === "waiting"
							? "Open"
							: lobby.state === "inProgress"
								? "In Progress"
								: "Closed"}
					</Badge>*/}
					<Badge
						variant="outline"
						className={`text-xs ${
							isCreatorActive
								? "border-emerald-500/20 bg-emerald-500/10 text-emerald-500"
								: "border-red-500/20 bg-red-500/10 text-red-500"
						}`}
					>
						{isCreatorActive ? "Active" : "Inactive"}
					</Badge>
				</div>
			</div>
			<CardDescription className="flex items-center justify-between gap-1">
				<p>
					Created by{" "}
					{lobby.creator.username ||
						truncateAddress(lobby.creator.walletAddress)}
				</p>
				{lobby.entryAmount === 0 && (
					<Badge variant={"outline"}>sponsored</Badge>
				)}
			</CardDescription>
		</CardHeader>
	);
}
