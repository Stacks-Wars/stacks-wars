import { Card, CardContent } from "@/components/ui/card";
import { Lobby } from "@/types/schema/lobby";
import { Player } from "@/types/schema/player";
import { formatNumber } from "@/lib/utils";
import { Gamepad2, Trophy, Users } from "lucide-react";

interface LobbyStatsProps {
	lobby: Lobby;
	players: Player[];
}

export default function LobbyStats({ lobby, players }: LobbyStatsProps) {
	return (
		<div className="grid grid-cols-1 xs:grid-cols-2 sm:grid-cols-3 gap-3 sm:gap-4">
			{lobby.entryAmount !== null && (
				<Card className="bg-card/50 backdrop-blur-sm border-primary/10 hover:border-primary/20 transition-colors">
					<CardContent className="p-3 sm:p-4 md:p-6">
						<div className="flex items-center gap-2 sm:gap-3 md:gap-4">
							<div className="h-8 w-8 sm:h-10 sm:w-10 md:h-12 md:w-12 rounded-full bg-primary/10 flex items-center justify-center shrink-0">
								<Trophy className="h-4 w-4 sm:h-5 sm:w-5 md:h-6 md:w-6 text-primary" />
							</div>
							<div className="min-w-0 flex-1">
								<p className="text-xs sm:text-sm font-medium text-muted-foreground">
									Pool Size
								</p>
								<p className="text-base sm:text-xl md:text-2xl font-bold truncate">
									{formatNumber(
										lobby.entryAmount !== 0
											? lobby.entryAmount * players.length
											: lobby.currentAmount || 0
									)}{" "}
									{lobby.tokenSymbol}
								</p>
							</div>
						</div>
					</CardContent>
				</Card>
			)}

			<Card className="bg-card/50 backdrop-blur-sm border-primary/10 hover:border-primary/20 transition-colors">
				<CardContent className="p-3 sm:p-4 md:p-6">
					<div className="flex items-center gap-2 sm:gap-3 md:gap-4">
						<div className="h-8 w-8 sm:h-10 sm:w-10 md:h-12 md:w-12 rounded-full bg-primary/10 flex items-center justify-center shrink-0">
							<Users className="h-4 w-4 sm:h-5 sm:w-5 md:h-6 md:w-6 text-primary" />
						</div>
						<div className="min-w-0 flex-1">
							<p className="text-xs sm:text-sm font-medium text-muted-foreground">
								Players
							</p>
							<p className="text-base sm:text-xl md:text-2xl font-bold">
								{players.length}
							</p>
						</div>
					</div>
				</CardContent>
			</Card>

			<Card className="bg-card/50 backdrop-blur-sm border-primary/10 hover:border-primary/20 transition-colors xs:col-span-2 sm:col-span-1">
				<CardContent className="p-3 sm:p-4 md:p-6">
					<div className="flex items-center gap-2 sm:gap-3 md:gap-4">
						<div className="h-8 w-8 sm:h-10 sm:w-10 md:h-12 md:w-12 rounded-full bg-primary/10 flex items-center justify-center shrink-0">
							<Gamepad2 className="h-4 w-4 sm:h-5 sm:w-5 md:h-6 md:w-6 text-primary" />
						</div>
						<div className="min-w-0 flex-1">
							<p className="text-xs sm:text-sm font-medium text-muted-foreground">
								Game
							</p>
							<p className="text-base sm:text-xl md:text-2xl font-bold truncate">
								{lobby.name}
							</p>
							<p className="text-xs sm:text-sm text-muted-foreground line-clamp-2 break-words">
								{lobby.description}
							</p>
						</div>
					</div>
				</CardContent>
			</Card>
		</div>
	);
}
