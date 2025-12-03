import { Card, CardContent, CardFooter } from "@/components/ui/card";
import { Users } from "lucide-react";
import OpenLobby from "./open-lobby";
import ActiveLobbyHeader from "./active-lobby-header";
import { LobbyExtended } from "@/types/schema/lobby";
import { formatNumber } from "@/lib/utils";

export default function Lobbies({ lobbies }: { lobbies: LobbyExtended[] }) {
	return (
		<>
			{lobbies.map((lobby) => (
				<Card
					key={lobby.lobby.id}
					className="bg-primary/30 overflow-hidden"
				>
					<ActiveLobbyHeader lobby={lobby.lobby} />
					<div className="flex h-full flex-col justify-between">
						<CardContent className="pb-3">
							<div className="grid gap-2">
								<div className="flex justify-between">
									<span className="text-muted-foreground">
										Game:
									</span>
									<span className="font-medium">
										{lobby.lobby.game.name}
									</span>
								</div>
								{lobby.lobby.currentAmount !== null && (
									<div className="flex justify-between">
										<span className="text-muted-foreground">
											Pool size:
										</span>
										<span className="font-medium">
											{formatNumber(
												lobby.lobby.currentAmount
											)}{" "}
											{lobby.lobby.tokenSymbol}
										</span>
									</div>
								)}
								{lobby.lobby.entryAmount !== null && (
									<div className="flex justify-between">
										<span className="text-muted-foreground">
											Entry fee:
										</span>
										<span className="font-medium">
											{formatNumber(
												lobby.lobby.entryAmount
											)}{" "}
											{lobby.lobby.tokenSymbol}
										</span>
									</div>
								)}
								<div className="flex items-center justify-between">
									<span className="text-muted-foreground">
										Players:
									</span>
									<div className="flex items-center gap-1">
										<Users className="text-muted-foreground h-4 w-4" />
										<span className="font-medium">
											{lobby.lobby.participants}
											{/*/{lobby.maxPlayers}*/}
										</span>
									</div>
								</div>
							</div>
						</CardContent>
						<CardFooter>
							<OpenLobby lobby={lobby.lobby} />
						</CardFooter>
					</div>
				</Card>
			))}
		</>
	);
}
