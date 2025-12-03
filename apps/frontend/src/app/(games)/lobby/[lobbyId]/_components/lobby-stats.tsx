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
		<div className="xs:grid-cols-2 grid grid-cols-1 gap-3 sm:grid-cols-3 sm:gap-4">
			{lobby.entryAmount !== null && (
				<Card className="bg-card/50 border-primary/10 hover:border-primary/20 backdrop-blur-sm transition-colors">
					<CardContent className="p-3 sm:p-4 md:p-6">
						<div className="flex items-center gap-2 sm:gap-3 md:gap-4">
							<div className="bg-primary/10 flex h-8 w-8 shrink-0 items-center justify-center rounded-full sm:h-10 sm:w-10 md:h-12 md:w-12">
								<Trophy className="text-primary h-4 w-4 sm:h-5 sm:w-5 md:h-6 md:w-6" />
							</div>
							<div className="min-w-0 flex-1">
								<p className="text-muted-foreground text-xs font-medium sm:text-sm">
									Pool Size
								</p>
								<p className="truncate text-base font-bold sm:text-xl md:text-2xl">
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

			<Card className="bg-card/50 border-primary/10 hover:border-primary/20 backdrop-blur-sm transition-colors">
				<CardContent className="p-3 sm:p-4 md:p-6">
					<div className="flex items-center gap-2 sm:gap-3 md:gap-4">
						<div className="bg-primary/10 flex h-8 w-8 shrink-0 items-center justify-center rounded-full sm:h-10 sm:w-10 md:h-12 md:w-12">
							<Users className="text-primary h-4 w-4 sm:h-5 sm:w-5 md:h-6 md:w-6" />
						</div>
						<div className="min-w-0 flex-1">
							<p className="text-muted-foreground text-xs font-medium sm:text-sm">
								Players
							</p>
							<p className="text-base font-bold sm:text-xl md:text-2xl">
								{players.length}
							</p>
						</div>
					</div>
				</CardContent>
			</Card>

			<Card className="bg-card/50 border-primary/10 hover:border-primary/20 xs:col-span-2 backdrop-blur-sm transition-colors sm:col-span-1">
				<CardContent className="p-3 sm:p-4 md:p-6">
					<div className="flex items-center gap-2 sm:gap-3 md:gap-4">
						<div className="bg-primary/10 flex h-8 w-8 shrink-0 items-center justify-center rounded-full sm:h-10 sm:w-10 md:h-12 md:w-12">
							<Gamepad2 className="text-primary h-4 w-4 sm:h-5 sm:w-5 md:h-6 md:w-6" />
						</div>
						<div className="min-w-0 flex-1">
							<p className="text-muted-foreground text-xs font-medium sm:text-sm">
								Game
							</p>
							<p className="truncate text-base font-bold sm:text-xl md:text-2xl">
								{lobby.name}
							</p>
							<p className="text-muted-foreground line-clamp-2 text-xs break-words sm:text-sm">
								{lobby.description}
							</p>
						</div>
					</div>
				</CardContent>
			</Card>
		</div>
	);
}
